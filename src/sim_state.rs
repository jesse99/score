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

	pub fn contains(&self, id: ComponentID, key: &str) -> bool
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.path(id), key);
		store.contains(&path)
	}

	pub fn get_int(&self, id: ComponentID, key: &str) -> i64
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.path(id), key);
		store.get_int(&path)
	}

	pub fn get_float(&self, id: ComponentID, key: &str) -> f64
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.path(id), key);
		store.get_float(&path)
	}

	pub fn get_string(&self, id: ComponentID, key: &str) -> String
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.path(id), key);
		store.get_string(&path)
	}
}
