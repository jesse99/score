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
	/// use rand::{Rng, SeedableRng, StdRng};
	/// use score::*;
	/// use std::thread;
	///
	/// fn component_thread(data: ThreadData)
	/// {
	/// 	let mut rng = StdRng::from_seed(&[data.seed]);
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
	pub seed: usize,	// TODO: document stuff to be careful of, eg HashMap iteration
}

impl ThreadData
{
	pub(crate) fn new(id: ComponentID, rx: mpsc::Receiver<(Event, SimState)>, tx: mpsc::Sender<Effector>, seed: usize) -> ThreadData
	{
		ThreadData{id, rx, tx, seed: seed}
	}
}