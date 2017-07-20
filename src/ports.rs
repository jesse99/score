//! Ports are simple wrappers around [`Event`] sending. They don't do very much but they assist
//! in creating type safe [`Component`] structs. See the [`connect`] macro for an example.
use component::*;
use effector::*;
use event::*;
use std::any::Any;
use std::marker::PhantomData;

/// Use the connect! macro to bind this to an InPort.
pub struct OutPort<T: Any + Send>
{
	/// The ID of the component the InPort is part of.
	pub remote_id: ComponentID,
	
	/// The field name of the InPort, e.g. an ethernet switch could use this
	/// to send the event back out all but the port a packet came in on. This
	/// is assigned to the port_name field of [`Event`].
	pub remote_port: String,
	
	// We only use the T parameter for type checking but the compiler will
	// whine at us if we don't use it somewhere so we include this zero-sized
	// field.
	dummy: PhantomData<T>,
}

/// Use the connect! macro to bind this to an OutPort.
pub struct InPort<T: Any + Send>
{
	dummy: PhantomData<T>,
}

impl<T: Any + Send> InPort<T>
{
	pub fn new() -> InPort<T>
	{
		InPort {
			dummy: PhantomData,
		}
	}
}

impl<T: Any + Send> OutPort<T>
{
	pub fn new() -> OutPort<T>
	{
		OutPort {
			remote_id: NO_COMPONENT,
			remote_port: "".to_string(),
			dummy: PhantomData,
		}
	}

	/// Queue up an event to be processed ASAP.
	pub fn send_payload(&self, effector: &mut Effector, name: &str, payload: T)
	{
		let event = Event::with_port_payload(name, &self.remote_port, payload);
		effector.schedule_immediately(event, self.remote_id);
	}
	
	/// Queue up an event to be processed after secs time elapses.
	pub fn send_payload_after_secs(&self, effector: &mut Effector, name: &str, secs: f64, payload: T)
	{
		let event = Event::with_port_payload(name, &self.remote_port, payload);
		effector.schedule_after_secs(event, self.remote_id, secs);
	}

	/// This is normally called using the [`connect!`] macro.
	pub fn connect(&mut self, _in_port: &InPort<T>, in_port_name: &str, target: ComponentID)
	{
		self.remote_id = target;
		self.remote_port = in_port_name.to_string();
	}
}

impl OutPort<()>
{
	/// Queue up an event with no payload to be processed ASAP.
	pub fn send(&self, effector: &mut Effector, name: &str)
	{
		let event = Event::with_port(name, &self.remote_port);
		effector.schedule_immediately(event, self.remote_id);
	}
	
	/// Queue up an event with no payload to be processed after secs time elapses.
	pub fn send_after_secs(&self, effector: &mut Effector, name: &str, secs: f64)
	{
		let event = Event::with_port(name, &self.remote_port);
		effector.schedule_after_secs(event, self.remote_id, secs);
	}
}

/// Type safe way to send [`Event`]s.
///
/// # Examples
///
/// ```
/// use score::*;
///
/// struct Sender
/// {
/// 	output: OutPort<String>,
/// }
///
/// struct Receiver
/// {
/// 	input: InPort<String>,
/// }
///
/// fn wire_up(sender: &Sender, receiver: &Receiver)
/// {
/// 	connect!(sender.output -> receiver.input);
/// }
///
/// fn greeting(sender: &Sender, effector: &mut Effector)
/// {
/// 	sender.output.send_payload(effector, "text", "hello world".to_string());
/// }
/// ```
#[macro_export]
macro_rules! connect
{
	($out_instance:ident.$out_port_name:ident -> $in_instance:ident.$in_port_name:ident) => ({
		$out_instance.$out_port_name.connect(&$in_instance.$in_port_name, stringify!($in_port_name), $in_instance.id);
	});
}
