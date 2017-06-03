use component::*;

/// Contains all the `Component`s used within the `Simulation`.
pub struct Components
{
	components: Vec<Component>	// TODO: might want to either store an Option<Component> or have some sort of dead flag
}

impl Components
{
	pub fn new() -> Components
	{
		Components {components: Vec::new()}
	}
	
	/// Note that this should only be used for components that are not
	/// dynamically added/removed.
	pub fn get(&self, id: ComponentID) -> &Component
	{
		assert!(id != NO_COMPONENT);
		let index = id.0;
		self.components.get(index).unwrap()
	}
	
	/// Use this if the component's lifetime may be dynamic.
	pub fn find(&self, id: ComponentID) -> Option<&Component>	// TODO: are we really going to need this?
	{
		assert!(id != NO_COMPONENT);
		let index = id.0;
		self.components.get(index)
	}
	
	/// Returns the id of the root component.
	pub fn find_root_id(&self, id: ComponentID) -> ComponentID
	{
		assert!(id != NO_COMPONENT);

		let mut id = id;
		loop {
			let c = self.get(id);
			if c.parent == NO_COMPONENT {
				return id;
			}
			id = c.parent;
		}
	}
	
	/// Returns the id for the topmost parent of the component,
	/// i.e. one down from the root.
	pub fn find_top_id(&self, id: ComponentID) -> ComponentID
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
				return id;
			}
			id = c.parent;
		}
	}
	
	pub fn get_child_id(&self, id: ComponentID, name: &str) -> ComponentID
	{
		// TODO: should this include grand-kids?
		let c = self.get(id);
		for candidate in c.children.iter() {
			let d = self.get(*candidate);
			if d.name == name {
				return *candidate;
			}
		}
		
		panic!("Didn't find child {} within {}", name, c.name)
	}
	
	/// Iterates over all the components.
	pub fn iter<'a>(&'a self) -> Box<Iterator<Item=&'a Component> + 'a>
	{
		Box::new(self.components.iter())
	}
	
	/// Returns the path from the top component downwards.
	/// Note that this does not include the root component because it's a little silly
	/// to include it everywhere when it never changes.
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
		
	pub fn is_empty(&self) -> bool
	{
		self.components.is_empty()
	}
	
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
