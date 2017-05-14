//! This is an outer doc comment.
pub mod context;
pub mod env;
pub mod logger;
pub mod store;
pub mod time;

pub use context::*;
pub use env::*;
pub use logger::*;
pub use store::*;
pub use time::*;

type Executer = fn (env: &Env, context: &mut Context) -> ();

#[allow(unused)]
pub fn run(executer: Executer)
{
	let env = Env::_new();
	let mut context = Context{logger: &env, name: "elevator".to_string(), store: Store::_new()};
	env.log_debug("rsimbase", "starting up");
	executer(&env, &mut context);
}
