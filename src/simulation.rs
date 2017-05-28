use component::*;
use components::*;
use effector::*;
use event::*;
use time::*;
use std::sync::Arc;
use std::sync::mpsc;

/// This is the top-level data structure. Once an exe initializes
/// it the simulation will run until either a time limit elapses
/// or there are no events left to process.
pub struct Simulation
{
	components: Arc<Components>,	// all of these are indexed by ComponentID
	event_senders: Vec<Option<mpsc::Sender<DispatchedEvent>>>,
	effector_receivers: Vec<Option<mpsc::Receiver<Effector>>>,
	time_units: f64,
	precision: usize,
	time: Time,
}
	
impl Simulation
{
	/// Creates a simulation using micro-second resolution.
	pub fn new() -> Simulation
	{
		Simulation::new_with_time_units(1_000_000.0)
	}
	
	/// Creates a simulation using an arbitrary time resolution.
	/// Use 1_000.0 for ms, 1.0 for seconds, 0.1667 for minutes, etc.
	/// Note that larger time units may allow for additional parallelism.
	pub fn new_with_time_units(units: f64) -> Simulation
	{
		assert!(units > 0.0, "time units ({}) are not positive", units);
		
		Simulation {
			components: Arc::new(Components::new()),
			event_senders: Vec::new(),
			effector_receivers: Vec::new(),
			time_units: units,
			precision: units.log10().max(0.0) as usize,
			time: Time(0),
		}
	}
	
	/// Adds a `Component` that is not intended to receive `Event`s.
	/// These can be used to organize related components together which
	/// can make navigation nicer within GUIs.
	pub fn add_component(&mut self, name: &str, parent: ComponentID) -> ComponentID
	{
		assert!(!name.is_empty(), "name should not be empty");
		assert!(parent != NO_COMPONENT || self.components.is_empty(), "can't have more than one root component");
		
		let component = Component{
			name: name.to_string(),
			parent: parent,
			children: Vec::new()};
		let c = Arc::get_mut(&mut self.components).unwrap();
		c.append(component);
		self.event_senders.push(None);
		self.effector_receivers.push(None);
		ComponentID(self.event_senders.len() - 1)
	}
	
	/// Adds a component with a thread that can be sent a `DispatchedEvent`
	/// which processes the event and sends back an `Effector`.
	pub fn add_active_component<T>(&mut self, name: &str, parent: ComponentID, thread: T) -> ComponentID
		where T: FnOnce (ComponentID, mpsc::Receiver<DispatchedEvent>, mpsc::Sender<Effector>) -> ()
	{
		assert!(!name.is_empty(), "name should not be empty");
		assert!(parent != NO_COMPONENT || self.components.is_empty(), "can't have more than one root component");
		
		let (txd, rxd) = mpsc::channel::<DispatchedEvent>();
		let (txe, rxe) = mpsc::channel::<Effector>();

		let id = ComponentID(self.event_senders.len());
		let component = Component{
			name: name.to_string(),
			parent: parent,
			children: Vec::new()};
		let c = Arc::get_mut(&mut self.components).unwrap();
		c.append(component);
		self.event_senders.push(Some(txd));
		self.effector_receivers.push(Some(rxe));
		id
	}
	
	/// Dispatches events until there are no more events left to dispatch
	/// or time elapses. TODO: elapses how?
	/// Stages is the number of times to send an init event to the active
	/// components, e.g. if stages is two then "init 0" and "init 1" events
	/// will be sent.
	pub fn run(&mut self, stages: i32)
	{
		assert!(stages > 0, "stages ({}) is not positive", stages);	// need an init step to schedule at least one event to process
		
		for i in 0..stages {
			self.init_components(i)
		}
	}
	
	fn init_components(&mut self, stage: i32)
	{
		self.log(LogLevel::Info, "simulation", &format!("init {}", stage));
//		let name = format!("init {}", stage);
//		for i in 0..self.event_senders.len() {
//			if let Some(sender) = self.event_senders[i] {
//			}
//		}
	}

	fn log(&mut self, level: LogLevel, path: &str, message: &str)
	{
		let t = (self.time.0 as f64)/self.time_units;
		let t = format!("{:.*}", self.precision, t);
		
		match level {
			LogLevel::Error		=> log_to_console(&t, path, message, &error_escape(), end_escape()),
			LogLevel::Warning	=> log_to_console(&t, path, message, &warning_escape(), end_escape()),
			LogLevel::Info		=> log_to_console(&t, path, message, &info_escape(), end_escape()),
			LogLevel::Debug		=> log_to_console(&t, path, message, &debug_escape(), end_escape()),
			LogLevel::Excessive	=> log_to_console(&t, path, message, &excessive_escape(), end_escape()),
		}
	}
	
//	fn add_secs(&self, secs: f64) -> Time
//	{
//		let delta = secs*self.time_units;
//		Time(self.time.0 + i64(delta))
//	}
}

// TODO: We'll need a logger to write to a file or something
// (the store doesn't seem like a great place because we need
// to record stuff with a fair amount of structure and because
// logging should not require a mutable reference).
fn log_to_console(time: &str, path: &str, message: &str, begin: &str, end: &str)
{
	print!("{}{}   {}   {}{}\n", begin, time, path, message, end);
}

// TODO: escapes should be some sort of config option
// See https://en.wikipedia.org/wiki/ANSI_escape_code#Colors and https://aweirdimagination.net/2015/02/21/256-color-terminals/
fn error_escape() -> String		// these could be static references but that probably won't fly once we make these config options
{
	"\x1b[31;1m".to_string()	// bright red
}

fn warning_escape() -> String
{
	"\x1b[31m".to_string()		// red
}

fn info_escape() -> String
{
	"\x1b[30;1m".to_string()	// bold black
}

fn debug_escape() -> String
{
	"".to_string()				// black
}

fn excessive_escape() -> String
{
	"\x1b[1;38;5;244m".to_string()	// light gray
}

fn end_escape() -> &'static str
{
	"\x1b[0m"
}
