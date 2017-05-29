//! This example simulates a collection of battle bots with different behaviors, e.g.
//! some of the bots flee from other bots and some are aggressive and attempt to attack
//! other bots.
#[macro_use]
extern crate clap;
extern crate rand;
extern crate rsimbase;

use clap::{App, ArgMatches};
use rand::Rng;
use rsimbase::*;
use std::fmt::Display;
use std::io::{Write, stderr};
use std::process;
use std::str::FromStr;
use std::thread;

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
		// These are the defaults: all of these can be overriden using command line options.
		LocalConfig {
			num_bots: 4,
			width: 100.0,
			height: 100.0,
		}
	}
}

type ComponentThread = fn (LocalConfig, ThreadData) -> ();

fn randomize_location(local: &LocalConfig, top: ComponentID, effector: &mut Effector)
{
	let mut rng = rand::thread_rng();	// TODO: make sure these are using the same seed
	let payload = (rng.gen::<f64>()*local.width, rng.gen::<f64>()*local.height);
	let event = Event::new_with_payload("set-location", payload);
	effector.schedule_immediately(event, top);
}

fn cowardly_thread(local: LocalConfig, data: ThreadData)
{
	thread::spawn(move || {
		for dispatched in data.rx {
			let mut effector = Effector::new();
			let ename = &dispatched.event.name;
			if ename == "init 0" {
				effector.log(LogLevel::Info, "initializing");
				let top = dispatched.components.find_top_id(data.id);
				randomize_location(&local, top, &mut effector);
			} else {
				let cname = &(*dispatched.components).get(data.id).name;
				panic!("component {} can't handle event {}", cname, ename);
			}
			
			let _ = data.tx.send(effector);
		}
	});
}

// TODO: pick a random bot
fn new_random_bot(index: i32) -> (String, ComponentThread)
{
	let name = format!("cowardly-{}", index);
	(name, cowardly_thread)
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

fn parse_options() -> (LocalConfig, Config)
{
	let mut local = LocalConfig::new();
	let mut config = Config::new();
	
	// see https://docs.rs/clap/2.24.2/clap/struct.Arg.html#method.from_usage for syntax
	let usage = format!(
		"--height=[N] 'Max number of times bots can move up without wrapping [{default_height}]'
		--max-secs=[TIME] 'Maximum time to run the simulation, use s, m, or h suffixes [no limit]'
		--no-colors 'Don't color code console output'
		--num-bots=[N] 'Number of bots to start out with [{default_bots}]'
		--seed=[N] 'Random number generator seed [random]'
		--width=[N] 'Max number of times bots can move right without wrapping [{default_width}]'",
		default_height = local.height, default_width = local.width, default_bots = local.num_bots);
	
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
	
	let mut max_secs = matches.value_of("max-secs").unwrap_or("").to_string();
	if !max_secs.is_empty() {
		let units = max_secs.pop().unwrap();
		let base = f64::from_str(&max_secs).unwrap_or_else(|_| fatal_err("--max-secs should have an f64 value followed by a suffix"));
		config.max_secs = match units {
			's' => base,
			'm' => 60.0*base,
			'h' => 60.0*60.0*base,
			'd' => 24.0*60.0*60.0*base,
			'w' => 7.0*24.0*60.0*60.0*base,
			_  => fatal_err("--max-secs should have an s, m, h, d, or w suffix")
		};
	}
	
	config.colorize = !matches.is_present("no-colors");
	
	(local, config)
}

fn main()
{
	let (local, mut config) = parse_options();
	config.time_units = 1000.0;	// ms
	
	let mut sim = Simulation::new(config);
	let world = sim.add_component("world", NO_COMPONENT);
	for i in 0..local.num_bots {
		let (name, thread) = new_random_bot(i);
		let top = sim.add_active_component(&name, world, locatable_thread);
		let _ = sim.add_active_component("AI", top, |data| thread(local.clone(), data));
	}
	sim.run();
}