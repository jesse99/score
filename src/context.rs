use logger::*;
use store::*;

/// Encapsulates information related to the component currently being executed.
pub struct Context<'a>
{
	/// Instead of using the log methods on Env or this field directly it is
	/// generally simplest to use the log methods on Context.
	pub logger: &'a Logger,
	
	/// The name of the component currently being executed. This is used
	/// by the log methods on Context.
	pub name: String,
	
	/// As components execute they write state here. (If they want to read
	/// state they will ordinarily use env.store).
	pub store: Store
}

impl<'a> Context<'a>
{
	pub fn log_error(&self, message: &str)
	{
		self.logger.log_error(&self.name, message);
	}

	pub fn log_warning(&self, message: &str)
	{
		self.logger.log_warning(&self.name, message);
	}

	pub fn log_info(&self, message: &str)
	{
		self.logger.log_info(&self.name, message);
	}

	pub fn log_debug(&self, message: &str)
	{
		self.logger.log_debug(&self.name, message);
	}

	pub fn log_excessive(&self, message: &str)
	{
		self.logger.log_excessive(&self.name, message);
	}
}

