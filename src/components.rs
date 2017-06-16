use component::*;
use std::collections::VecDeque;

/// Contains all the `Component`s used within the `Simulation`.
pub struct Components
{
	components: Vec<Component>,
}

pub struct ComponentsIterator<'a>
{
	components: &'a Components,
	next: usize,
}

impl Components
{
	pub fn new() -> Components
	{
		Components {components: Vec::new()}
	}
	
	/// Note that this, and related methods, can return a reference to
	/// a removed `Component`. This is not a problem in general but if
	/// you are regularly sending events to a component that may have
	/// been removed then you can make your simulation more efficient
	/// by asking `SimState` if the component has been removed.
	pub fn get(&self, id: ComponentID) -> &Component
	{
		assert!(id != NO_COMPONENT);
		let index = id.0;
		&self.components[index]
	}
	
	pub fn get_root(&self, id: ComponentID) -> (ComponentID, &Component)
	{
		assert!(id != NO_COMPONENT);

		let mut id = id;
		loop {
			let c = self.get(id);
			if c.parent == NO_COMPONENT {
				return (id, c);
			}
			id = c.parent;
		}
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
	/// if id or a parent of id has been removed. Note that this does not
	/// include the root component because it's a little silly to include
	/// it everywhere when it never changes.
	pub fn path(&self, id: ComponentID) -> String
	{
		let mut path = Vec::new();
		
		let mut c = self.get(id);
		while c.parent != NO_COMPONENT {
			path.insert(0, c.name.clone());
			c = self.get(c.parent);
		}
		
		path.join(".")
	}
		
	#[doc(hidden)]
	pub fn is_empty(&self) -> bool
	{
		self.components.is_empty()
	}
	
	#[doc(hidden)]
	pub fn append(&mut self, id: ComponentID, component: Component, parent: ComponentID)
	{
		assert!(id != NO_COMPONENT);

		// TODO: should check for cycles
		if parent != NO_COMPONENT {
			let mut p = self.components.get_mut(parent.0).unwrap();
			p.children.push(id);
		}
		
		self.components.push(component);
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
