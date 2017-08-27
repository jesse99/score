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
use component::*;
use event::*;
use logging::*;
use sim_time::*;
use store::*;
use std::f64::EPSILON;

/// Effectors are returned by [`Component`]s after they process an [`Event`].
/// The effector encapsulates the state changes the component wishes to make.
pub struct Effector
{
	pub(crate) logs: Vec<LogRecord>,
	pub(crate) events: Vec<(ComponentID, Event, f64)>,
	pub(crate) store: Store,
	pub(crate) exit: bool,
	pub(crate) removed: bool,
}

// It'd be nice to wrap this up in a smart pointer so that we could do the send
// when the pointer is dropped. But we can't move fields out of a struct in a
// drop method (see E0509) so we'd have to do a copy which could be expensive.
impl Effector
{
	pub fn new() -> Effector
	{
		Effector{logs: Vec::new(), events: Vec::new(), store: Store::new(), exit: false, removed: false}
	}
	
	/// Normally you'll use one of the log macros, e.g. log_info!.
	pub fn log(&mut self, level: LogLevel, message: &str)
	{
		self.logs.push(LogRecord{level, message: message.to_string()});
	}
	
	/// Dispatch an event to a component after secs time elapses.
	pub fn schedule_after_secs(&mut self, event: Event, to: ComponentID, secs: f64)
	{
		assert!(to != NO_COMPONENT);
		assert!(secs > 0.0, "secs ({:.3}) is not positive", secs);	// negative secs are just bad, for zero secs use schedule_immediately

		self.events.push((to, event, secs));
	}
	
	/// Events should not be scheduled for zero time because the `Simulation` guarantees
	/// that state is updated all at once at each time step. So if you want to schedule
	/// an event for as soon as possible use this method.
	pub fn schedule_immediately(&mut self, event: Event, to: ComponentID)
	{
		assert!(to != NO_COMPONENT);

		self.events.push((to, event, EPSILON));
	}
	
	/// Exit the sim after all events at the current time have been processed.
	pub fn exit(&mut self)
	{
		self.exit = true;
	}
	
	/// This will swap in a [`Component`] thread that drops all events and add a removed=1
	/// data entry to the store (so GUIs can stop rendering the component). Note that
	/// this is done for the associated component and all its children.
	pub fn remove(&mut self)
	{
		self.removed = true;
	}
	
	/// Use these methods to write out new values for data associated with the component.
	/// Note that when the data is written to the main store the name will be appended
	/// onto the component's path.
	///
	/// There is one special int valued key:
	/// * removed - This is added when score removes a component via `Effector`'s remove method.
	/// Client code should use [`SimState`]'s was_removed method instead of directly accessing
	/// this value.
	pub fn set_int(&mut self, name: &str, value: i64)
	{
		assert!(!name.is_empty(), "name should not be empty");
		self.store.set_int(name, value, Time(0));
	}
	
	/// There are several special float valued keys:
	/// * display-location-x and y - These are used by GUIs (like sdebug) to position top level
	/// component's within a map view (the origin is at the upper left).
	/// * display-size-x and y - The dimensions of the map view.
	pub fn set_float(&mut self, name: &str, value: f64)
	{
		assert!(!name.is_empty(), "name should not be empty");
		self.store.set_float(name, value, Time(0));
	}
		
	/// There are several special string valued keys:
	/// * display-color - An X11 color name used by GUI map views when drawing top level components.
	/// * display-details - Arbitrary text used when drawing top level component and displaying component hierarchies.
	/// * display-name - For now this is used instead of an icon when drawing components in sdebug's map view.
	/// * display-title - Used to give GUIs a simulation specific name for header text.
	pub fn set_string(&mut self, name: &str, value: &str)
	{
		assert!(!name.is_empty(), "name should not be empty");
		self.store.set_string(name, value, Time(0));
	}
}

pub(crate) struct LogRecord
{
	pub(crate) level: LogLevel,
	pub(crate) message: String,
}

