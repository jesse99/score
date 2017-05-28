use component::*;
use event::*;
use store::*;
use time::*;
use std::collections::HashMap;
//use std;

pub enum LogLevel	// TODO: move this somewhere else?
{
	Error,
	Warning,
	Info,
	Debug,
	Excessive
}

/// Effectors are returned by `Component`s after they process a `DispatchedEvent`.
/// The effector encapsulates the state changes the component wishes to make.
pub struct Effector
{
	#[doc(hidden)]
	pub logs: Vec<LogRecord>,
	
	#[doc(hidden)]
	pub events: HashMap<ComponentID, (Event, f64)>,
	
	#[doc(hidden)]
	pub store: Store,
}

impl Effector
{
	pub fn new() -> Effector
	{
		Effector{logs: Vec::new(), events: HashMap::new(), store: Store::new()}
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
		// TODO: scheduling in 0s is a little delicate, might want to have a schedule_immediately that uses the smallest time delta
		assert!(to != NO_COMPONENT);
		assert!(secs > 0.0, "secs ({:.3}) is not positive", secs);	// negative secs are just bad, for zero secs use schedule_immediately

		self.events.insert(to, (event, secs));
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

