use store::*;
use time::*;

/// This provides access to shared resources.
pub struct Env
{
	/// Time advances to what ever time the next component is scheduled to
	/// execute at.
	time: Time,
	
	/// Provides access to simulation state.
	store: internals::Store
}

pub struct LocalEnv
{
	/// Time advances to what ever time the next component is scheduled to
	/// execute at.
	time: Time,
	
	/// The name of the component currently being executed. This is used
	/// by the log methods.
	name: String,
	
	/// Provides access to simulation state.
	store: internals::Store
}

#[allow(unused)]	// TODO: remove this
impl Env
{
	fn new() -> Env
	{
		Env{time: Time(0), store: internals::Store::new()}
	}
}

pub fn get_description(env: Env, key: &str) -> String
{
	env.store.get_description(key)
}

// error
// warn
// info
// debug
// excessive
#[allow(unused)]	// TODO: remove this
impl LocalEnv
{
	pub fn log_error(&mut self, message: &str)
	{
		self.store.set_string_data("logger", self.name + message, self.time);
	}
}

pub fn execute(local: &mut LocalEnv)
{
	local.log_error("oops");
}