use std;
use std::fmt;

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
