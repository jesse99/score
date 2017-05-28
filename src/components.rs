use component::*;
//use env::*;
//use std;

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
		let index = id.0;
		self.components.get(index).unwrap()
	}
	
	/// Use this if the component's lifetime may be dynamic.
	pub fn find(&self, id: ComponentID) -> Option<&Component>
	{
		let index = id.0;
		self.components.get(index)
	}
	
	// TODO: should be able to get the root and the top/first
	
	pub fn append(&mut self, component: Component)
	{
		self.components.push(component);
	}
}
