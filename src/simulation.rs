use component::*;
use components::*;
use config::*;
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
	config: Config,
	precision: usize,	// number of decimal places to include when logging, derived from config.time_units
	time: Time,
}

// config
// time units
// colorize
//    maybe escape sequences too
// num stages?
	
impl Simulation
{
	pub fn new(config: Config) -> Simulation
	{
		assert!(config.time_units > 0.0, "time units ({}) are not positive", config.time_units);
		assert!(config.num_init_stages > 0, "num_init_stages ({}) is not positive", config.num_init_stages);	// need an init step to schedule at least one event to process
		
		let precision = config.time_units.log10().max(0.0) as usize;
		Simulation {
			components: Arc::new(Components::new()),
			event_senders: Vec::new(),
			effector_receivers: Vec::new(),
			config: config,
			precision,
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
	/// or config.max_secs elapses.
	pub fn run(&mut self)
	{
		for i in 0..self.config.num_init_stages {
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
		let t = (self.time.0 as f64)/self.config.time_units;
		let t = format!("{:.*}", self.precision, t);
		
		if self.config.colorize {
			match level {
				LogLevel::Error		=> log_to_console(&t, path, message, &self.config.error_escape_code, end_escape()),
				LogLevel::Warning	=> log_to_console(&t, path, message, &self.config.warning_escape_code, end_escape()),
				LogLevel::Info		=> log_to_console(&t, path, message, &self.config.info_escape_code, end_escape()),
				LogLevel::Debug		=> log_to_console(&t, path, message, &self.config.debug_escape_code, end_escape()),
				LogLevel::Excessive	=> log_to_console(&t, path, message, &self.config.excessive_escape_code, end_escape()),
			}
		} else {
			log_to_console(&t, path, message, "", "")
		}
	}
	
//	fn add_secs(&self, secs: f64) -> Time
//	{
//		let delta = secs*self.config.time_units;
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

fn end_escape() -> &'static str
{
	"\x1b[0m"
}
