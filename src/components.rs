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
use std::collections::VecDeque;

/// Contains all the `Component`s used within the `Simulation`.
pub struct Components
{
	components: Vec<Component>,
	max_log_path: usize,
}

pub struct ComponentsIterator<'a>
{
	components: &'a Components,
	next: usize,
}

impl Components
{
	pub(crate) fn new(max_log_path: usize) -> Components
	{
		Components {components: Vec::new(), max_log_path}
	}
	
	/// Dump state to stdout.
	pub fn print(&self)
	{
		for (index, comp) in self.components.iter().enumerate() {
			let id = ComponentID(index);
			if comp.parent != NO_COMPONENT {
				let parent = self.get(comp.parent);
				println!("   {}: {} ({})", comp.name, parent.name, id);
			} else {
				println!("   {}: <none> ({})", comp.name, id);
			}
		}
	}
	
	/// Note that this, and related methods, can return a reference to
	/// a removed [`Component`]. This is not a problem in general but if
	/// you are regularly sending events to a component that may have
	/// been removed then you can make your simulation more efficient
	/// by asking `SimState` if the component has been removed.
	pub fn get(&self, id: ComponentID) -> &Component
	{
		assert!(id != NO_COMPONENT);
		let index = id.0;
		&self.components[index]
	}
	
	/// The root is the component that is the grand parent of all components.
	pub fn get_root(&self) -> (ComponentID, &Component)
	{
		// TODO: Might want to optimize this in case people do weird things
		// like adding the root last.
		for (index, comp) in self.components.iter().enumerate() {
			let id = ComponentID(index);
			if comp.parent == NO_COMPONENT {
				return (id, comp);
			}
		}

		panic!("Failed to find the root");
	}
	
	/// Returns the id for the topmost parent of the component,
	/// i.e. one down from the root. Note that it is an error to
	/// call this with an id already at the top or root.
	pub fn get_top(&self, id: ComponentID) -> (ComponentID, &Component)
	{
		assert!(id != NO_COMPONENT);

		let mut id = id;
		loop {
			let c = self.get(id);
			if c.parent == NO_COMPONENT {
				panic!("Can't find the top component when starting at the root");
			}

			let d = self.get(c.parent);
			if d.parent == NO_COMPONENT {
				return (id, c);
			}
			id = c.parent;
		}
	}
	
	/// Does a breadth first check for the first child that satisfies the predicate.
	pub fn find_child<P>(&self, id: ComponentID, predicate: P) -> Option<(ComponentID, &Component)>
		where P: Fn (ComponentID, &Component) -> bool
	{
		assert!(id != NO_COMPONENT);
		let mut workset = VecDeque::<ComponentID>::new();

		let component = &self.components[id.0];
		workset.extend(component.children.iter());
				
		while !workset.is_empty() {
			let child_id = workset.pop_back().unwrap();
			let child = &self.components[child_id.0];
			if predicate(child_id, child) {
				return Some((child_id, child));
			}

			for child_id in child.children.iter() {
				workset.push_front(*child_id);
			}
		}
	
		return None
	}
	
	pub fn for_each_child<P, C>(&self, id: ComponentID, predicate: P, callback: C)
		where P: Fn (ComponentID, &Component) -> bool, C: Fn (ComponentID, &Component) -> ()
	{
		assert!(id != NO_COMPONENT);

		let component = &self.components[id.0];
		for &child_id in component.children.iter() {
			let child = self.get(child_id);
			if predicate(child_id, child) {
				callback(child_id, child);
			}
		}
	}
	
	pub fn for_each_child_mut<P, C>(&self, id: ComponentID, predicate: P, callback: &mut C)
		where P: Fn (ComponentID, &Component) -> bool, C: FnMut (ComponentID, &Component) -> ()
	{
		assert!(id != NO_COMPONENT);

		let component = &self.components[id.0];
		for &child_id in component.children.iter() {
			let child = self.get(child_id);
			if predicate(child_id, child) {
				callback(child_id, child);
			}
		}
	}
	
	/// Find the first parent component that satisfies the predicate.
	pub fn find_parent<P>(&self, id: ComponentID, predicate: P) -> Option<(ComponentID, &Component)>
		where P: Fn (ComponentID, &Component) -> bool
	{
		assert!(id != NO_COMPONENT);

		let mut id = id;
		loop {
			let child = self.get(id);
			if child.parent == NO_COMPONENT {
				return None;
			}

			let parent_id = child.parent;
			let parent = self.get(parent_id);
			if predicate(parent_id, parent) {
				return Some((parent_id, parent));
			}

			id = parent_id;
		}
	}
		
	/// Iterates over all the components.
	pub fn iter(&self) -> ComponentsIterator
	{
		ComponentsIterator::new(self)
	}
	
	/// Returns the path from the top component downwards. Returns "removed"
	/// if id or a parent of id has been removed.
	pub fn full_path(&self, mut id: ComponentID) -> String
	{
		let mut path = Vec::new();
		
		while id != NO_COMPONENT {
			let c = self.get(id);
			path.insert(0, c.name.clone());
			id = c.parent;
		}
		
		path.join(".")
	}
				
	/// Like path except that the path is truncated from the left using max_log_path
	/// from [`Config`].
	pub fn display_path(&self, id: ComponentID) -> String
	{
		let path = self.full_path(id);
		format!("{0:<1$}", path, self.max_log_path)
	}
				
	pub fn is_empty(&self) -> bool
	{
		self.components.is_empty()
	}
	
	pub(crate) fn append(&mut self, id: ComponentID, component: Component, parent: ComponentID)
	{
		assert!(id != NO_COMPONENT);

		if parent != NO_COMPONENT {
			self.check_for_dupes(parent, &component);
		}

		if parent != NO_COMPONENT {
			let mut p = self.components.get_mut(parent.0).unwrap();
			p.children.push(id);
		}
		
		self.components.push(component);
	}
	
	#[cfg(debug_assertions)]
	fn check_for_dupes(&self, parent_id: ComponentID, child: &Component)
	{
		let parent = self.get(parent_id);
		for &existing_id in parent.children.iter() {
			let existing = self.get(existing_id);
			assert!(existing.name != child.name, "{} is already a child of {}", child.name, parent.name);
		}
	}
}

impl<'a> ComponentsIterator<'a>
{
	pub fn new(components: &'a Components) -> ComponentsIterator
	{
		ComponentsIterator {components: components, next: 0}
	}
}

impl<'a> Iterator for ComponentsIterator<'a>
{
	type Item = (ComponentID, &'a Component);
	
	fn next(&mut self) -> Option<Self::Item>
	{
		if self.next < self.components.components.len() {
			self.next += 1;
			Some((ComponentID(self.next-1), &self.components.components[self.next-1]))
		} else {
			None
		}
	}
}
