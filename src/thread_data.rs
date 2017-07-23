use component::*;
use effector::*;
use event::*;
use sim_state::*;
use std::sync::mpsc;

/// This is moved into each thread of an active `Component`.
pub struct ThreadData
{
	/// The ID of the `Component` bound to the thread instance.
	pub id: ComponentID,

	/// Threads receive from this in order to process `Event`s sent to them.
	/// Normally called via the process_events! macro.
	pub rx: mpsc::Receiver<(Event, SimState)>,
	
	/// Threads use this to send their side effects back to the simulation using
	/// an [`Effector`]. Normally called via the process_events! macro.
	pub tx: mpsc::Sender<Effector>,
	
	/// In order to have deterministic simuluations randomness has to be carefully
	/// controlled. Each component thread is given its own random number generator
	/// seed which should be the only source of randomness used by the thread.
	///
	/// # Examples
	///
	/// ```
	/// #[macro_use]
	/// extern crate score;
	/// extern crate rand;
	///
	/// use rand::{Rng, SeedableRng, XorShiftRng};
	/// use score::*;
	/// use std::thread;
	///
	/// fn component_thread(data: ThreadData)
	/// {
	/// 	let mut rng = XorShiftRng::from_seed([data.seed; 4]);
	/// 	thread::spawn(move || {
	/// 		process_events!(data, event, state, effector,
	/// 			"init 0" => {
	/// 				if rng.gen::<bool>() {
	/// 					log_info!(effector, "heads");
	/// 				} else {
	/// 					log_info!(effector, "tails");
	/// 				}
	/// 			}
	/// 		);
	/// 	});
	/// }
	/// # fn main() {
	/// # }
	/// ```
	pub seed: u32,	// TODO: document stuff to be careful of, eg HashMap iteration
}

impl ThreadData
{
	pub(crate) fn new(id: ComponentID, rx: mpsc::Receiver<(Event, SimState)>, tx: mpsc::Sender<Effector>, seed: u32) -> ThreadData
	{
		ThreadData{id, rx, tx, seed: seed}
	}
}