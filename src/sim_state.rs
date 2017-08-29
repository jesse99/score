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
use components::*;
use store::*;
use std::borrow::Borrow;
use std::sync::Arc;

/// This is sent together with an [`Event`] when the [`Simulation`] dispatches
/// an event to a [`Component`] thread.
pub struct SimState
{
	/// The components in the Simulation.
	pub components: Arc<Components>,
	
	/// The state of the [`Store`] at the [`Time`] the event was dispatched:
	/// changes to the simulation happen after all events at time T have
	/// finished processing.
	pub store: Arc<Store>,
}

impl SimState
{
	pub fn was_removed(&self, id: ComponentID) -> bool
	{
		let store:&Store = self.store.borrow();
		let key = self.components.full_path(id) + ".removed";
		store.contains(&key)
	}

	pub fn contains(&self, id: ComponentID, key: &str) -> bool
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.full_path(id), key);
		store.contains(&path)
	}

	pub fn get_int(&self, id: ComponentID, key: &str) -> i64
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.full_path(id), key);
		store.get_int(&path)
	}

	pub fn get_float(&self, id: ComponentID, key: &str) -> f64
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.full_path(id), key);
		store.get_float(&path)
	}

	pub fn get_string(&self, id: ComponentID, key: &str) -> String
	{
		let store:&Store = self.store.borrow();
		let path = format!("{}.{}", self.components.full_path(id), key);
		store.get_string(&path)
	}
}
