// Copyright (C) 2017 Jesse Jones
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 3, or (at your option)
// any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software Foundation,
// Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.

//! This example is a fairly crude simulation of the telephone game, aka chinese whispers.
//! Instead of garbling a message at each step we randomly replace letters with dashes.
//! When a message is received that contains all dashes we terminate the simulation.
//! It's a simple simulation but structured similarly to many more complex simulations.
#[macro_use]
extern crate clap;
extern crate rand;
#[macro_use]
extern crate score;

use clap::{App, ArgMatches};
use rand::{Rng, SeedableRng, StdRng};
use score::*;
use std::fmt::Display;
use std::process;
use std::str::FromStr;
use std::thread;

const DISPLAY_WIDTH: f64 = 50.0;
const DISPLAY_HEIGHT: f64 = 100.0;

const START_X: f64 = 25.0;
const START_Y: f64 = 5.0;
const DY: f64 = 10.0;

// This is the message we send.
const POEM: &str = "Tyger Tyger, burning bright,\nIn the forests of the night;\nWhat immortal hand or eye,\nCould frame thy fearful symmetry?\n\nIn what distant deeps or skies.\nBurnt the fire of thine eyes?\nOn what wings dare he aspire?\nWhat the hand, dare seize the fire?\n\nAnd what shoulder, & what art,\nCould twist the sinews of thy heart?\nAnd when thy heart began to beat,\nWhat dread hand? & what dread feet?\n\nWhat the hammer? what the chain,\nIn what furnace was thy brain?\nWhat the anvil? what dread grasp,\nDare its deadly terrors clasp!\n\nWhen the stars threw down their spears\nAnd water'd heaven with their tears:\nDid he smile his work to see?\nDid he who made the Lamb make thee?\n\nTyger Tyger burning bright,\nIn the forests of the night:\nWhat immortal hand or eye,\nDare frame thy fearful symmetry?";

#[derive(Clone)]
struct LocalConfig
{
	// Repeaters garble the message and forward it along.
	num_repeaters: i32,
	
	// Each letter is replaced with a dash with probability of 1 in error_rate.
	error_rate: u32,
}

impl LocalConfig
{
	fn new() -> LocalConfig
	{
		// These are the defaults: all of them can be overriden using command line options.
		LocalConfig {
			num_repeaters: 5,
			error_rate: 100,
		}
	}
}

// Returns the error percentage of the text, i.e. how many letters were replaced by dashes.
fn compute_error(text: &str) -> f64
{
	let count = text.chars().count();
	let errors = text.chars().fold(0, |sum, c| if c == '-' {sum+1} else {sum});
	100.0*(errors as f64)/(count as f64)
}

// This is typical of more elaborate simulations: components are organized into hierarchies that act
// as black boxes where the ports on the outer component are connected to the inner components. It's
// not required that structs be set up this way (or that you use a struct like this at all) but doing
// so makes the device's inputs, outputs, and structure clearer.
struct SenderDevice
{
	id: ComponentID,
	sender: SenderComponent,
	mangler: ManglerComponent,
	outbound: OutPort<String>,
}

impl SenderDevice
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID, error_rate: u32) -> SenderDevice
	{
		let id = sim.add_component("sender", parent_id);
		SenderDevice {
			id: id,
			sender: SenderComponent::new(sim, id),
			mangler: ManglerComponent::new(sim, id, error_rate),
			outbound: OutPort::new(),
		}
	}
	
	pub fn start(mut self, sim: &mut Simulation)
	{
		// Wire together the sender and the mangler.
		self.sender.output.connect_to(&self.mangler.upper_in);
		self.mangler.output = self.outbound.clone();
		
		// Spin up the sender and mangler threads.
		self.sender.start();
		self.mangler.start();
		
		// Set some state for the device. We could use a thread to do this but it's simpler
		// to just use an Effector.
		let mut effector = Effector::new();
		effector.set_string("display-name", "sender-0");
		effector.set_string("display-color", "blue");
		effector.set_float("display-location-x", START_X);
		effector.set_float("display-location-y", START_Y);
		sim.apply(self.id, effector);
	}
}

struct RepeaterDevice
{
	id: ComponentID,
	index: i32,
	repeater: RepeaterComponent,
	stats: StatsComponent,
	mangler: ManglerComponent,
	
	inbound: InPort<String>,
	outbound: OutPort<String>,
}

impl RepeaterDevice
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID, error_rate: u32, i: i32) -> RepeaterDevice
	{
		let name = format!("repeater{}", i);
		let id = sim.add_component(&name, parent_id);
		let mut device = RepeaterDevice {
			id: id,
			index: i,
			repeater: RepeaterComponent::new(sim, id),
			stats: StatsComponent::new(sim, id),
			mangler: ManglerComponent::new(sim, id, error_rate),
			inbound: InPort::new(id),
			outbound: OutPort::new(),
		};
		device.inbound = device.mangler.input.clone();
		device
	}
	
	pub fn start(mut self, sim: &mut Simulation)
	{
		// Wire together the repeater and stats.
		self.repeater.lower_out.connect_to(&self.stats.upper_in);
		self.stats.upper_out.connect_to(&self.repeater.lower_in);
		
		// Wire together stats and mangler.
		self.stats.lower_out.connect_to(&self.mangler.upper_in);
		self.mangler.upper_out.connect_to(&self.stats.lower_in);
	
		// Mangler output goes where ever the device was connected to.
		self.mangler.output = self.outbound.clone();
		
		// Spin up the threads.
		self.repeater.start();
		self.stats.start();
		self.mangler.start();
		
		// Set our state.
		let mut effector = Effector::new();
		effector.set_string("display-name", &format!("repeat-{}", self.index));
		effector.set_float("display-location-x", START_X);
		effector.set_float("display-location-y", START_Y + DY*(self.index + 1) as f64);
		sim.apply(self.id, effector);
	}
}

struct ReceiverDevice
{
	id: ComponentID,
	receiver: ReceiverComponent,
	mangler: ManglerComponent,

	inbound: InPort<String>,
}

impl ReceiverDevice
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID, error_rate: u32) -> ReceiverDevice
	{
		let id = sim.add_component("receiver", parent_id);
		let mut device = ReceiverDevice {
			id: id,
			receiver: ReceiverComponent::new(sim, id),
			mangler: ManglerComponent::new(sim, id, error_rate),
			inbound: InPort::empty(),
		};
		device.inbound = device.mangler.input.clone();
		device
	}
	
	pub fn start(mut self, sim: &mut Simulation, num_repeaters: i32)
	{
		self.mangler.upper_out.connect_to(&self.receiver.lower_in);
		
		self.receiver.start();
		self.mangler.start();
		
		let mut effector = Effector::new();
		effector.set_string("display-name", "receiver-0");
		effector.set_string("display-color", "green");
		effector.set_float("display-location-x", START_X);
		effector.set_float("display-location-y", START_Y + DY*(num_repeaters + 1) as f64);
		sim.apply(self.id, effector);
	}
}

// These are the components that are nested within devices.
struct SenderComponent
{
	id: ComponentID,
	data: ThreadData,
	output: OutPort<String>,
}

impl SenderComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> SenderComponent
	{
		// Active components have a thread that wakes up when an Event is sent to them.
		let (id, data) = sim.add_active_component("sender", parent_id);
		SenderComponent {
			id: id,
			data: data,
			output: OutPort::new(),
		}
	}
	
	pub fn start(self)
	{
		thread::spawn(move || {
			// data is ThreadData and contains the component's id, mpsc channels to communicate
			// with the Simulator, and a random number seed specific to the component.
			//
			// event is the Event dispatched to the component. It contains the name of the event,
			// an optional InPort name, and an optional arbitrary payload.
			//
			// state is a SimState and contains a read-only snapshot of the simulator state:
			// namely components and the store.
			//
			// effector is an Effector. process_events creates a new one each time an event is
			// delivered. It's used to capture side effects so that they can be applied after all
			// the events scheduled for the current time have had a chance to run.
			process_events!(self.data, event, state, effector,
				// "init N" events are scheduled by the simulation. All other events are scheduled
				// by component threads. Components may send an event directly to a component or
				// more typically to one of their OutPorts.
				"init 0" => {
					log_info!(effector, "init");
				
					let event = Event::new("timer");
					effector.schedule_immediately(event, self.id);
				},
				"timer" => {
					// This is where the action begins: the sender sends a poem to a
					// repeater, which sends it to another repeater, and so on until
					// the last repeater sends it to the receiver.
					self.output.send_payload(&mut effector, "text", POEM.to_string());
	
					let event = Event::new("timer");
					effector.schedule_after_secs(event, self.id, 1.0);
				}
			);
		});
	}
}

struct ManglerComponent
{
	data: ThreadData,
	error_rate: u32,

	input: InPort<String>,
	output: OutPort<String>,
	
	upper_in: InPort<String>,
	upper_out: OutPort<String>,
}

impl ManglerComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID, error_rate: u32) -> ManglerComponent
	{
		let (id, data) = sim.add_active_component("mangler", parent_id);
		ManglerComponent {
			data: data,
			error_rate: error_rate,

			input: InPort::with_port_name(id, "input"),
			output: OutPort::new(),

			upper_in: InPort::with_port_name(id, "upper_in"),
			upper_out: OutPort::new(),
		}
	}
	
	pub fn start(self)
	{
		// Note that it is important that components use the seed given to them by the simulation.
		// If they use other sources of randomness then simulations won't be deterministic which
		// makes bugs much harder to reproduce.
		let mut rng = StdRng::from_seed(&[self.data.seed]);
		
		thread::spawn(move || {
			process_events!(self.data, event, state, effector,
				"init 0" => {
				},
				"text" => {
					let old = event.payload_ref::<String>("text should have a String payload");
					if event.port_name == "upper_in" {
						let new = if self.upper_out.is_connected() {
							old.to_string()						// we're on the downward path of repeater
						} else {
							self.mangle(&mut rng, old)			// we're on the sender
						};
						self.output.send_payload(&mut effector, "text", new);
					} else {
						let new = self.mangle(&mut rng, old);	// we're on the inbound path of a repeater
						self.upper_out.send_payload(&mut effector, "text", new);
					}
				}
			);
		});
	}
	
	fn mangle(&self, rng: &mut StdRng, old: &str) -> String
	{
		let mut new = "".to_string();
		for ch in old.chars() {
			if rng.gen_weighted_bool(self.error_rate) {
				new.push('-');
			} else {
				new.push(ch);
			}
		}
		new
	}
}

struct StatsComponent
{
	data: ThreadData,
	err_percent: FloatValue,

	upper_in: InPort<String>,
	upper_out: OutPort<String>,

	lower_in: InPort<String>,
	lower_out: OutPort<String>,
}

impl StatsComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> StatsComponent
	{
		let (id, data) = sim.add_active_component("stats", parent_id);
		StatsComponent {
			data: data,
			err_percent: FloatValue{},

			upper_in: InPort::new(id),
			upper_out: OutPort::new(),

			lower_in: InPort::with_port_name(id, "lower_in"),
			lower_out: OutPort::new(),
		}
	}
	
	pub fn start(self)
	{
		thread::spawn(move || {
			process_events!(self.data, event, state, effector,
				"init 0" => {
				},
				"text" => {
					let text = event.payload_ref::<String>("text should have a String payload");
					if event.port_name == "lower_in" {
						let err = compute_error(text);
						log_debug!(effector, "{:.1}% error", err);
						set_value!(effector, self.err_percent = err);
						self.upper_out.send_payload(&mut effector, "text", text.to_string());
					} else {
						self.lower_out.send_payload(&mut effector, "text", text.to_string());
					}
				}
			);
		});
	}
}

struct RepeaterComponent
{
	data: ThreadData,
	lower_in: InPort<String>,
	lower_out: OutPort<String>,
}

impl RepeaterComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> RepeaterComponent
	{
		let (id, data) = sim.add_active_component("repeater", parent_id);
		RepeaterComponent {
			data: data,
			lower_in: InPort::new(id),
			lower_out: OutPort::new(),
		}
	}
	
	pub fn start(self)
	{
		thread::spawn(move || {
			process_events!(self.data, event, state, effector,
				"init 0" => {
				},
				"text" => {
					let text = event.take_payload();
					self.lower_out.send_payload(&mut effector, "text", text);
				}
			);
		});
	}
}

struct ReceiverComponent
{
	data: ThreadData,
	lower_in: InPort<String>,
}

impl ReceiverComponent
{
	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> ReceiverComponent
	{
		let (id, data) = sim.add_active_component("receiver", parent_id);
		ReceiverComponent {
			data: data,
			lower_in: InPort::new(id),
		}
	}
	
	pub fn start(self)
	{
		thread::spawn(move || {
			process_events!(self.data, event, state, effector,
				"init 0" => {
				},
				"text" => {
					let text = event.payload_ref::<String>("text should have a String payload");
					let err = compute_error(&text);
					log_info!(effector, "{:.1}% total error", err);
					log_excessive!(effector, "{}", text);
					if err > 99.0 {
						effector.exit();
					}
				}
			);
		});
	}
}

fn create_sim(local: LocalConfig, config: Config) -> Simulation
{
	// The components are setup very much like a computer network: there are devices
	// connected to one another and each device contains a stack of components with
	// messages traveling up and down the stack:
	//
	// sender   repeater0   repeater1  receiver
	//   |         ||         ||          |
	//   |       stats       stats        |
	//   |         ||         ||          |
	// mangler - mangler --- mangler --- mangler
	let mut sim = Simulation::new(config);
	let world_id = sim.add_component("world", NO_COMPONENT);

	// Create the devices,
	let mut sender = SenderDevice::new(&mut sim, world_id, local.error_rate);
	
	let mut repeaters = Vec::new();
	for i in 0..local.num_repeaters {
		let repeater = RepeaterDevice::new(&mut sim, world_id, local.error_rate, i);
		repeaters.push(repeater);
	}
	
	let receiver = ReceiverDevice::new(&mut sim, world_id, local.error_rate);

	{
	// wire them together,
	let mut last_port = &mut sender.outbound;
	for r in repeaters.iter_mut() {
		last_port.connect_to(&r.inbound);
		last_port = &mut r.outbound;
	}
	last_port.connect_to(&receiver.inbound);
	}
		
	// This is used by GUIs, e.g. sdebug.
	let mut effector = Effector::new();
	effector.set_float("display-size-x", DISPLAY_WIDTH);
	effector.set_float("display-size-y", DISPLAY_HEIGHT);
	effector.set_string("display-title", "telephone");
	sim.apply(world_id, effector);

	// and spin up their threads.
	sender.start(&mut sim);
	for r in repeaters.drain(..) {
		r.start(&mut sim);
	}
	receiver.start(&mut sim, local.num_repeaters);
	
	sim
}

fn fatal_err(message: &str) -> !
{
	eprintln!("{}", message);
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

fn parse_options() -> (LocalConfig, Config)
{
	let mut local = LocalConfig::new();
	let mut config = Config::new();
	
	// see https://docs.rs/clap/2.24.2/clap/struct.Arg.html#method.from_usage for syntax
	let usage = format!(
		"--address=[ADDR] 'Address for the web server to bind to [{default_address}]'
		--error=[N] 'Each step has a 1 in N chance of garbling a letter [{default_error}]'
		--home=[PATH] 'Start the web server and serve up PATH when / is hit'
		--log=[LEVEL:GLOB]... 'Overrides --log-level, glob is used to match component names'
		--log-level=[LEVEL] 'Default log level: {log_levels} [{default_level}]'
		--max-time=[TIME] 'Maximum time to run the simulation, use {time_suffixes} suffixes [no limit]'
		--no-colors 'Don't color code console output'
		--repeaters=[N] 'Number of steps between the sender and receiver [{default_repeaters}]'
		--seed=[N] 'Random number generator seed [random]'",
		default_address = config.address,
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
		config.seed = match_num(&matches, "seed", 1, usize::max_value());
	}
	
	if matches.is_present("address") {
		config.address = matches.value_of("address").unwrap().to_string();
	}
	
	if matches.is_present("home") {
		config.home_path = matches.value_of("home").unwrap().to_string();
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
	config.time_units = 10.0;	// tenths of seconds (1000 would be ms)
	
	let mut sim = create_sim(local, config);
	sim.run();
}
