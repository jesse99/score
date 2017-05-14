use time::*;
use store::*;

/// This provides access to shared resources.
pub struct Env
{
	/// Time advances to what ever time the next component is scheduled to
	/// execute at.
	pub time: Time,
	
	/// Provides access to simulation state.
	pub store: Store
}

#[allow(unused)]	// TODO: remove this
impl Env
{
	#[doc(hidden)]
	pub fn _new() -> Env
	{
		Env{time: Time(0), store: Store::_new()}
	}
}

