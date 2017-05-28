use components::*;
use store::*;
use std::any::Any;
use std::sync::Arc;

/// Events are scheduled to be sent to a `Component` at a particular `Time`.
/// Components process the event using a thread and send an `Effector` back
/// to the `Simulation` which encapsulates the state changes they wish to
/// make.
pub struct Event
{
	/// Typically components may process different types of events so this
	/// is what they check to decide what they need to do.
	pub name: String,
	
	/// Arbitrary extra information associated with the event.
	pub payload: Option<Box<Any + Send>>,
}

impl Event
{
	pub fn new(name: &str) -> Event
	{
		assert!(!name.is_empty(), "name should not be empty");
		Event{name: name.to_string(), payload: None}
	}

	pub fn new_with_payload<T: Any + Send>(name: &str, payload: T) -> Event
	{
		assert!(!name.is_empty(), "name should not be empty");
		Event{name: name.to_string(), payload: Some(Box::new(payload))}
	}
}

/// This is what is actually sent to a `Component` when the `Simulation`
/// processes an `Event`.
pub struct DispatchedEvent
{
	/// The event that was scheduled.
	pub event: Event,
	
	/// The components in the Simulation.
	pub components: Arc<Components>,
	
	/// The state of the `Store` at the `Time` the event was dispatched:
	/// changes to the simulation happen after all events at time T have
	/// finished processing.
	pub store: Arc<Store>,
}
