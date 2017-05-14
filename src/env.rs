use time::*;
use store::*;

/// This provides access to shared resources.
pub struct Env
{
	/// Time advances to what ever time the next component is scheduled to
	/// execute at.
	pub time: Time,
	
	/// This is where simulation state is stored. As components execute
	/// they write state to their Context object which is merged into
	/// the Env state once all the components for a time slice execute.
	pub store: Store
}

impl Env
{
	#[doc(hidden)]
	pub fn _new() -> Env
	{
		Env{time: Time(0), store: Store::_new()}
	}
}

