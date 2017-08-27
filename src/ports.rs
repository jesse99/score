// Copyright (C) 2017 Jesse Jones
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 3, or (at your option)
// any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software Foundation,
// Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.

//! Ports are simple wrappers around [`Event`] sending. They don't do very much but they assist
//! in creating type safe [`Component`] structs. See the [`connect`] macro for an example.
use component::*;
use effector::*;
use event::*;
use std::any::Any;
use std::marker::PhantomData;

/// OutPort's are connected to InPort's.
#[derive(Clone)]
pub struct OutPort<T: Any + Send>
{
	/// The ID of the component the InPort is part of.
	pub remote_id: ComponentID,
	
	/// Optionl name of the InPort, e.g. an ethernet switch could use this
	/// to send the event back out all but the port a packet came in on. This
	/// is assigned to the port_name field of [`Event`].
	pub remote_port: String,
	
	// We only use the T parameter for type checking but the compiler will
	// whine at us if we don't use it somewhere so we include this zero-sized
	// field.
	dummy: PhantomData<T>,
}

/// Use OutPort's connect_to method to connect up ports.
#[derive(Clone)]
pub struct InPort<T: Any + Send>
{
	target_id: ComponentID,
	target_port: String,
	dummy: PhantomData<T>,
}

impl<T: Any + Send> InPort<T>
{
	/// Creates an InPort with no component or port name. This is useful for components that
	/// wrap nested components, for example:
	///
	/// # Examples
	///
	/// ```
	/// use score::*;
	/// use std::thread;
	///
	/// struct BlackBox
	/// {
	/// 	data: ThreadData,
	/// 	nested: Nested,
	/// 	pub inbound: InPort<String>,	// connecting to this connects to nested's InPort
	/// }
	///
	/// impl BlackBox
	/// {
	/// 	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> BlackBox
	/// 	{
	/// 		let (id, data) = sim.add_active_component("black-box", parent_id);
	/// 		let mut device = BlackBox {
	/// 			data: data,
	/// 			nested: Nested::new(sim, id),
	/// 			inbound: InPort::empty(),
	/// 		};
	/// 		device.inbound = device.nested.input.clone();
	/// 		device
	/// 	}
	///
	/// 	pub fn start(mut self)
	/// 	{
	/// 		self.nested.start();	// nested is moved out
	///
	/// 		let data = self.data;	// so to avoid using a partially moved struct we move out the fields our thread needs
	/// 		thread::spawn(move || {
	/// 			process_events!(data, event, state, effector,
	/// 				"init 0" => {
	/// 					log_info!(effector, "initing!");
	/// 				}
	/// 			);
	/// 		});
	/// 	}
	/// }
	///
	/// struct Nested
	/// {
	/// 	data: ThreadData,
	/// 	input: InPort<String>,
	/// }
	///
	/// impl Nested
	/// {
	/// 	pub fn new(sim: &mut Simulation, parent_id: ComponentID) -> Nested
	/// 	{
	/// 		let (id, data) = sim.add_active_component("nested", parent_id);
	/// 		Nested {
	/// 			data: data,
	/// 			input: InPort::new(id),
	/// 		}
	/// 	}
	///
	/// 	pub fn start(self)
	/// 	{
	/// 		thread::spawn(move || {
	/// 			process_events!(self.data, event, state, effector,
	/// 				"init 0" => {
	/// 					log_info!(effector, "initing!");
	/// 				}
	/// 			);
	/// 		});
	/// 	}
	/// }
	/// ```
	pub fn empty() -> InPort<T>
	{
		InPort {
			target_id: NO_COMPONENT,
			target_port: "".to_string(),
			dummy: PhantomData,
		}
	}

	pub fn new(id: ComponentID) -> InPort<T>
	{
		InPort {
			target_id: id,
			target_port: "".to_string(),
			dummy: PhantomData,
		}
	}

	/// When this is used the [`Event`]'sport_name will be set which allows components
	/// to take different actions depending upon how the event arrived.
	pub fn with_port_name(id: ComponentID, port: &str) -> InPort<T>
	{
		InPort {
			target_id: id,
			target_port: port.to_string(),
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
		assert!(self.remote_id != NO_COMPONENT);
		let event = Event::with_port_payload(name, &self.remote_port, payload);
		effector.schedule_immediately(event, self.remote_id);
	}
	
	/// Queue up an event to be processed after secs time elapses.
	pub fn send_payload_after_secs(&self, effector: &mut Effector, name: &str, secs: f64, payload: T)
	{
		assert!(self.remote_id != NO_COMPONENT);
		let event = Event::with_port_payload(name, &self.remote_port, payload);
		effector.schedule_after_secs(event, self.remote_id, secs);
	}

	pub fn connect_to(&mut self, port: &InPort<T>)
	{
		assert!(port.target_id != NO_COMPONENT);
		self.remote_id = port.target_id;
		self.remote_port = port.target_port.to_string();	// can be empty
	}

	pub fn is_connected(&self) -> bool
	{
		self.remote_id != NO_COMPONENT
	}
}

impl OutPort<()>
{
	/// Queue up an event with no payload to be processed ASAP.
	pub fn send(&self, effector: &mut Effector, name: &str)
	{
		assert!(self.remote_id != NO_COMPONENT);
		let event = Event::with_port(name, &self.remote_port);
		effector.schedule_immediately(event, self.remote_id);
	}
	
	/// Queue up an event with no payload to be processed after secs time elapses.
	pub fn send_after_secs(&self, effector: &mut Effector, name: &str, secs: f64)
	{
		assert!(self.remote_id != NO_COMPONENT);
		let event = Event::with_port(name, &self.remote_port);
		effector.schedule_after_secs(event, self.remote_id, secs);
	}
}
