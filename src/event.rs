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
use std::any::Any;

/// Events are scheduled to be sent to a `Component` at a particular `Time`.
/// Components process the event using a thread and send an `Effector` back
/// to the `Simulation` which encapsulates the state changes they wish to
/// make.
pub struct Event
{
	/// Typically components may process different types of events so this
	/// is what they check to decide what they need to do.
	pub name: String,
	
	/// If the event was delivered via a named port then this will be the field
	/// name of the port the event came in on.
	pub port_name: String,
	
	/// Arbitrary extra information associated with the event.
	pub payload: Option<Box<Any + Send>>,
}

impl Event
{
	pub fn new(name: &str) -> Event
	{
		assert!(!name.is_empty(), "name should not be empty");
		Event{name: name.to_string(), port_name: "".to_string(), payload: None}
	}

	pub fn with_payload<T: Any + Send>(name: &str, payload: T) -> Event
	{
		assert!(!name.is_empty(), "name should not be empty");
		Event{name: name.to_string(), port_name: "".to_string(), payload: Some(Box::new(payload))}
	}

	pub fn with_port(name: &str, port: &str) -> Event
	{
		assert!(!name.is_empty(), "name should not be empty");
		Event{name: name.to_string(), port_name: port.to_string(), payload: None}
	}

	pub fn with_port_payload<T: Any + Send>(name: &str, port: &str, payload: T) -> Event
	{
		assert!(!name.is_empty(), "name should not be empty");
		Event{name: name.to_string(), port_name: port.to_string(), payload: Some(Box::new(payload))}
	}

	pub fn expect_payload<T: Any>(&self, message: &str) -> &T
	{
		if let Some(ref value) = self.payload {
			if let Some(x) = value.downcast_ref::<T>() {
				x
			} else {
				panic!("event {} {} (downcast failed)", self.name, message);
			}
		} else {
			panic!("event {} {} (missing payload)", self.name, message);
		}
	}
}
