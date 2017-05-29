use component::*;
use components::*;
use config::*;
use effector::*;
use event::*;
use store::*;
use thread_data::*;
use time::*;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::f64::EPSILON;
use std::sync::Arc;
use std::sync::mpsc;

/// This is the top-level data structure. Once an exe initializes
/// it the simulation will run until either a time limit elapses
/// or there are no events left to process.
pub struct Simulation
{
	store: Arc<Store>,
	components: Arc<Components>,	// all of these are indexed by ComponentID
	event_senders: Vec<Option<mpsc::Sender<DispatchedEvent>>>,
	effector_receivers: Vec<Option<mpsc::Receiver<Effector>>>,
	config: Config,
	precision: usize,	// number of decimal places to include when logging, derived from config.time_units
	current_time: Time,
	scheduled: BinaryHeap<ScheduledEvent>,
}
	
impl Simulation
{
	pub fn new(config: Config) -> Simulation
	{
		assert!(config.time_units > 0.0, "time units ({}) are not positive", config.time_units);
		assert!(config.num_init_stages > 0, "num_init_stages ({}) is not positive", config.num_init_stages);	// need an init step to schedule at least one event to process
		
		let precision = config.time_units.log10().max(0.0) as usize;
		Simulation {
			store: Arc::new(Store::new()),
			components: Arc::new(Components::new()),
			event_senders: Vec::new(),
			effector_receivers: Vec::new(),
			config: config,
			precision,
			current_time: Time(0),
			scheduled: BinaryHeap::new(),
		}
	}
	
	/// Adds a `Component` that is not intended to receive `Event`s.
	/// These can be used to organize related components together which
	/// can make navigation nicer within GUIs.
	pub fn add_component(&mut self, name: &str, parent: ComponentID) -> ComponentID
	{
		assert!(!name.is_empty(), "name should not be empty");
		assert!(parent != NO_COMPONENT || self.components.is_empty(), "can't have more than one root component");
		
		let id = ComponentID(self.event_senders.len());
		let component = Component{
			name: name.to_string(),
			parent: parent,
			children: Vec::new()};
		let components = Arc::get_mut(&mut self.components).unwrap();
		components.append(id, component, parent);
		self.event_senders.push(None);
		self.effector_receivers.push(None);
		id
	}
	
	/// Adds a component with a thread that can be sent a `DispatchedEvent`
	/// which processes the event and sends back an `Effector` .
	pub fn add_active_component<T>(&mut self, name: &str, parent: ComponentID, thread: T) -> ComponentID
		where T: FnOnce (ThreadData) -> ()
	{
		assert!(!name.is_empty(), "name should not be empty");
		assert!(parent != NO_COMPONENT || self.components.is_empty(), "can't have more than one root component");
		// when we support children properly assert that parent is not in children (recursively?)
		
		let (txd, rxd) = mpsc::channel::<DispatchedEvent>();
		let (txe, rxe) = mpsc::channel::<Effector>();

		let id = ComponentID(self.event_senders.len());
		let component = Component{
			name: name.to_string(),
			parent: parent,
			children: Vec::new()};
		let components = Arc::get_mut(&mut self.components).unwrap();
		components.append(id, component, parent);
		self.event_senders.push(Some(txd));
		self.effector_receivers.push(Some(rxe));
		
		thread(ThreadData{id, rx: rxd, tx: txe});
		id
	}
	
	/// Dispatches events until there are no more events left to dispatch
	/// or config.max_secs elapses.
	pub fn run(&mut self)
	{
		self.init_components();

		let mut i = 0;
		while !self.scheduled.is_empty() && i < 100 {	// TODO: use a config time limit instead
			self.current_time = self.dispatch_events();
			i += 1;
		}
	}
	
	pub fn init_components(&mut self)
	{
		for i in 0..self.config.num_init_stages {
			self.schedule_init_stage(i);
			let time = self.dispatch_events();
			assert!(time.0 == 0);
		}
	}
	
	fn dispatch_events(&mut self) -> Time
	{
		let time = self.scheduled.peek().unwrap().time;
		let mut ids = Vec::new();
		
		// TODO: track statistics on how parallel we are doing
		// TODO: should cap the number of threads we use (probably via config)
		while !self.scheduled.is_empty() && self.scheduled.peek().unwrap().time == time {	// while let can't have a guard so we use this somewhat ugly syntax
			let e = self.scheduled.pop().unwrap();
			self.log(&LogLevel::Excessive, "simulation", &format!("dispatching {} to id {}", &e.event.name, e.to.0));
			ids.push(e.to);
			
			if let Some(ref tx) = self.event_senders[e.to.0] {
				let d = DispatchedEvent{event: e.event, store: self.store.clone(), components: self.components.clone()};
				tx.send(d).unwrap();
			} else {
				let c = self.components.get(e.to);
				panic!("Attempt to send event {} to component {} which isn't an active component",
					e.event.name, c.name);
			}
		}
		
		let mut effects = HashMap::new();	// be sure to apply side effects after all events have finished processing
		for id in ids {
			if let Some(ref rx) = self.effector_receivers[id.0] {
				let e = rx.recv().expect(&format!("rx failed for id {}", id.0));	// TODO: use the timeout version and panic if it takes too long
				effects.insert(id, e);
			} else {
				assert!(false);
			}
		}
		
		for (id, e) in effects.iter_mut() {
			self.apply_logs(*id, e);
			self.apply_events(e);
			self.apply_stores(e);
		}
		time
	}
	
	fn schedule_init_stage(&mut self, stage: i32)
	{
		self.log(&LogLevel::Info, "simulation", &format!("initializing components at stage {}", stage));
		let name = format!("init {}", stage);
		for i in 0..self.event_senders.len() {
			if let Some(_) = self.event_senders[i] {
				let event = Event::new(&name);
				self.schedule(event, ComponentID(i), Time(0));
			}
		}
		assert!(!self.scheduled.is_empty());	// silly to have a simulation with no active components
	}
	
	fn schedule(&mut self, event: Event, to: ComponentID, time: Time)
	{
		self.scheduled.push(ScheduledEvent{event, to, time});
	}

	fn apply_logs(&mut self, id: ComponentID, effects: &Effector)
	{
		// TODO: also need to persist these
		let path = self.components.path(id);
		for record in effects.logs.iter() {
			self.log(&record.level, &path, &record.message);
		}
	}

	fn apply_events(&mut self, effects: &mut Effector)
	{
		for (to, (event, secs)) in effects.events.drain().take(1) {
			let time = self.add_secs(secs);
			self.schedule(event, to, time);
		}
	}

	fn apply_stores(&mut self, effects: &Effector)
	{
		// TODO: when saving these off need to turn the keys into paths
		assert!(effects.store.int_data.is_empty(), "event storing isn't implemented yet");
		assert!(effects.store.float_data.is_empty(), "event storing isn't implemented yet");
		assert!(effects.store.string_data.is_empty(), "event storing isn't implemented yet");
	}

	fn log(&mut self, level: &LogLevel, path: &str, message: &str)
	{
		let t = (self.current_time.0 as f64)/self.config.time_units;
		let t = format!("{:.*}", self.precision, t);
		
		if self.config.colorize {
			match level {
				&LogLevel::Error		=> log_to_console(&t, path, message, &self.config.error_escape_code, end_escape()),
				&LogLevel::Warning	=> log_to_console(&t, path, message, &self.config.warning_escape_code, end_escape()),
				&LogLevel::Info		=> log_to_console(&t, path, message, &self.config.info_escape_code, end_escape()),
				&LogLevel::Debug		=> log_to_console(&t, path, message, &self.config.debug_escape_code, end_escape()),
				&LogLevel::Excessive	=> log_to_console(&t, path, message, &self.config.excessive_escape_code, end_escape()),
			}
		} else {
			log_to_console(&t, path, message, "", "")
		}
	}
	
	fn add_secs(&self, secs: f64) -> Time
	{
		if secs == EPSILON {
			Time(self.current_time.0 + 1)
		} else {
			let delta = secs*self.config.time_units;
			Time(self.current_time.0 + (delta as i64))
		}
	}
}

struct ScheduledEvent
{
	time: Time,
	to: ComponentID,
	event: Event,
}

impl PartialEq for ScheduledEvent
{
	fn eq(&self, other: &ScheduledEvent) -> bool
	{
		self.time.0 == other.time.0
	}
}

impl Eq for ScheduledEvent {}

impl PartialOrd for ScheduledEvent
{
	fn partial_cmp(&self, other: &ScheduledEvent) -> Option<Ordering>
	{
		Some(self.cmp(other))
	}
}

impl Ord for ScheduledEvent
{
	fn cmp(&self, other: &ScheduledEvent) -> Ordering
	{
		other.time.0.cmp(&self.time.0)	// reversed because BinaryHeap returns the largest values first
	}
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
