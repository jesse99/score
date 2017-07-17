use component::*;
use components::*;
use store::*;
use std::borrow::Borrow;
use std::sync::Arc;

/// This is sent together with an `Event` when the `Simulation` dispatches
/// an event to a `Component` thread.
pub struct SimState
{
	/// The components in the Simulation.
	pub components: Arc<Components>,
	
	/// The state of the `Store` at the `Time` the event was dispatched:
	/// changes to the simulation happen after all events at time T have
	/// finished processing.
	pub store: Arc<Store>,
}

impl SimState
{
	pub fn was_removed(&self, id: ComponentID) -> bool
	{
		let store:&Store = self.store.borrow();
		let key = self.components.path(id) + ".removed";
		store.contains(&key)
	}
}