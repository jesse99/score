//! This example simulates a collection of battle bots with different behaviors, e.g.
//! some of the bots flee from other bots and some are aggressive and attempt to attack
//! other bots.
#[macro_use]
extern crate clap;
extern crate glob;
extern crate rand;
#[macro_use]
extern crate score;

use clap::{App, ArgMatches};
use rand::Rng;
use score::*;
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

// These events are handled by score's handle_location_event function.
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

// When a bot's energy goes to zero we consider it to be dead and remove it (which switches in a
// do-nothing thread so that it stops responding to events and also adds a removed flag to the store).
fn is_bot(state: &SimState, id: ComponentID) -> bool
{
	let path = state.components.path(id);
	let lpath = path.clone() + ".location-x";
	let epath = path + ".energy";
	state.store.has_data(&lpath) && state.store.get_int_data(&epath) > 0 && !state.was_removed(id)
}

fn count_bots(state: &SimState, id: ComponentID) -> i64
{
	let (_, root) = state.components.get_root(id);
	root.children.iter().filter(|&id| is_bot(state, *id)).fold(0, |sum, _| sum + 1)
}

fn get_distance_to_nearby_bots(local: &LocalConfig, state: &SimState, data: &ThreadData, delta: &(f64, f64)) -> f64
{
	let (_, root) = state.components.get_root(data.id);
	root.children.iter()
		.filter(|&id| *id != data.id && is_bot(state, *id))
		.fold(0.0, |dist, &id| {
			// Ignore bots that are far away.
			let (candidate, _, _) = bot_dist_squared(local, state, id, data.id, delta);
			if candidate <= 16.0 {dist + candidate} else {dist}
		})
}

fn find_closest_bot(local: &LocalConfig, state: &SimState, data: &ThreadData) -> (ComponentID, f64, f64)
{
	let zero = (0.0, 0.0);
	let (_, root) = state.components.get_root(data.id);
	let result = root.children.iter()
		.filter(|&id| *id != data.id && is_bot(state, *id))
		
		//     0=id          1=dx      2=dy      3=dist
		.fold((NO_COMPONENT, INFINITY, INFINITY, INFINITY), |closest, &id| {
			let (new_dist, dx, dy) = bot_dist_squared(local, state, id, data.id, &zero);
			if new_dist < closest.3 {
				(id, dx, dy, new_dist)
			} else {
				closest
			}
		});
	(result.0, result.1, result.2)
}

fn dir_furthest_from_other_bots(local: &LocalConfig, state: &SimState, data: &ThreadData) -> (f64, f64)
{
	// See which direction we can move (including not moving at all) which will put us the
	// furthest from other bots).
	let deltas = vec!((0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (-1.0, 0.0), (0.0, -1.0));
	let result = deltas.iter()
		//      0=delta    1=dist
		.fold(((0.0, 0.0), INFINITY), |best, delta| {
			let dist = get_distance_to_nearby_bots(local, state, data, delta);
			if dist < best.1 {
				(*delta, dist)
			} else {
				best
			}
		});
	result.0
}

fn init_bot(local: &LocalConfig, id: ComponentID, rng: &mut Box<Rng + Send>, state: &SimState, event: &Event, effector: &mut Effector)
{
	// The only way components can affect the simulation state is through an
	// Effector. This prevents spooky action at a distance and also allows
	// component threads to execute in parallel.
	effector.set_description("energy", "Amount of health the bot has.");	// all data in the store needs a description
	handle_location_event(id, state, event, effector);
	randomize_location(&local, rng, id, effector);

	let event = Event::new("timer");
	let delay = 0.1 + 0.9*rng.next_f64();
	effector.schedule_after_secs(event, id, delay);
	effector.set_int_data("energy", 100);
}

// This bot will run from all the other bots and will never initiate an attack.
fn cowardly_thread(local: LocalConfig, mut data: ThreadData)
{
	thread::spawn(move || {
		// "init N" events are scheduled by the simulation. All other events are scheduled
		// by component threads. Components may send an event to a different component.
		// SimState encapsulates the state of the simulation at the time the event was
		// dispatched.
		process_events!(data, event, state, effector,
			"init 0" => {
				init_bot(&local, data.id, &mut data.rng, &state, &event, &mut effector);
			},
			"timer" => {
				let path = state.components.path(data.id);
				let energy = state.store.get_int_data(&(path + ".energy"));
				assert!(energy > 0, "energy was {}", energy);	// should be removed once energy hits zero

				// If we have enough energy to move then see which direction would be furthest
				// from all the other bots (including not moving at all).
				let delay = if energy > 1 {
					let best_delta = dir_furthest_from_other_bots(&local, &state, &data);
					if best_delta.0 != 0.0 || best_delta.1 != 0.0 {
						log_excessive!(effector, "moving by {:?}", best_delta);
						offset_bot(data.id, &mut effector, best_delta.0, best_delta.1);
						effector.set_int_data("energy", energy - 1);
						MOVE_DELAY
					} else {
						log_excessive!(effector, "no others bots are nearby");
						MOVE_DELAY/2.0
					}
				} else {
					MOVE_DELAY
				};
		
				// We should always schedule our timer, e.g. if we're really low on energy
				// someone could attack us and if we win then we'll want to have an opportunity
				// to begin running again.
				let event = Event::new("timer");
				effector.schedule_after_secs(event, data.id, delay);
			},
			"won-attack" => {
				let path = state.components.path(data.id);
				let energy = state.store.get_int_data(&(path + ".energy"));
				let bonus = event.expect_payload::<i64>("won-attack should have an i64 payload");
				log_info!(effector, "energy is now {}", energy + *bonus);
				effector.set_int_data("energy", energy + *bonus);
			},
			"lost-attack" => {
				effector.set_int_data("energy", 0);
				effector.remove();	// this will drop the tx side of data.rx which will cause our this thread to exit
			},
			"set-location" => {	// TODO: for now can't do "xxx" | "xxx" which is part of https://github.com/rust-lang/rust/issues/30450
				// Don't need to fiddle with energy because it has already been accounted for.
				handle_location_event(data.id, &state, &event, &mut effector);
			},
			"offset-location" => {
				handle_location_event(data.id, &state, &event, &mut effector);
			}
		);
	});
}

// Components can read each others state but they cannot change other components so when a bot
// attacks another bot it figures out who won or lost and then sends a "won-attack" or "lost-attack"
// event to the other bot so that it can update its state.
fn handle_attack(effector: &mut Effector, state: &SimState, my_id: ComponentID, their_id: ComponentID)
{
	let my_path = state.components.path(my_id);
	let my_energy = state.store.get_int_data(&(my_path.clone() + ".energy"));

	let their_path = state.components.path(their_id);
	let their_energy = state.store.get_int_data(&(their_path.clone() + ".energy"));
	
	if my_energy >= their_energy {
		log_info!(effector, "{} lost ({} >= {})", their_path, my_energy, their_energy);
		let gained = their_energy/2;
		log_info!(effector, "energy is now {}", my_energy + gained);
		log_info!(effector, "{} bots left", count_bots(state, my_id)-1);
		let event = Event::with_payload("lost-attack", their_energy/2);
		effector.schedule_immediately(event, their_id);
		effector.set_int_data("energy", my_energy + gained);
		
	} else {
		log_info!(effector, "{} won ({} < {})", their_path, my_energy, their_energy);
		effector.remove();
		let event = Event::with_payload("won-attack", my_energy/2);
		effector.schedule_immediately(event, their_id);
		effector.set_int_data("energy", 0);
	}
}

fn handle_chase(effector: &mut Effector, state: &SimState, dx: f64, dy: f64, my_id: ComponentID, their_id: ComponentID)
{
	let my_path = state.components.path(my_id);
	let my_energy = state.store.get_int_data(&(my_path.clone() + ".energy"));

	let their_path = state.components.path(their_id);
	log_info!(effector, "chasing {}", their_path);
	
	let delta = if dx.abs() > dy.abs() {
		if dx > 0.0 {(1.0, 0.0)} else {(-1.0, 0.0)}
	} else {
		if dy > 0.0 {(0.0, 1.0)} else {(0.0, -1.0)}
	};
	offset_bot(my_id, effector, delta.0, delta.1);
	effector.set_int_data("energy", my_energy - 1);
}

// This bot will chase the closest bot to it and attack bots that are nearby.
fn aggresive_thread(local: LocalConfig, mut data: ThreadData)
{
	thread::spawn(move || {
		process_events!(data, event, state, effector,
			"init 0" => {
				init_bot(&local, data.id, &mut data.rng, &state, &event, &mut effector);
			},
			"timer" => {
				let path = state.components.path(data.id);
				let energy = state.store.get_int_data(&(path + ".energy"));
				assert!(energy > 0, "energy was {}", energy);	// should be removed once energy hits zero

				if energy > 10 {
					let (closest, dx, dy) = find_closest_bot(&local, &state, &data);
					if closest != NO_COMPONENT {
						if dx*dx + dy*dy <= 1.0 {
							handle_attack(&mut effector, &state, data.id, closest);
						} else {
							handle_chase(&mut effector, &state, dx, dy, data.id, closest);
						}
				
					} else {
						log_debug!(effector, "didn't find a bot to chase");
					}

				} else {
					// If we are very low health then just wait for someone to get close
					// and hope we still win.
					log_debug!(effector, "energy is to low to chase after anyone");
				}
		
				let event = Event::new("timer");
				effector.schedule_after_secs(event, data.id, MOVE_DELAY);
			},
			"won-attack" => {
				let path = state.components.path(data.id);
				let energy = state.store.get_int_data(&(path + ".energy"));
				let bonus = event.expect_payload::<i64>("won-attack should have an i64 payload");
				log_info!(effector, "energy is now {}", energy + *bonus);
				effector.set_int_data("energy", energy + *bonus);
			},
			"lost-attack" => {
				effector.set_int_data("energy", 0);
				effector.remove();	// this will drop the tx side of data.rx which will cause our this thread to exit
			},
			"set-location" => {
				// Don't need to fiddle with energy because it has already been accounted for.
				handle_location_event(data.id, &state, &event, &mut effector);
			},
			"offset-location" => {
				handle_location_event(data.id, &state, &event, &mut effector);
			}
		);
	});
}

// Everything a bot does (except just sitting in place) costs energy so if a bot's
// energy changes something significant happened.
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

		process_events!(data, event, state, effector,
			"init 0" => {
				let event = Event::new("timer");
				effector.schedule_after_secs(event, data.id, 1.1*MOVE_DELAY);
			},
			"timer" => {
				// The longest action bots take is movement so if none of the bots do anything
				// for a bit longer then that then we have reached a steady state and can stop
				// the sim.
				if !bots_have_changed(&mut locations, &state) {
					effector.exit();
				} else {
					let event = Event::new("timer");
					effector.schedule_after_secs(event, data.id, 1.1*MOVE_DELAY);
				}
			}
		);
	});
}

fn world_thread(local: LocalConfig, data: ThreadData)
{
	thread::spawn(move || {
		process_events!(data, event, state, effector,
			"init 0" => {
				// It's nice to log important configuration details so that they can be seen
				// when reviewing a saved run.
				log_info!(effector, "num-bots = {}", local.num_bots);
				log_info!(effector, "height = {}", local.height);
				log_info!(effector, "width = {}", local.width);
				log_info!(effector, "processing {}", event.name);
			}
		);
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
	let world = sim.add_active_component("world", NO_COMPONENT, |data| world_thread(local.clone(), data));
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
		--server 'Startup a web server so sdebug can be used'
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
	
	if matches.is_present("server") {
		config.address = "127.0.0.1:9000".to_string();
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
