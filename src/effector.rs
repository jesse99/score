use component::*;
use event::*;
use logging::*;
use sim_time::*;
use store::*;
use std::collections::HashMap;
use std::f64::EPSILON;

/// Effectors are returned by `Component`s after they process an `Event`.
/// The effector encapsulates the state changes the component wishes to make.
pub struct Effector
{
	#[doc(hidden)]
	pub logs: Vec<LogRecord>,
	
	#[doc(hidden)]
	pub events: HashMap<ComponentID, (Event, f64)>,
	
	#[doc(hidden)]
	pub store: Store,
	
	#[doc(hidden)]
	pub exit: bool,
	
	#[doc(hidden)]
	pub removed: bool,
}

// It'd be nice to wrap this up in a smart pointer so that we could do the send
// when the pointer is dropped. But we can't move fields out of a struct in a
// drop method (see E0509) so we'd have to do a copy which could be expensive.
impl Effector
{
	pub fn new() -> Effector
	{
		Effector{logs: Vec::new(), events: HashMap::new(), store: Store::new(), exit: false, removed: false}
	}
	
	/// Normally you'll use one of the log macros, e.g. log_info!.
	pub fn log(&mut self, level: LogLevel, message: &str)
	{
		self.logs.push(LogRecord{level, message: message.to_string()});
	}
	
	/// Dispatch an event to a component after secs time elapses.
	pub fn schedule_after_secs(&mut self, event: Event, to: ComponentID, secs: f64)
	{
		// TODO: might want two versions: one that takes an absolute time and another that takes a relative time
		assert!(to != NO_COMPONENT);
		assert!(secs > 0.0, "secs ({:.3}) is not positive", secs);	// negative secs are just bad, for zero secs use schedule_immediately

		self.events.insert(to, (event, secs));
	}
	
	/// Events should not be scheduled for zero time because the `Simulation` guarantees
	/// that state is updated all at once at each time step. So if you want to schedule
	/// an event for as soon as possible use this method.
	pub fn schedule_immediately(&mut self, event: Event, to: ComponentID)
	{
		assert!(to != NO_COMPONENT);

		self.events.insert(to, (event, EPSILON));
	}
	
	/// Exit the sim after all events at the current time have been processed.
	pub fn exit(&mut self)
	{
		self.exit = true;
	}
	
	/// This will swap in a Component thread that drops all events and add a removed=1
	/// data entry to the store (so GUIs can stop rendering the component).
	pub fn remove(&mut self)
	{
		self.removed = true;
	}
	
	/// Use these methods to write out new values for data associated with the component.
	/// Note that when the data is written to the main store the name will be appended
	/// onto the component's path.
	pub fn set_int_data(&mut self, name: &str, value: i64)
	{
		assert!(!name.is_empty(), "name should not be empty");
		self.store.set_int_data(name, value, Time(0));
	}
	
	pub fn set_float_data(&mut self, name: &str, value: f64)
	{
		assert!(!name.is_empty(), "name should not be empty");
		self.store.set_float_data(name, value, Time(0));
	}
		
	pub fn set_string_data(&mut self, name: &str, value: &str)
	{
		assert!(!name.is_empty(), "name should not be empty");
		self.store.set_string_data(name, value, Time(0));
	}
}

#[doc(hidden)]
pub struct LogRecord
{
	#[doc(hidden)]
	pub level: LogLevel,

	#[doc(hidden)]
	pub message: String,
}

