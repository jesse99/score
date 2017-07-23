//! `IntValue` and `FloatValue` and `StringValue` are simple wrappers around an
//! [`Effector`]. They don't do very much but they assist in creating type safe
//! [`Component`] structs. See the [`set_value`] macro for an example.
use effector::*;

pub struct IntValue
{
}

pub struct FloatValue
{
}

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
/// fn reset(iface: InterfaceComponent, mut effector: Effector)
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
