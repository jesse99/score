use component::*;
use components::*;
use config::*;
use effector::*;
use event::*;
use glob;
use logging::*;
use rand::{Rng, SeedableRng, XorShiftRng};
use rouille;
use rustc_serialize;
use sim_state::*;
use sim_time::*;
use store::*;
use thread_data::*;
use std::cmp::{max, min, Ordering};
use std::collections::BinaryHeap;
use std::collections::VecDeque;
use std::io;
use std::fs::File;
use std::path::Path;
use std::process;
use std::sync::Arc;
use std::sync::{mpsc, Mutex};
use std::thread;
use time;

/// This is the top-level data structure. Once an exe initializes
/// it the simulation will run until either a time limit elapses
/// or there are no events left to process.
pub struct Simulation
{
	pub store: Arc<Store>,				// TODO: can we make this private?
	pub components: Arc<Components>,	// Components and vectors are indexed by ComponentID
	event_senders: Vec<Option<mpsc::Sender<(Event, SimState)>>>,
	effector_receivers: Vec<Option<mpsc::Receiver<Effector>>>,
	config: Config,
	precision: usize,	// number of decimal places to include when logging, derived from config.time_units
	current_time: Time,
	scheduled: BinaryHeap<ScheduledEvent>,
	rng: Box<Rng + Send>,
	max_path_len: usize,
	start_time: time::Timespec,
	event_num: u64,
	finger_print: u64,

	// These are used when the REST server is running.
	log_lines: Vec<LogLine>,
}
	
impl Simulation
{
	pub fn new(config: Config) -> Simulation
	{
		assert!(config.time_units > 0.0, "time units ({}) are not positive", config.time_units);
		assert!(config.num_init_stages > 0, "num_init_stages ({}) is not positive", config.num_init_stages);	// need an init step to schedule at least one event to process
				
		let precision = config.time_units.log10().max(0.0) as usize;
		let seed = config.seed;
		Simulation {
			store: Arc::new(Store::new()),
			components: Arc::new(Components::new()),
			event_senders: Vec::new(),
			effector_receivers: Vec::new(),
			config: config,
			precision,
			current_time: Time(0),
			scheduled: BinaryHeap::new(),
			rng: Box::new(new_rng(seed, 10_000)),
			max_path_len: 0,
			start_time: time::get_time(),
			event_num: 0,
			finger_print: 0,
			
			log_lines: Vec::new(),
		}
	}
	
	/// Dump simulation state to stdout.
	pub fn print(&self)
	{
		println!("Components:");
		self.components.print();

		println!("Store:");
		self.store.print(self.config.time_units, self.precision);

		let t = (self.current_time.0 as f64)/self.config.time_units;
		println!("Current Time:");
		println!("   {:.1$}s", t, self.precision);

		println!("Scheduled:");
		for s in self.scheduled.iter() {
			let t = (s.time.0 as f64)/self.config.time_units;
			let path = self.components.path(s.to);
			println!("   {:.1$}s {2} -> {3}", t, self.precision, s.event.name, path);
		}
	}
	
	/// Adds a `Component` that is not intended to receive `Event`s.
	/// These can be used to organize related components together which
	/// can make navigation nicer within GUIs.
	pub fn add_component(&mut self, name: &str, parent: ComponentID) -> ComponentID
	{
		assert!(!name.is_empty(), "name should not be empty");
		assert!(parent != NO_COMPONENT || self.components.is_empty(), "can't have more than one root component");
		assert!(name.chars().nth(0).unwrap().is_alphabetic());
		assert!(name.chars().all(is_valid_name_char));
		
		let id = ComponentID(self.event_senders.len());
		{
		let component = Component{
			name: name.to_string(),
			parent: parent,
			children: Vec::new()};
		let components = Arc::get_mut(&mut self.components).unwrap();
		components.append(id, component, parent);
		}
		let path = self.components.path(id);
		self.max_path_len = max(path.len(), self.max_path_len);
		self.event_senders.push(None);
		self.effector_receivers.push(None);
		id
	}
	
	/// Adds a component that is expected to spin up a thread taking [`ThreadData`].
	pub fn add_active_component(&mut self, name: &str, parent: ComponentID) -> (ComponentID, ThreadData)
	{
		// TODO: probably should have an usage example
		assert!(!name.is_empty(), "name should not be empty");
		assert!(parent != NO_COMPONENT || self.components.is_empty(), "can't have more than one root component");
		assert!(name.chars().nth(0).unwrap().is_alphabetic());
		assert!(name.chars().all(is_valid_name_char));
		// TODO: when we support children properly assert that parent is not in children (recursively?)
		
		let (txd, rxd) = mpsc::channel::<(Event, SimState)>();
		let (txe, rxe) = mpsc::channel::<Effector>();

		let id = ComponentID(self.event_senders.len());
		{
		let component = Component{
			name: name.to_string(),
			parent: parent,
			children: Vec::new()};
		let components = Arc::get_mut(&mut self.components).unwrap();
		components.append(id, component, parent);
		}
		let path = self.components.path(id);
		self.max_path_len = max(path.len(), self.max_path_len);
		self.event_senders.push(Some(txd));
		self.effector_receivers.push(Some(rxe));
		
		let seed = get_seed(self.config.seed, id.0 as u32);
		(id, ThreadData::new(id, rxd, txe, seed))
	}
	
	/// Use this if you want to update the store, or log, or schedule events when
	/// initializing components. Often used to avoid spinning up a thread,
	pub fn apply(&mut self, id: ComponentID, mut effects: Effector)
	{
		assert!(!effects.exit);
		self.apply_effects(id, &mut effects);
	}
	
	/// Use this if you want to do something random when initializing components.
	pub fn rng(&mut self) -> &mut Box<Rng + Send>
	{
		// Using a Box was the only way I could figure out to use the Rng trait
		// while constraining Rng to be Sized as well so clients get the full
		// set of Rng methods. (We can't actually constrain a type to Sized but
		// we can to Send which is evidently enough).
		&mut self.rng
	}
	
	/// Dispatches events until there are no more events left to dispatch,
	/// config.max_secs elapses, or `Effector`s exit method was called.
	pub fn run(&mut self)
	{
		if self.config.root.is_empty() {
			self.run_normally();
		} else {
			if Path::new(&self.config.root).is_file() {
				self.run_server();
			} else {
				eprintln!("'{}' is not a file", self.config.root);
				process::exit(1);
			}
		}
	}
	
	// ---- Private Functions ----------------------------------------------------------------
	fn run_normally(&mut self)
	{
		let mut exiting = self.init_components();
		while exiting.is_empty() {
			exiting = self.run_time_slice()
		}
		
//		self.print();
		self.exit(exiting);
	}
	
	fn run_server(&mut self)
	{
		let address = self.config.address.clone();
		self.log(LogLevel::Info, NO_COMPONENT, &format!("running web server at {}", address));

		let (tx_command, rx_command) = mpsc::channel();
		let (tx_reply, rx_reply) = mpsc::channel();
		spin_up_rest(&self.config.address, &self.config.root, tx_command, rx_reply);

		let mut exiting = self.init_components();
		for command in rx_command.iter() {
			let reply = match command {
				RestCommand::GetLogAfter(time) => {
					let lines = self.get_log_lines(time);
					let data = rustc_serialize::json::encode(&lines).unwrap();	
					RestReply{data, code:200}
				},
				RestCommand::GetState(path) => {
					let lines = self.get_state(&path);
					let data = rustc_serialize::json::encode(&lines).unwrap();
					RestReply{data, code:200}
				},
				RestCommand::GetTime => {
					let t = (self.current_time.0 as f64)/self.config.time_units;
					let data = rustc_serialize::json::encode(&t).unwrap();
					RestReply{data, code:200}
				},
				RestCommand::GetTimePrecision => {
					let data = rustc_serialize::json::encode(&self.precision).unwrap();
					RestReply{data, code:200}
				},
				RestCommand::RunUntilLogChanged => {
					let num_lines = self.log_lines.len();
					while exiting.is_empty() && self.log_lines.len() == num_lines {
						exiting = self.run_time_slice()
					}
					
					let message = if exiting.is_empty() {
						"ok"
					} else {
						"exited"
					};
					let data = rustc_serialize::json::encode(&message.to_string()).unwrap();
					RestReply{data, code:200}
				}
				RestCommand::SetFloatState(path, value) => {
					let store = Arc::get_mut(&mut self.store).expect("Has a component retained a reference to the store?");
					store.set_float(&path, value, self.current_time);
					let data = "\"ok\"".to_string();
					RestReply{data, code:200}
				}
				RestCommand::SetIntState(path, value) => {
					let store = Arc::get_mut(&mut self.store).expect("Has a component retained a reference to the store?");
					store.set_int(&path, value, self.current_time);
					let data = "\"ok\"".to_string();
					RestReply{data, code:200}
				}
				RestCommand::SetStringState(path, value) => {
					let store = Arc::get_mut(&mut self.store).expect("Has a component retained a reference to the store?");
					store.set_string(&path, &value, self.current_time);
					let data = "\"ok\"".to_string();
					RestReply{data, code:200}
				}
				RestCommand::SetTime(secs) => {
					let target = (secs*self.config.time_units) as i64;
					while exiting.is_empty() && self.current_time.0 < target {
						exiting = self.run_time_slice()
					}
					
					let message = if exiting.is_empty() {
						"ok"
					} else {
						"exited"
					};
					let data = rustc_serialize::json::encode(&message.to_string()).unwrap();
					RestReply{data, code:200}
				}
			};
			tx_reply.send(reply).unwrap();
		}
		
		// Note that we don't want to exit in order to allow GUIs to inspect state at the end.
		// TODO: but we should have some sort of /exit endpoint to allow GUIs to kill us cleanly.
		//self.exit(exiting);
	}
	
	fn init_components(&mut self) -> &'static str
	{
		let mut exiting = "";
		for i in 0..self.config.num_init_stages {
			self.schedule_init_stage(i);
			let exit = self.dispatch_events();
			assert!(self.current_time.0 == 0);
			if exit {
				exiting = "Effector.exit was called during initialization";
			}
		}
		exiting
	}
	
	fn run_time_slice(&mut self) -> &'static str
	{
		let max_time = if self.config.max_secs.is_infinite() {i64::max_value()} else {(self.config.max_secs*self.config.time_units) as i64};
		if self.scheduled.is_empty() {
			"no events"
		
		} else if self.current_time.0 >= max_time {
			"reached config.max_secs"

		} else {
			if self.dispatch_events() {
				"effector.exit was called"
			} else {
				""
			}
		}
	}
	
	fn exit(&mut self, exiting: &str)
	{
		// TODO: Might want to also print events/sec, maybe at debug
		let elapsed = (time::get_time() - self.start_time).num_milliseconds();
		self.log(LogLevel::Debug, NO_COMPONENT, &format!("exiting sim, run time was {}.{}s ({})",
			elapsed/1000, elapsed%1000, exiting));	// TODO: eventually will need a friendly_duration_str fn
			
		let finger_print = self.finger_print.clone();
		self.log(LogLevel::Info, NO_COMPONENT, &format!("finger print = {:X}", finger_print));
	}
	
	fn dispatch_events(&mut self) -> bool
	{
		self.current_time = self.scheduled.peek().unwrap().time;
		let mut ids = Vec::new();
		
		// TODO: track statistics on how parallel we are doing
		// TODO: should cap the number of threads we use (probably via config)
		while !self.scheduled.is_empty() && self.scheduled.peek().unwrap().time == self.current_time {	// while let can't have a guard so we use this somewhat ugly syntax
			let e = self.scheduled.pop().unwrap();
			self.update_finger_print(&e);
			
			// TODO: If we use speculative execution we'll need to be careful not to do
			// anything wrong when REST is being used. Maybe just disable speculation.
			if self.should_log(LogLevel::Excessive, NO_COMPONENT) {
				let path = self.components.path(e.to);
				let num = self.event_num;
				self.log(LogLevel::Excessive, NO_COMPONENT, &format!("dispatching #{} '{}' to {}", num, e.event.name, path));
			}
			ids.push(e.to);
			
			self.event_num += 1;
			if let Some(ref tx) = self.event_senders[e.to.0] {
				let state = SimState{store: self.store.clone(), components: self.components.clone()};
				tx.send((e.event, state)).unwrap();
			} else {
				let c = self.components.get(e.to);
				panic!("Attempt to send event {} to component {} which isn't an active component",
					e.event.name, c.name);
			}
		}
		
		// Note that it is important that we collect all of the side effects for a time t
		// before we apply them. That way components executing at t do not affect each other.
		let mut effects = Vec::with_capacity(ids.len());
		for id in ids {
			if let Some(ref rx) = self.effector_receivers[id.0] {
				let e = rx.recv().expect(&format!("rx failed for id {}", id.0));	// TODO: use the timeout version and panic if it takes too long
				effects.push((id, e));
			} else {
				let c = self.components.get(id);
				panic!("Failed to receive an effector from component {}", c.name);
			}
		}
		
		// This isn't terribly important but does keep the log ordering at a time
		// consistent which is kind of nice.
		effects.sort_by(|a, b| a.0.cmp(&b.0));
		
		let mut exit = false;
		for (id, mut e) in effects.drain(..) {
			self.apply_effects(id, &mut e);
			
			if e.exit {
				exit = true;
			}
		}
		exit
	}
	
	fn apply_effects(&mut self, id: ComponentID, effects: &mut Effector)
	{
		self.apply_logs(id, &effects);
		self.apply_events(effects);
		self.apply_stores(&effects, id);

		if effects.removed {
			self.remove_components(id);
		}
	}
	
	// The finger print is used to verify that the simulation is deterministic: things like
	// the order of hash map iteration or random number generation (assuming the same seed)
	// should not change what happens during a simulation run. We could only compute the finger
	// print when told to but it should be quite cheap and non-determinism is annoying enough
	// that it's worth keeping an eye on.
	fn update_finger_print(&mut self, sevent: &ScheduledEvent)
	{
		let mut delta = sevent.time.0 as u64;
		delta += sevent.to.0 as u64;
		
		let name = &sevent.event.name;
		for b in name.bytes().take(min(name.len(), 8)) {
			delta += b as u64;
		}
		
		self.finger_print = self.finger_print.wrapping_add(delta);
	}
	
	fn remove_components(&mut self, id: ComponentID)
	{
		{
		self.install_removed_thread(id);
		
		let store = Arc::get_mut(&mut self.store).expect("Has a component retained a reference to the store?");
		let key = self.components.path(id) + ".removed";
		store.set_int(&key, 1, self.current_time);
		}
		
		let children = self.components.get(id).children.clone();
		for child_id in children.iter() {
			self.remove_components(*child_id);
		}
	}
	
	fn install_removed_thread(&mut self, id: ComponentID)
	{
		let (txd, rxd) = mpsc::channel::<(Event, SimState)>();
		let (txe, rxe) = mpsc::channel::<Effector>();
		
		self.event_senders[id.0] = Some(txd);
		self.effector_receivers[id.0] = Some(rxe);
		
		no_op_thread(rxd, txe);
	}
	
	fn schedule_init_stage(&mut self, stage: i32)
	{
		self.log(LogLevel::Info, NO_COMPONENT, &format!("initializing components at stage {}", stage));
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
//		let path = self.components.path(to);
//		let t = (time.0 as f64)/self.config.time_units;
//		self.log(LogLevel::Debug, NO_COMPONENT, &format!("scheduling {} for {} to {:.3}", event.name, path, t));
		
		self.scheduled.push(ScheduledEvent{event, to, time});
	}

	fn apply_logs(&mut self, id: ComponentID, effects: &Effector)
	{
		for record in effects.logs.iter() {
			self.log(record.level, id, &record.message);
		}
	}

	fn apply_events(&mut self, effects: &mut Effector)
	{
		for (to, event, secs) in effects.events.drain(..) {	// we drain because we want to move the event into our list of scheduled events
			let time = self.add_secs(secs);
//			let path = self.components.path(to);
//			self.log(LogLevel::Info, NO_COMPONENT, &format!("scheduling {} to {} at {:.3}", event.name, path, secs));
			self.schedule(event, to, time);
		}
	}

	fn apply_stores(&mut self, effects: &Effector, id: ComponentID)
	{
		let path = self.components.path(id);
		let store = Arc::get_mut(&mut self.store).expect("Has a component retained a reference to the store?");

		store.int_data.reserve(effects.store.int_data.len());
		for (key, value) in effects.store.int_data.iter() {
			let key = format!("{}.{}", path, key);
			store.set_int(&key, value.1, self.current_time);
		}
		
		store.float_data.reserve(effects.store.float_data.len());
		for (key, value) in effects.store.float_data.iter() {
			let key = format!("{}.{}", path, key);
			store.set_float(&key, value.1, self.current_time);
		}
		
		store.string_data.reserve(effects.store.string_data.len());
		for (key, value) in effects.store.string_data.iter() {
			let key = format!("{}.{}", path, key);
			store.set_string(&key, &value.1, self.current_time);
		}
	}

	fn log(&mut self, level: LogLevel, id: ComponentID, message: &str)
	{
		if self.should_log(level, id) {
			let t = (self.current_time.0 as f64)/self.config.time_units;
			
			let path = self.logged_path(id);
			if self.config.colorize {
				let begin_escape = match level {
					LogLevel::Error	=> &self.config.error_escape_code,
					LogLevel::Warning	=> &self.config.warning_escape_code,
					LogLevel::Info		=> &self.config.info_escape_code,
					LogLevel::Debug	=> &self.config.debug_escape_code,
					LogLevel::Excessive=> &self.config.excessive_escape_code,
				};
				print!("{0}{1:.2$}   {3} {4}{5}\n", begin_escape, t, self.precision, path, message, end_escape());
			} else {
				let prefix = match level {
					LogLevel::Error	=> "error",
					LogLevel::Warning	=> "warn ",
					LogLevel::Info		=> "info ",
					LogLevel::Debug	=> "debug",
					LogLevel::Excessive=> "exces",
				};
				print!("{0:.1$}  {2} {3}  {4}\n", t, self.precision, prefix, path, message);
			}
		}

		if !self.config.root.is_empty() {
			let time = (self.current_time.0 as f64)/self.config.time_units;
			let path = if id == NO_COMPONENT {"simulation".to_string()} else {self.components.path(id)};
//			let level = format!("{}", level);
			let message = message.to_string();
			let line = LogLine{time, path, level, message};
			self.log_lines.push(line);
		}
	}

	fn logged_path(&self, id: ComponentID) -> String
	{
		let mut path = if id == NO_COMPONENT {"simulation".to_string()} else {self.components.path(id)};
		if self.config.max_log_path > 0 && self.max_path_len > self.config.max_log_path {
			let len = path.len();
			if len > self.config.max_log_path {
				format!("…{}", path.split_off(len - self.config.max_log_path))
			} else {
				format!("{0:<1$}", path, self.config.max_log_path)
			}
		} else {
			format!("{0:<1$}", path, self.max_path_len)
		}
	}
	
	fn should_log(&self, level: LogLevel, id: ComponentID) -> bool
	{
		if !self.config.log_levels.is_empty() {	// short circuit some work if we have no overrides
			let name = if id == NO_COMPONENT {"simulation"} else {&self.components.get(id).name};
			
			for (pattern, clevel) in self.config.log_levels.iter() {
				if pattern.matches(name) {
					return level <= *clevel
				}
			}
		}

		level <= self.config.log_level
	}
	
	fn add_secs(&self, secs: f64) -> Time
	{
		assert!(secs >= 0.0);
		
		let delta = (secs*self.config.time_units) as i64;
		if delta > 0 {
			Time(self.current_time.0 + delta)
		} else {
			Time(self.current_time.0 + 1)	// event dispatch should always take a bit of time so that all the effects at a time can be applied all at once
		}
	}

	fn get_log_lines(&self, after_time: f64) -> VecDeque<&LogLine>
	{
		let mut result = VecDeque::new();
		
		for line in self.log_lines.iter().rev() {
			if line.time > after_time {
				result.push_front(line);
			}
		}
		
		result
	}

	fn get_state(&self, path: &glob::Pattern) -> Vec<String>
	{
		let mut removed = Vec::new();
		for (key, value) in self.store.int_data.iter() {
			if key.ends_with(".removed") && value.1 == 1 {
				let (prefix, _) = key.split_at(key.len() - "removed".len());
				removed.push(prefix);
			}
		}

		let mut result = Vec::new();
		for (key, value) in self.store.int_data.iter() {
			if path.matches(&key) && !removed.iter().any(|r| key.starts_with(r)) {
				let line = format!("{} = {}", key, value.1);
				result.push(line);
			}
		}
		
		for (key, value) in self.store.float_data.iter() {
			if path.matches(&key) && !removed.iter().any(|r| key.starts_with(r)) {
				let line = format!("{} = {}", key, value.1);
				result.push(line);
			}
		}
		
		for (key, value) in self.store.string_data.iter() {
			if path.matches(&key) && !removed.iter().any(|r| key.starts_with(r)) {
				let line = format!("{} = \"{}\"", key, value.1);
				result.push(line);
			}
		}
		
		result.sort();
		result
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

fn end_escape() -> &'static str
{
	"\x1b[0m"
}

fn get_seed(seed: u32, offset: u32) -> u32
{
	let seed = if seed != 0 {seed} else {time::get_time().nsec as u32};
	seed + offset	// offset is used to give each thread its own random stream
}

// We care about speed much more than we care about a cryptographic RNG so
// XorShiftRng should be plenty good enough.
fn new_rng(seed: u32, offset: u32) -> XorShiftRng
{
	let seed = get_seed(seed, offset);
	XorShiftRng::from_seed([seed; 4])
}

fn no_op_thread(rx: mpsc::Receiver<(Event, SimState)>, tx: mpsc::Sender<Effector>)
{
	thread::spawn(move || {
		for dispatched in rx {
			// We drop all events but we still need to tell the Simulator that we haven't actually done anything.
			drop(dispatched);
			let _ = tx.send(Effector::new());
		}
	});
}

enum RestCommand
{
	GetLogAfter(f64),
	GetState(glob::Pattern),
	GetTime,
	GetTimePrecision,
	RunUntilLogChanged,
	SetFloatState(String, f64),
	SetIntState(String, i64),
	SetStringState(String, String),
	SetTime(f64),
}

struct RestReply
{
	data: String,
	code: u16,
}

#[derive(RustcEncodable)]
struct LogLine
{
	time: f64,
	path: String,
	level: LogLevel,
	message: String,
}

fn file_response(request: &rouille::Request, path: &Path) -> rouille::Response
{
	match File::open(&path) {
		Ok(file) => rouille::Response::from_file("text/html; charset=utf8", file),
		Err(ref err) if err.kind() == io::ErrorKind::NotFound => {
			eprintln!("Didn't find file for {} {}", request.method(), request.url());
			rouille::Response::empty_404()
		},
		Err(ref err) => {
			let mesg: &str = &format!("{:?}", err.kind());
			eprintln!("Error reading file for {} {}: {}", request.method(), request.url(), mesg);
			rouille::Response::text(mesg).with_status_code(403)
		},
	}
}

// For debugging can do stuff like:
//    curl http://127.0.0.1:9000/log/all
//    curl -X POST http://127.0.0.1:9000/time/10
fn spin_up_rest(address: &str, root: &str, tx_command: mpsc::Sender<RestCommand>, rx_reply: mpsc::Receiver<RestReply>)
{
	let addr = address.to_string();
	let root = root.to_string();
	
	// rouille will spawn up a thread for each client that attaches and there's no good
	// way to clone the channels into them so we need to use a mutex to serialize access.
	let tx_command = Mutex::new(tx_command);
	let rx_reply = Mutex::new(rx_reply);

	thread::spawn(move|| {rouille::start_server(&addr, move |request| {
		let path = Path::new(&root);
		let root_dir = path.parent().unwrap();

//		println!("{} {}", request.method(), request.url());
		router!(request,
			(GET) (/) => {
				file_response(&request, path)
			},
			
			// In theory REST endpoints can conflict with file names within root_dir but none of
			// the REST endpoints have an extension so this shouldn't be a problem in practice.
			(GET) (/log/after/{time: f64}) => {
				handle_endpoint(RestCommand::GetLogAfter(time), &tx_command, &rx_reply)
			},
			
			(POST) (/run/until/log-changed) => {
				handle_endpoint(RestCommand::RunUntilLogChanged, &tx_command, &rx_reply)
			},
			
			// These really should be PUTs but crest doesn't support PUT...
			(POST) (/state/float/{path: String}/{value: f64}) => {
				handle_endpoint(RestCommand::SetFloatState(path, value), &tx_command, &rx_reply)
			},
			
			(POST) (/state/int/{path: String}/{value: i64}) => {
				handle_endpoint(RestCommand::SetIntState(path, value), &tx_command, &rx_reply)
			},
			
			(GET) (/state/{path: String}) => {
				if let Ok(path) = glob::Pattern::new(&path) {
					handle_endpoint(RestCommand::GetState(path), &tx_command, &rx_reply)
				} else {
					rouille::Response::empty_400()
				}
			},
			
			(POST) (/state/string/{path: String}/{value: String}) => {
				handle_endpoint(RestCommand::SetStringState(path, value), &tx_command, &rx_reply)
			},
			
			(GET) (/time) => {
				handle_endpoint(RestCommand::GetTime, &tx_command, &rx_reply)
			},
			
			(GET) (/time/precision) => {
				handle_endpoint(RestCommand::GetTimePrecision, &tx_command, &rx_reply)
			},
			
			(POST) (/time/{secs: f64}) => {
				handle_endpoint(RestCommand::SetTime(secs), &tx_command, &rx_reply)
			},
			
			_ => {
				let response = rouille::match_assets(&request, &root_dir);
				if !response.is_success() {
					eprintln!("Failed to read file for {} {}", request.method(), request.url());
				}
				response.with_no_cache()	// TODO: might want to do this just in debug (altho the client and server are normally both local so it shouldn't matter much)
			}
			)
		});
	});
}

fn handle_endpoint(command: RestCommand, tx_command: &Mutex<mpsc::Sender<RestCommand>>, rx_reply: &Mutex<mpsc::Receiver<RestReply>>) -> rouille::Response
{
	tx_command.lock().unwrap().send(command).unwrap();
	let reply = rx_reply.lock().unwrap().recv().unwrap();
	
	rouille::Response {
		status_code: reply.code,
		headers: vec![("Content-Type".into(), "application/json".into())],
		data: rouille::ResponseBody::from_data(reply.data),
		upgrade: None,
	}
}

fn is_valid_name_char(ch: char) -> bool
{
	!ch.is_whitespace() &&		// no spaces makes it much easier for sdebug to parse commands (paths don't need to be quoted)
	!ch.is_control() &&			// these are just silly to include in a name
	ch != '"' && ch != '\'' &&	// parsing is simpler if paths don't have quotes
	ch != '.'					// allowing periods in a name would cause a lot of confusion when looking at paths
}
