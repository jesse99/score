use sim_time::*;
use std::collections::HashMap;

/// This is used to persist all of the significant state within a simulation.
/// It is a write-once temporal store, i.e. new values can be written to the
/// current time but values at prior times cannot be overwritten. The store is
/// normally written to disk to allow for off-line analysis of the results and
/// to allow the simulation to be replayed.
///
/// _Getters_ take a &str key and return either an i64, an f64, or a &str. The key
/// is normally a path from the root component through the inner components to a
/// data name. The value returned is that for the current time.
///
/// _Setters_ set a value for the current time. To ensure thread safety and to allow
/// speculative execution setters are invoked by the [`Simulation`] using the information
/// [`Component`]s recorded within an [`Effector`].
pub struct Store
{
	pub(crate) edition: u32,
	pub(crate) int_data: HashMap<String, (Time, i64)>,	// TODO: probably want [(Time, i64)]
	pub(crate) float_data: HashMap<String, (Time, f64)>,
	pub(crate) string_data: HashMap<String, (Time, String)>,
}

pub trait ReadableStore
{
	fn contains(&self, key: &str) -> bool;

	fn get_int(&self, key: &str) -> i64;
	fn get_float(&self, key: &str) -> f64;
	fn get_string(&self, key: &str) -> String;
}

pub trait WriteableStore
{
	fn set_int(&mut self, key: &str, value: i64, time: Time);
	fn set_float(&mut self, key: &str, value: f64, time: Time);
	fn set_string(&mut self, key: &str, value: &str, time: Time);
}

impl ReadableStore for Store
{
	fn contains(&self, key: &str) -> bool
	{
		if let Some(_) = self.int_data.get(key) {
			return true
		}
		if let Some(_) = self.float_data.get(key) {
			return true
		}
		if let Some(_) = self.string_data.get(key) {
			return true
		}
		false
	}

	fn get_int(&self, key: &str) -> i64
	{
		match self.int_data.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("int key '{}' is missing", key)
		}
	}

	fn get_float(&self, key: &str) -> f64
	{
		match self.float_data.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("float key '{}' is missing", key)
		}
	}

	fn get_string(&self, key: &str) -> String
	{
		match self.string_data.get(key) {
			Some(ref value) => return value.1.clone(),
			_ => panic!("string key '{}' is missing", key)
		}
	}
}

impl WriteableStore for Store
{
	fn set_int(&mut self, key: &str, value: i64, time: Time)
	{
		assert!(!key.is_empty(), "key should not be empty");
		if let Some(old) = self.int_data.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				panic!("int key '{}' has already been set", key)
			}
			if old.1 != value {
				// Edition is used by REST to detect changes to values in the store so we
				// don't want to increment it when the same value is added again.
				self.edition = self.edition.wrapping_add(1);
			}
		} else {
			self.edition = self.edition.wrapping_add(1);
		}
	}
	
	fn set_float(&mut self, key: &str, value: f64, time: Time)
	{
		assert!(!key.is_empty(), "key should not be empty");
		if let Some(old) = self.float_data.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				panic!("float key '{}' has already been set", key)
			}
			if old.1 != value {
				self.edition = self.edition.wrapping_add(1);
			}
		} else {
			self.edition = self.edition.wrapping_add(1);
		}
	}
		
	fn set_string(&mut self, key: &str, value: &str, time: Time)
	{
		assert!(!key.is_empty(), "key should not be empty");
		if let Some(old) = self.string_data.insert(key.to_string(), (time, value.to_string())) {
			if old.0 == time {
				panic!("string key '{}' has already been set", key)
			}
			if old.1 != value {
				self.edition = self.edition.wrapping_add(1);
			}
		} else {
			self.edition = self.edition.wrapping_add(1);
		}
	}
}

impl Store
{
	pub(crate) fn new() -> Store
	{
		Store{
			edition: 0,
			int_data: HashMap::new(),
			float_data: HashMap::new(),
			string_data: HashMap::new()
		}
	}
			
	/// Dump state to stdout.
	pub fn print(&self, time_units: f64, precision: usize)
	{
		for (key, value) in self.int_data.iter() {
			if !key.contains("display-") {
				let t = ((value.0).0 as f64)/time_units;
				println!("   {} = {} @ {:.3$}s", key, value.1, t, precision);
			}
		}
		for (key, value) in self.float_data.iter() {
			if !key.contains("display-") {
				let t = ((value.0).0 as f64)/time_units;
				println!("   {} = {:.3} @ {:.3$}s", key, value.1, t, precision);
			}
		}
		for (key, value) in self.string_data.iter() {
			if !key.contains("display-") {
				let t = ((value.0).0 as f64)/time_units;
				println!("   {} = '{}' @ {:.3$}s", key, value.1, t, precision);
			}
		}
	}

	// TODO:
	// persist old state
	// flush all the state to a file on exit
	// need to expose state via a REST API
	// reflected metadata
	// stuff GUIs will need for replay
}

#[cfg(test)]
mod tests
{
	use super::*;
	
	#[test]
	#[should_panic(expected = "key 'foo' is missing")]
	fn mising_key()
	{
		let store = Store::new();
		store.get_int("foo");
	}
	
	#[test]
	fn has_value()
	{
		let mut store = Store::new();
		store.set_int("weight", 120, Time(0));
		let weight = store.get_int("weight");
		assert_eq!(weight, 120);
	}
	
	#[test]
	fn has_new_value()
	{
		let mut store = Store::new();
		store.set_int("weight", 120, Time(0));
		store.set_int("weight", 130, Time(1));
		let weight = store.get_int("weight");
		assert_eq!(weight, 130);
	}
	
	#[test]
	#[should_panic(expected = "already been set")]
	fn changing_value()
	{
		let mut store = Store::new();
		store.set_int("weight", 120, Time(1));
		store.set_int("weight", 130, Time(1));
	}
}
