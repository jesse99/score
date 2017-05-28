//use context::*;
//use env::*;
use std;

/// `Component`s are the building blocks of a `Simulation`. They are arranged into
/// a tree and use a thread to respond to `Event`s which execute at some particular
/// `Time`. Note that, in general, all state managed within components should be
/// stored in the `Store`. This makes it possible to use GUI tools to see what is
/// happening within components and even more importantly allows the Simulation to
/// perform speculative execution of components.
pub struct Component
{
	/// The name of the component. Note that, in general, these are not unique.
	pub name: String,
	
	/// ID for the component's parent. The root component will return NO_COMPONENT.
	pub parent: ComponentID,
	
	pub children: Vec<ComponentID>,
}

// Formed by concatenating component names from the root to this component.
// These do uniquely identify a component.
//fn path(&self) -> String
//{
//	self.name().clone()
//}

/// To make lifetime management easier components are referenced using a small
/// integer instead of a rust reference.
#[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct ComponentID(pub usize);

/// The id of the root component.
pub const NO_COMPONENT: ComponentID = ComponentID(std::usize::MAX);
