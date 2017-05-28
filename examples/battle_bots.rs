//! This example simulates a collection of battle bots with different behaviors, e.g.
//! some of the bots flee from other bots and some are aggressive and attempt to attack
//! other bots.
extern crate rsimbase;

use rsimbase::*;
use std::sync::mpsc;
use std::thread;

const NUM_BOTS: i32 = 4;	// TODO: make this a command line option

type ComponentThread = fn (ComponentID, mpsc::Receiver<DispatchedEvent>, mpsc::Sender<Effector>) -> ();

fn cowardly_thread(id: ComponentID, rx_event: mpsc::Receiver<DispatchedEvent>,
	tx_reply: mpsc::Sender<Effector>)
{
	thread::spawn(move || {
		for dispatched in rx_event {
			let mut effector = Effector::new();
			let ename = dispatched.event.name;
			match &*ename
			{
				"init 0" => {
					effector.log(LogLevel::Info, "initializing");
				},
				_ => {
					let cname = &(*dispatched.components).get(id).name;
					panic!("component {} can't handle event {}", cname, ename);
				}
			}
			
			let _ = tx_reply.send(effector);
		}
	});
}

// TODO: pick a random bot
fn new_random_bot(index: i32) -> (String, ComponentThread)
{
	let name = format!("bot {} (cowardly)", index);
	(name, cowardly_thread)
}

// TODO: take a seed option on the command line, if missing use a random seed
fn main()
{
	let mut config = Config::new();
	config.time_units = 1000.0;	// ms
	config.colorize = false;	// TODO: use a command line option
	let mut sim = Simulation::new(config);
	
	let world = sim.add_component("world", NO_COMPONENT);
	for i in 0..NUM_BOTS {
		let (name, thread) = new_random_bot(i);
		let top = sim.add_active_component(&name, world, locatable_thread);
		let _ = sim.add_active_component("AI", top, thread);
	}
	sim.run();
}