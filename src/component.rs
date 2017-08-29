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
use std;
use std::fmt;

/// `Component`s are the building blocks of a `Simulation`. They are arranged into
/// a tree and use a thread to respond to `Event`s which execute at some particular
/// `Time`. Note that, in general, all state managed within components should be
/// stored in the `Store`. This makes it possible to use GUI tools to see what is
/// happening within components and even more importantly allows the Simulation to
/// perform speculative execution of components.
///
/// Typically type safe structs are defined for components with the aid of [`OutPort`],
/// [`InPort`], [`IntValue`], etc.
pub struct Component
{
	/// The name of the component. Note that, in general, these are not unique.
	pub name: String,
	
	/// ID for the component's parent. The root component will return NO_COMPONENT.
	pub parent: ComponentID,
	
	pub children: Vec<ComponentID>,
}

/// To make lifetime management easier components are referenced using a small
/// integer instead of a rust reference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComponentID(pub usize);

/// The id of the root component.
pub const NO_COMPONENT: ComponentID = ComponentID(std::usize::MAX);

impl fmt::Display for ComponentID
{
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result
	{
		write!(formatter, "{}", self.0)
	}
}
