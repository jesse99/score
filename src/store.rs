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
/// data name. The value returned is that for the current time. Note that it is a
/// programmer error if the key or description is missing.
///
/// _Setters_ set a value for the current time. To ensure thread safety and to allow
/// speculative execution setters are invoked by the `Simulation` using the information
/// `Component`s recorded within an `Effector`.
pub struct Store
{
	#[doc(hidden)]
	pub descriptions: HashMap<String, String>,
	
	#[doc(hidden)]
	pub int_data: HashMap<String, (Time, i64)>,
	
	#[doc(hidden)]
	pub float_data: HashMap<String, (Time, f64)>,
	
	#[doc(hidden)]
	pub string_data: HashMap<String, (Time, String)>,
}

pub trait ReadableStore
{
	fn get_description(&self, key: &str) -> String;
	
	fn has_data(&self, key: &str) -> bool;

	fn get_int_data(&self, key: &str) -> i64;
	fn get_float_data(&self, key: &str) -> f64;
	fn get_string_data(&self, key: &str) -> String;
}

pub trait WriteableStore
{
	fn set_description(&mut self, key: &str, value: &str);
	fn set_int_data(&mut self, key: &str, value: i64, time: Time);
	fn set_float_data(&mut self, key: &str, value: f64, time: Time);
	fn set_string_data(&mut self, key: &str, value: &str, time: Time);
}

impl ReadableStore for Store
{
	// --- descriptions ----------------------------------------------------------
	fn get_description(&self, key: &str) -> String
	{
		match self.descriptions.get(key) {
			Some(ref value) => return value.to_string(),
			_ => panic!("description for key '{}' is missing", key)
		}
	}
	
	// --- data ------------------------------------------------------------------
	fn has_data(&self, key: &str) -> bool
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

	fn get_int_data(&self, key: &str) -> i64
	{
		match self.int_data.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("int key '{}' is missing", key)
		}
	}

	fn get_float_data(&self, key: &str) -> f64
	{
		match self.float_data.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("float key '{}' is missing", key)
		}
	}

	fn get_string_data(&self, key: &str) -> String
	{
		match self.string_data.get(key) {
			Some(ref value) => return value.1.clone(),
			_ => panic!("string key '{}' is missing", key)
		}
	}
}

impl WriteableStore for Store
{
	// --- descriptions ----------------------------------------------------------
	fn set_description(&mut self, key: &str, value: &str)
	{
		assert!(!key.is_empty(), "key should not be empty");
		if let Some(_) = self.descriptions.insert(key.to_string(), value.to_string()) {
			panic!("description for key '{}' has already been set", key)
		}
	}

	// --- data ------------------------------------------------------------------
	fn set_int_data(&mut self, key: &str, value: i64, time: Time)
	{
		assert!(!key.is_empty(), "key should not be empty");
		if let Some(old) = self.int_data.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				panic!("int key '{}' has already been set", key)
			}
		}
	}
	
	fn set_float_data(&mut self, key: &str, value: f64, time: Time)
	{
		assert!(!key.is_empty(), "key should not be empty");
		if let Some(old) = self.float_data.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				panic!("float key '{}' has already been set", key)
			}
		}
	}
		
	fn set_string_data(&mut self, key: &str, value: &str, time: Time)
	{
		assert!(!key.is_empty(), "key should not be empty");
		if let Some(old) = self.string_data.insert(key.to_string(), (time, value.to_string())) {
			if old.0 == time {
				panic!("string key '{}' has already been set", key)
			}
		}
	}
}

impl Store
{
	pub fn new() -> Store
	{
		Store{
			descriptions: HashMap::new(),
			
			int_data: HashMap::new(),
			float_data: HashMap::new(),
			string_data: HashMap::new()
		}
	}
	
	#[doc(hidden)]
	pub fn check_descriptions(&self) -> Vec<String>
	{
		let mut errors = Vec::new();
		
		self.check_descriptions_for(&self.int_data, &mut errors);
		self.check_descriptions_for(&self.float_data, &mut errors);
		self.check_descriptions_for(&self.string_data, &mut errors);
		
		errors
	}
	
	fn check_descriptions_for<V>(&self, map: &HashMap<String, (Time, V)>, errors: &mut Vec<String>)
	{
		for name in map.keys() {
			if !self.descriptions.contains_key(name) && !name.ends_with(".removed") {
				errors.push(format!("Effector.set_description was not called for '{}'", name));
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
		store.get_int_data("foo");
	}
	
	#[test]
	fn has_value()
	{
		let mut store = Store::new();
		store.set_int_data("weight", 120, Time(0));
		let weight = store.get_int_data("weight");
		assert_eq!(weight, 120);
	}
	
	#[test]
	fn has_new_value()
	{
		let mut store = Store::new();
		store.set_int_data("weight", 120, Time(0));
		store.set_int_data("weight", 130, Time(1));
		let weight = store.get_int_data("weight");
		assert_eq!(weight, 130);
	}
	
	#[test]
	#[should_panic(expected = "already been set")]
	fn changing_value()
	{
		let mut store = Store::new();
		store.set_int_data("weight", 120, Time(1));
		store.set_int_data("weight", 130, Time(1));
	}
}
