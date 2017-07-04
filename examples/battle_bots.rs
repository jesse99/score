//! This example simulates a collection of battle bots with different behaviors, e.g.
//! some of the bots flee from other bots and some are aggressive and attempt to attack
//! other bots.
#[macro_use]
extern crate clap;
extern crate glob;
extern crate rand;
#[macro_use]
extern crate rsimbase;

use clap::{App, ArgMatches};
use rand::Rng;
use rsimbase::*;
use std::collections::HashMap;
use std::f64::INFINITY;
use std::fmt::Display;
use std::io::{Write, stderr};
use std::process;
use std::str::FromStr;
use std::thread;

const MOVE_DELAY: f64 = 1.0;

#[derive(Clone)]
struct LocalConfig
{
	num_bots: i32,
	width: f64,
	height: f64,
}

impl LocalConfig
{
	fn new() -> LocalConfig
	{
		// These are the defaults: all of them can be overriden using command line options.
		LocalConfig {
			num_bots: 4,
			width: 50.0,
			height: 50.0,
		}
	}
}

type ComponentThread = fn (LocalConfig, ThreadData) -> ();

fn move_bot(id: ComponentID, effector: &mut Effector, x: f64, y: f64)
{
	let event = Event::with_payload("set-location", (x, y));
	effector.schedule_immediately(event, id);
}

fn offset_bot(id: ComponentID, effector: &mut Effector, x: f64, y: f64)
{
	let event = Event::with_payload("offset-location", (x, y));
	effector.schedule_immediately(event, id);
}

fn randomize_location(local: &LocalConfig, rng: &mut Box<Rng + Send>, id: ComponentID, effector: &mut Effector)
{
	let x = rng.gen_range(0.0, local.width);
	let y = rng.gen_range(0.0, local.height);
	move_bot(id, effector, x, y);
}

fn bot_dist_squared(local: &LocalConfig, state: &SimState, id1: ComponentID, id2: ComponentID, delta: &(f64, f64)) -> (f64, f64, f64)
{
	let p1 = state.components.path(id1);
	let x1 = state.store.get_float_data(&(p1.clone() + ".location-x"));
	let y1 = state.store.get_float_data(&(p1 + ".location-y"));
	
	let p2 = state.components.path(id2);
	let x2 = state.store.get_float_data(&(p2.clone() + ".location-x")) + delta.0;
	let y2 = state.store.get_float_data(&(p2 + ".location-y")) + delta.1;
	
	let x2 = x2.max(0.0).min(local.width);
	let y2 = y2.max(0.0).min(local.height);
	
	let dx = x1 - x2;
	let dy = y1 - y2;
	(dx*dx + dy*dy, dx, dy)
}

fn is_bot(state: &SimState, id: ComponentID) -> bool
{
	let path = state.components.path(id);
	let lpath = path.clone() + ".location-x";
	let epath = path + ".energy";
	state.store.has_data(&lpath) && state.store.get_int_data(&epath) > 0 && !state.was_removed(id)
}

fn count_bots(state: &SimState, id: ComponentID) -> i64
{
	let mut count = 0;
	
	let (_, root) = state.components.get_root(id);
	for id in root.children.iter() {	// TODO: use a HOF
		if is_bot(state, *id) {
			count += 1;
		}
	}
	
	return count
}

fn get_distance_to_nearby_bots(local: &LocalConfig, state: &SimState, data: &ThreadData, delta: &(f64, f64)) -> f64
{
	let mut dist = 0.0;
	
	let (_, root) = state.components.get_root(data.id);
	for id in root.children.iter() {
		if *id != data.id && is_bot(state, *id) {
			let (candidate, _, _) = bot_dist_squared(local, state, *id, data.id, delta);

			// Ignore bots that are far away.
			if candidate <= 5.0 {
				dist += candidate;
			}
		}
	}
	
	return dist
}

fn find_closest_bot(local: &LocalConfig, state: &SimState, data: &ThreadData) -> (ComponentID, f64, f64)
{
	let mut closest = NO_COMPONENT;
	let mut dx = INFINITY;
	let mut dy = INFINITY;
	let mut dist = INFINITY;
	
	let (_, root) = state.components.get_root(data.id);

	let delta = (0.0, 0.0);
	for id in root.children.iter() {
		if *id != data.id && is_bot(state, *id) {
			let (dist2, dx2, dy2) = bot_dist_squared(local, state, *id, data.id, &delta);
			if dist2 < dist {
				closest = *id;
				dx = dx2;
				dy = dy2;
				dist = dist2;
			}
		}
	}
	
	return (closest, dx, dy)
}

fn handle_cowardly_timer(local: &LocalConfig, effector: &mut Effector, state: &SimState, data: &ThreadData, mut energy: i64) -> i64
{
	if energy > 0 {
		// See which direction we can move (including not moving at all) which will put us the
		// furthest from other bots).
		let mut best_delta = (0.0, 0.0);
		let mut best_dist = INFINITY;
		let deltas = vec!((0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0));
		for delta in deltas.iter() {	// TODO: can we be slicker about this?
			let dist = get_distance_to_nearby_bots(local, &state, data, &delta);
			//log_info!(effector, "dist for {:?} = {:.1}", delta, dist);
			if dist < best_dist {
				best_delta = *delta;
				best_dist = dist;
			}
		}
		
		let delay = if best_delta.0 != 0.0 || best_delta.1 != 0.0 {
			log_excessive!(effector, "moving by {:?}", best_delta);
			offset_bot(data.id, effector, best_delta.0, best_delta.1);
			energy -= 1;
			MOVE_DELAY
		} else {
			log_excessive!(effector, "no others bots are nearby");
			MOVE_DELAY/2.0
		};

		let event = Event::new("timer");
		effector.schedule_after_secs(event, data.id, delay);
	} else {
		log_excessive!(effector, "dead");
	}
	energy
}

fn handle_aggresive_timer(local: &LocalConfig, effector: &mut Effector, state: &SimState, data: &ThreadData, mut energy: i64) -> i64
{
	// If we are very low health then just wait for someone to attack us and hope we still win.
	if energy > 10 {
		let (closest, dx, dy) = find_closest_bot(local, &state, data);
		if closest != NO_COMPONENT {
			if dx*dx + dy*dy <= 1.0 {
				let path = state.components.path(closest);
				log_info!(effector, "attacking {}", path);
				
				let event = Event::with_payload("attacked", (energy, data.id));
				effector.schedule_immediately(event, closest);
			} else {
				let delta = if dx.abs() > dy.abs() {
					if dx > 0.0 {
						(1.0, 0.0)
					} else {
						(-1.0, 0.0)
					}
				} else {
					if dy > 0.0 {
						(0.0, 1.0)
					} else {
						(0.0, -1.0)
					}
				};
				offset_bot(data.id, effector, delta.0, delta.1);
				energy -= 1;
			}

			let event = Event::new("timer");
			effector.schedule_after_secs(event, data.id, MOVE_DELAY);

		} else {
			log_debug!(effector, "didn't find a bot to chase");
		}
	} else {
		log_debug!(effector, "energy is to low to chase after anyone");
	}
	energy
}

fn handle_begin_attack(effector: &mut Effector, event: &Event, state: &SimState, energy: i64) -> i64
{
	let &(attacker_energy, attacker_id) = event.expect_payload::<(i64, ComponentID)>("was-attacked should have an (i64, ComponentID) payload");
	let attacker_path = state.components.path(attacker_id);
	
	if energy == 0 {
		log_info!(effector, "{} attacked a dead bot", attacker_path);	// TODO: handle this better
		let event = Event::with_payload("attacker-won", 0 as i64);
		effector.schedule_immediately(event, attacker_id);
		0
		
	} else if attacker_energy >= energy {	// this is the attackee lost case
		log_info!(effector, "{} won ({} >= {})", attacker_path, attacker_energy, energy);
		log_info!(effector, "{} bots left", count_bots(state, attacker_id)-1);
		effector.remove();
		let event = Event::with_payload("attacker-won", energy/2);
		effector.schedule_immediately(event, attacker_id);
		0
		
	} else {	// this is the attacker lost case (so it needs to remove itself because we can't)
		log_info!(effector, "{} lost ({} < {})", attacker_path, attacker_energy, energy);
		let event = Event::new("attacker-lost");
		effector.schedule_immediately(event, attacker_id);
		log_info!(effector, "energy is now {}", energy + attacker_energy/2);
		energy + attacker_energy/2
	}
}

fn cowardly_thread(local: LocalConfig, mut data: ThreadData)
{
	thread::spawn(move || {
		for (event, state) in data.rx.iter() {
			let mut effector = Effector::new();
			{
				let ename = &event.name;
				let energy = if ename == "init 0" {
					log_info!(effector, "initializing");
					randomize_location(&local, &mut data.rng, data.id, &mut effector);
	
					let event = Event::new("timer");
					let delay = 0.1 + 0.9*data.rng.next_f64();
					effector.schedule_after_secs(event, data.id, delay);
					100
					
				} else if ename == "timer" {
					let path = state.components.path(data.id);
					let energy = state.store.get_int_data(&(path + ".energy"));
					handle_cowardly_timer(&local, &mut effector, &state, &data, energy)
				
				} else if ename == "attacked" {
					let path = state.components.path(data.id);
					let energy = state.store.get_int_data(&(path + ".energy"));
					handle_begin_attack(&mut effector, &event, &state, energy)
								
				} else if handle_location_event(data.id, &state, &event, &mut effector) {
					let path = state.components.path(data.id);
					let energy = state.store.get_int_data(&(path + ".energy"));
					energy	// we've already accounted for the move cost
					
				} else {
					let cname = &(*state.components).get(data.id).name;
					panic!("component {} can't handle event {}", cname, ename);
				};
				effector.set_int_data("energy", energy);
			}
			
			drop(state);	// we need to do this before the send to ensure that our references are dropped before the Simulator processes the send
			let _ = data.tx.send(effector);
		}
	});
}

fn aggresive_thread(local: LocalConfig, mut data: ThreadData)
{
	thread::spawn(move || {
		for (event, state) in data.rx.iter() {
			let mut effector = Effector::new();
			{
				let ename = &event.name;
				let energy = if ename == "init 0" {
					log_info!(effector, "initializing");	// TODO: fn for this
					randomize_location(&local, &mut data.rng, data.id, &mut effector);
	
					let event = Event::new("timer");
					let delay = 0.1 + 0.9*data.rng.next_f64();
					effector.schedule_after_secs(event, data.id, delay);
					100
					
				} else if ename == "timer" {
					let path = state.components.path(data.id);
					let energy = state.store.get_int_data(&(path + ".energy"));
					handle_aggresive_timer(&local, &mut effector, &state, &data, energy)
				
				} else if ename == "attacked" {
					let path = state.components.path(data.id);
					let energy = state.store.get_int_data(&(path + ".energy"));
					handle_begin_attack(&mut effector, &event, &state, energy)
				
				} else if ename == "attacker-won" {
					let path = state.components.path(data.id);
					let energy = state.store.get_int_data(&(path + ".energy"));
					let bonus = event.expect_payload::<i64>("won-attack should have an i64 payload");
					log_info!(effector, "energy is now {}", energy + *bonus);
					energy + *bonus

				} else if ename == "attacker-lost" {
					log_info!(effector, "{} bots left", count_bots(&state, data.id)-1);
					effector.remove();	// this will drop the tx side of data.rx which will cause our this thread to exit
					0
				
				} else if handle_location_event(data.id, &state, &event, &mut effector) {
					let path = state.components.path(data.id);
					let energy = state.store.get_int_data(&(path + ".energy"));
				energy	// we've already accounted for the move cost
					
				} else {
					let cname = &(*state.components).get(data.id).name;
					panic!("component {} can't handle event {}", cname, ename);
				};
				effector.set_int_data("energy", energy);
			}
			
			drop(state);	// we need to do this before the send to ensure that our references are dropped before the Simulator processes the send
			let _ = data.tx.send(effector);
		}
	});
}

fn bots_have_changed(locations: &mut HashMap<String, i64>, state: &SimState) -> bool
{
	let mut moved = false;

	for (id, _) in state.components.iter() {
		let path = state.components.path(id);
		let path = path + ".energy";
		
		if state.store.has_data(&path) {
			let new_energy = state.store.get_int_data(&path);
			//print!("{} = {}\n", path, new_energy);
			if let Some(&old_energy) = locations.get(&path) {
				if new_energy != old_energy {
					moved = true;
				}
			} else {
				moved = true;
			}
			locations.insert(path, new_energy);
		}
	}
	
	moved
}

fn watchdog_thread(data: ThreadData)
{
	thread::spawn(move || {
		let mut locations = HashMap::new();

		for (event, state) in data.rx {
			let mut effector = Effector::new();
			{
				let ename = &event.name;
				if ename == "timer" {
					// The longest action bots take is movement so if none of the bots do anything
					// for a bit longer then that then we have reached a steady state and can stop
					// the sim.
					if !bots_have_changed(&mut locations, &state) {
						effector.exit();
					}
				}
			}
			
			let event = Event::new("timer");
			effector.schedule_after_secs(event, data.id, 1.1*MOVE_DELAY);

			drop(state);
			let _ = data.tx.send(effector);
		}
	});
}

fn fatal_err(message: &str) -> !
{
	let _ = writeln!(&mut stderr(), "{}", message);
	process::exit(1);
}

// Min and max are inclusive.
fn match_num<T>(matches: &ArgMatches, name: &str, min: T, max: T) -> T
		where T: Copy + Display + FromStr + PartialOrd
{
	match value_t!(matches.value_of(name), T) {
		Ok(value) if value < min => fatal_err(&format!("--{} should be greater than {}", name, min)),
		Ok(value) if value > max => fatal_err(&format!("--{} should be less than {}", name, max)),
		Ok(value) => value,
		_ => fatal_err(&format!("--{} should be a number", name)),
	}
}

fn new_random_thread(rng: &mut Box<Rng + Send>, index: i32) -> (String, ComponentThread)
{
	// The sim is really boring if all the bots are cowardly so we'll ensure
	// that we have at least one aggressive bot.
	if index == 0 || rng.gen_weighted_bool(2) {
		(format!("aggresive-{}", index), aggresive_thread)
	} else {
		(format!("cowardly-{}", index), cowardly_thread)
	}
}

fn create_sim(local: LocalConfig, config: Config) -> Simulation
{
	let mut sim = Simulation::new(config);
	let world = sim.add_component("world", NO_COMPONENT);
	for i in 0..local.num_bots {
		let (name, thread) = new_random_thread(sim.rng(), i);
		let _ = sim.add_active_component(&name, world, |data| thread(local.clone(), data));
	}
	let _ = sim.add_active_component("watch-dog", world, watchdog_thread);
	sim
}

fn parse_options() -> (LocalConfig, Config)
{
	let mut local = LocalConfig::new();
	let mut config = Config::new();
	
	// see https://docs.rs/clap/2.24.2/clap/struct.Arg.html#method.from_usage for syntax
	let usage = format!(
		"--height=[N] 'Max number of times bots can move up without running into a wall [{default_height}]'
		--log=[LEVEL:GLOB]... 'Overrides --log-level, glob is used to match component names'
		--log-level=[LEVEL] 'Default log level: {log_levels} [{default_level}]'
		--max-time=[TIME] 'Maximum time to run the simulation, use {time_suffixes} suffixes [no limit]'
		--no-colors 'Don't color code console output'
		--num-bots=[N] 'Number of bots to start out with [{default_bots}]'
		--seed=[N] 'Random number generator seed [random]'
		--width=[N] 'Max number of times bots can move right without wrapping [{default_width}]'",
		default_height = local.height,
		default_width = local.width,
		default_bots = local.num_bots,
		default_level = format!("{:?}", config.log_level).to_lowercase(),
		log_levels = log_levels(),
		time_suffixes = time_suffixes());
	
	let matches = App::new("battle-bots")
		.version("1.0")
		.author("Jesse Jones <jesse9jones@gmail.com>")
		.about("Simulates bots that do battle with one another.")
		.args_from_usage(&usage)
	.get_matches();
		
	if matches.is_present("height") {
		local.height = match_num(&matches, "height", 10, 1_000) as f64;
	}
	if matches.is_present("width") {
		local.width = match_num(&matches, "height", 10, 1_000) as f64;
	}
	if matches.is_present("num-bots") {
		local.num_bots = match_num(&matches, "num-bots", 1, 100);
	}
	
	if matches.is_present("seed") {
		config.seed = match_num(&matches, "seed", 1, u32::max_value());
	}
	
	if matches.is_present("log-level") {
		if let Some(e) = config.parse_log_level(matches.value_of("log-level").unwrap()) {
			fatal_err(&e);
		}
	}

	if matches.is_present("log") {
		if let Some(e) = config.parse_log_levels(matches.values_of("log").unwrap().collect()) {
			fatal_err(&e);
		}
	}
	
	let max_secs = matches.value_of("max-time").unwrap_or("");
	if !max_secs.is_empty() {
		if let Some(e) = config.parse_max_secs(max_secs) {
			fatal_err(&e);
		}
	}
	
	config.colorize = !matches.is_present("no-colors");
	
	(local, config)
}

fn main()
{
	let (local, mut config) = parse_options();
	config.time_units = 1000.0;	// ms
	
	let mut sim = create_sim(local, config);
	sim.run();
}
