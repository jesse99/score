//! This is an outer doc comment.
pub mod env;
pub mod store;
pub mod time;

pub use env::*;
pub use store::*;
pub use time::*;

#[allow(unused)]
pub fn run()
{
	print!("running\n")
}
