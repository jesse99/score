use component::*;
use effector::*;
use event::*;
use std::sync::mpsc;
use rand::Rng;

/// This is moved into each thread of an active `Component`.
pub struct ThreadData
{
	/// The ID of the `Component` bound to the thread instance.
	pub id: ComponentID,

	/// Threads receive from this in order to process `Event`s sent to them.
	pub rx: mpsc::Receiver<DispatchedEvent>,
	
	/// After a thread processes an `Event` the thread must respond by sending
	/// back a new `Effector` struct.
	pub tx: mpsc::Sender<Effector>,
	
	/// This should be the only source of randomness used within a `Component`s
	/// thread.
	pub rng: Box<Rng + Send>,	// TODO: document stuff to be careful of, eg HashMap iteration
}
