//! This example is a fairly crude simulation of the telephone game, aka chinese whispers.
//! Instead of garbling a message at each step we randomly replace letters with a dashes.
//! When a message is received that contains all dashes we terminate the simulation.
//! It's a simple simulation but structured similarly to many more complex simulations.
#[macro_use]
extern crate clap;
extern crate rand;
#[macro_use]
extern crate score;

use clap::{App, ArgMatches};
use rand::Rng;
use score::*;
use std::fmt::Display;
use std::io::{Write, stderr};
use std::process;
use std::str::FromStr;
use std::sync::Arc;
use std::thread;

const WIDTH: f64 = 32.0;
const HEIGHT: f64 = 32.0;

const POEM: &str = "Tyger Tyger, burning bright,\nIn the forests of the night;\nWhat immortal hand or eye,\nCould frame thy fearful symmetry?\n\nIn what distant deeps or skies.\nBurnt the fire of thine eyes?\nOn what wings dare he aspire?\nWhat the hand, dare seize the fire?\n\nAnd what shoulder, & what art,\nCould twist the sinews of thy heart?\nAnd when thy heart began to beat,\nWhat dread hand? & what dread feet?\n\nWhat the hammer? what the chain,\nIn what furnace was thy brain?\nWhat the anvil? what dread grasp,\nDare its deadly terrors clasp!\n\nWhen the stars threw down their spears\nAnd water'd heaven with their tears:\nDid he smile his work to see?\nDid he who made the Lamb make thee?\n\nTyger Tyger burning bright,\nIn the forests of the night:\nWhat immortal hand or eye,\nDare frame thy fearful symmetry?";

#[derive(Clone)]
struct LocalConfig
{
	num_repeaters: i32,
	error_rate: u32,
}

impl LocalConfig
{
	fn new() -> LocalConfig
	{
		// These are the defaults: all of them can be overriden using command line options.
		LocalConfig {
			num_repeaters: 10,
			error_rate: 100,
		}
	}
}

fn compute_error(text: &str) -> f64
{
	let count = text.chars().count();
	let errors = text.chars().fold(0, |sum, c| if c == '-' {sum+1} else {sum});
	100.0*(errors as f64)/(count as f64)
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

// TODO: group these into devices
struct SenderComponent
{
	id: ComponentID,
	data: ThreadData,
	send_down: OutPort<String>,
}

impl SenderComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> SenderComponent
	{
		let (id, data) = sim.add_active_component("sender", parent_id);
		SenderComponent {
			id: id,
			data: data,
			send_down: OutPort::new(),
		}
	}
	
	pub fn start(self)
	{
		thread::spawn(move || {
			// "init N" events are scheduled by the simulation. All other events are scheduled
			// by component threads. Components may send an event to a different component.
			// SimState encapsulates the state of the simulation at the time the event was
			// dispatched. TODO: talk about what each of these args are 
			process_events!(self.data, event, state, effector,
				"init 0" => {
					// The only way components can affect the simulation state is through an
					// Effector. This prevents spooky action at a distance and also allows
					// component threads to execute in parallel.
					log_info!(effector, "init");
					let event = Event::with_payload("set-location", (WIDTH/2.0, HEIGHT/2.0));
					effector.schedule_immediately(event, self.id);
				
					let event = Event::new("timer");
					effector.schedule_immediately(event, self.id);
				},
				"timer" => {
					self.send_down.send_payload(&mut effector, "text", POEM.to_string());
	
					let event = Event::new("timer");
					effector.schedule_after_secs(event, self.id, 1.0);
				},
				"set-location" => {	// TODO: for now can't do "xxx" | "xxx" which is part of https://github.com/rust-lang/rust/issues/30450
					log_info!(effector, "set-location");
					handle_location_event(self.id, &state, &event, &mut effector);
				},
				"offset-location" => {
					log_info!(effector, "offset-location");
					handle_location_event(self.id, &state, &event, &mut effector);
				}
			);
		});
	}
}

struct ManglerComponent
{
	id: ComponentID,
	data: ThreadData,
	error_rate: u32,
	sent_down: InPort<String>,
	send_down: OutPort<String>,
}

impl ManglerComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID, error_rate: u32) -> ManglerComponent
	{
		let (id, data) = sim.add_active_component("mangler", parent_id);
		ManglerComponent {
			id: id,
			data: data,
			error_rate: error_rate,
			sent_down: InPort::new(),
			send_down: OutPort::new(),
		}
	}
	
	pub fn start(mut self)
	{
		thread::spawn(move || {
			process_events!(self.data, event, state, effector,
				"init 0" => {
					let event = Event::with_payload("set-location", (WIDTH + WIDTH/2.0, HEIGHT/2.0));
					effector.schedule_immediately(event, self.id);
				},
				"text" => {
					let old = event.expect_payload::<String>("text should have a String payload");
					
					let mut new = "".to_string();
					for ch in old.chars() {
						if self.data.rng.gen_weighted_bool(self.error_rate) {
							new.push('-');
						} else {
							new.push(ch);
						}
					}
					
					self.send_down.send_payload(&mut effector, "text", new);
				},
				"poke" => {
					log_info!(effector, "poked");
				},
				"set-location" => {
					handle_location_event(self.id, &state, &event, &mut effector);
				},
				"offset-location" => {
					handle_location_event(self.id, &state, &event, &mut effector);
				}
			);
		});
	}
}

struct StatsComponent
{
	id: ComponentID,
	data: ThreadData,
	sent_down: InPort<String>,
	send_down: OutPort<String>,

	err_percent: FloatValue,
}

impl StatsComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> StatsComponent
	{
		let (id, data) = sim.add_active_component("stats", parent_id);
		StatsComponent {
			id: id,
			data: data,
			sent_down: InPort::new(),
			send_down: OutPort::new(),
			err_percent: FloatValue{},
		}
	}
	
	pub fn start(self)
	{
		thread::spawn(move || {
			process_events!(self.data, event, state, effector,
				"init 0" => {
					let event = Event::with_payload("set-location", (2.0*WIDTH + WIDTH/2.0, HEIGHT/2.0));	// TODO: this isn't right
					effector.schedule_immediately(event, self.id);
				},
				"text" => {
					let text = event.expect_payload::<String>("text should have a String payload");
					let err = compute_error(text);
					log_info!(effector, "found {:.1}% error rate", err);
					set_value!(effector, self.err_percent = err);
	
					self.send_down.send_payload(&mut effector, "text", text.to_string());
				},
				"set-location" => {
					handle_location_event(self.id, &state, &event, &mut effector);
				},
				"offset-location" => {
					handle_location_event(self.id, &state, &event, &mut effector);
				}
			);
		});
	}
}

struct ReceiverComponent
{
	id: ComponentID,
	data: ThreadData,
	sent_down: InPort<String>,
}

impl ReceiverComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> ReceiverComponent
	{
		let (id, data) = sim.add_active_component("receiver", parent_id);
		ReceiverComponent {
			id: id,
			data: data,
			sent_down: InPort::new(),
		}
	}
	
	pub fn start(self)
	{
		thread::spawn(move || {
			process_events!(self.data, event, state, effector,
				"init 0" => {
					let event = Event::with_payload("set-location", (2.0*WIDTH + WIDTH/2.0, HEIGHT/2.0));
					effector.schedule_immediately(event, self.id);
				},
				"text" => {
					let text = event.expect_payload::<String>("text should have a String payload");
					let err = compute_error(&text);
					log_info!(effector, "{:.1}% error rate", err);
					if err > 99.0 {
						effector.exit();
					}
				},
				"set-location" => {
					handle_location_event(self.id, &state, &event, &mut effector);
				},
				"offset-location" => {
					handle_location_event(self.id, &state, &event, &mut effector);
				}
			);
		});
	}
}

fn create_sim(local: LocalConfig, config: Config) -> Simulation
{
	// These components are arranged very much like a network stack with messages sent
	// down the stack, transmitted to a different device, sent up the network stack and
	// then back down to be forwarded to yet another device:
	//
	// sender  repeater0  repeater1  receiver
	//   |        ||         ||         |
	//   |      stats      stats      stats
	//   |        ||         ||         |
	// mangle - mangle --- mangle --- mangle
	let mut sim = Simulation::new(config);
	let world_id = sim.add_component("world", NO_COMPONENT);
	{
	let store = Arc::get_mut(&mut sim.store).unwrap();
		store.set_float("world.display-size-x", WIDTH*(2.0 + local.num_repeaters as f64), Time(0));
		store.set_float("world.display-size-y", HEIGHT, Time(0));
	}

	// TODO: these should be grouped within some sort of locatable component
	// Sender just sends messages down.
	// Manglers mangle inbound messages and send them up. Manglers send downward messages to outbound.
	let mut sender = SenderComponent::new(&mut sim, world_id);
	let mut mangler = ManglerComponent::new(&mut sim, world_id, local.error_rate);
	let mut stats = StatsComponent::new(&mut sim, world_id);
	let receiver = ReceiverComponent::new(&mut sim, world_id);
	
	connect!(sender.send_down -> mangler.sent_down);
	connect!(mangler.send_down -> stats.sent_down);
	connect!(stats.send_down -> receiver.sent_down);
	
	sender.start();
	mangler.start();
	stats.start();
	receiver.start();
		
	sim
}

fn parse_options() -> (LocalConfig, Config)
{
	let mut local = LocalConfig::new();
	let mut config = Config::new();
	
	// see https://docs.rs/clap/2.24.2/clap/struct.Arg.html#method.from_usage for syntax
	let usage = format!(
		"--error=[N] 'Each step has a 1 in N chance of garbling a letter [{default_error}]'
		--log=[LEVEL:GLOB]... 'Overrides --log-level, glob is used to match component names'
		--log-level=[LEVEL] 'Default log level: {log_levels} [{default_level}]'
		--max-time=[TIME] 'Maximum time to run the simulation, use {time_suffixes} suffixes [no limit]'
		--no-colors 'Don't color code console output'
		--repeaters=[N] 'Number of steps between the sender and receiver [{default_repeaters}]'
		--seed=[N] 'Random number generator seed [random]'
		--server 'Startup a web server so sdebug can be used'",
		default_repeaters = local.num_repeaters,
		default_error = local.error_rate,
		default_level = format!("{:?}", config.log_level).to_lowercase(),
		log_levels = log_levels(),
		time_suffixes = time_suffixes());
	
	let matches = App::new("telephone")
		.version("1.0")
		.author("Jesse Jones <jesse9jones@gmail.com>")
		.about("Simulates the telephone game.")
		.args_from_usage(&usage)
	.get_matches();
		
	if matches.is_present("error") {
		local.error_rate = match_num(&matches, "error", 2, 10_000);
	}
	
	if matches.is_present("repeaters") {
		local.num_repeaters = match_num(&matches, "repeaters", 1, 100);
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
	config.time_units = 10.0;	// tenths of seconds
	
	let mut sim = create_sim(local, config);
	sim.run();
}
