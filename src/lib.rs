//! This is an outer doc comment.
pub mod env;
pub mod store;
pub mod time;

pub fn hello()
{
	println!("hey there");
}

pub fn four() -> i32
{
	4
}

#[cfg(test)]
mod tests
{
	use super::*;
	
	#[test]
	fn it_works()
	{
		assert_eq!(four(), 4);
	}
}