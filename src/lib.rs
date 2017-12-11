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

//! score is a general purpose discrete event simulator written in rust.
//! The key types are:
//!
//! *   Simulation is responsible for coordinating the execution of the simulation.
//! *   Components are used to define the structure of the simulation. Active components have a thread used to handle events.
//! *   Events are named messages sheduled to be delivered to a component at a specific time. Events may have am optional payload (which must satisfy the Any and Send traits).
//! *   The Store is where components persist state. (Using the store allows state to be viewed and changed using GUI tools like sdebug and allows side effects to be carefully managed.)
//! *   Components use an Effector to make changes. Components can use a an effector to log, change their own state within the store, and schedule events to be sent to arbitrary components.

extern crate glob;
extern crate rand;
extern crate rustc_serialize;
extern crate time;

#[macro_use]
extern crate rouille;

mod component;
mod components;
mod config;
mod effector;
mod event;
mod logging;
mod ports;
mod simulation;
mod sim_state;
mod sim_time;
mod store;
mod thread_data;
mod values;

pub use component::*;
pub use components::*;
pub use config::*;
pub use effector::*;
pub use event::*;
pub use logging::*;
pub use ports::*;
pub use simulation::*;
pub use sim_state::*;
pub use sim_time::*;
pub use store::*;
pub use thread_data::*;
pub use values::*;

