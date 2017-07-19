use effector::*;
use std;
use std::fmt;

/// `IntValue` and `FloatValue` and `StringValue` are simple wrappers around an
/// [`Effector`]. They don't do very much but they assist in creating type safe
/// [`Component`] structs. See the [`set_value`] macro for an example.
#[derive(Copy, Clone)]
pub struct IntValue
{
}

#[derive(Copy, Clone)]
pub struct FloatValue
{
}

#[derive(Copy, Clone)]
pub struct StringValue
{
}

impl IntValue
{
	/// This is normally called via the set_value! macro.
	pub fn set_value(&self, effector: &mut Effector, name: &str, value: i64)
	{
		effector.set_int(name, value);
	}
}

impl FloatValue
{
	/// This is normally called via the set_value! macro.
	pub fn set_value(&self, effector: &mut Effector, name: &str, value: f64)
	{
		effector.set_float(name, value);
	}
}

impl StringValue
{
	/// This is normally called via the set_value! macro.
	pub fn set_value(&self, effector: &mut Effector, name: &str, value: &str)
	{
		effector.set_string(name, value);
	}
}

/// Type safe way to update the simulation [`Store`].
///
/// # Examples
///
/// ```
/// use score::*;
///
/// struct InterfaceComponent
/// {
/// 	tx_packets: IntValue,
/// }
///
/// fn reset(iface: InterfaceComponent, effector: &mut Effector)
/// {
/// 	// The effector is actually the object that is updated.
/// 	// When the component finishes processing the current event
/// 	// the simulation will apply all the effects from effectors.
/// 	set_value!(effector, iface.tx_packets = 0);
/// }
/// ```
#[macro_export]
macro_rules! set_value
{
	($effector:ident, $target:ident . $field:ident = $value:expr) => ({
		$target.$field.set_value(&mut $effector, stringify!($field), $value);
	});
}

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
