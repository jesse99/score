//! This is used to persist all of the significant state within a simulation.
//! It is a write-once temporal store, i.e. new values can be written to the
//! current time but values at prior times cannot be overwritten. The store is
//! normally written to disk to allow for off-line analysis of the results and
//! to allow the simulation to be replayed. The store contains settings and data.
//!
//! _Settings_ are typically configured on startup and not changed. In the elevator
//! example the number of elevators is a setting.
//!
//! _Data_, on the other hand, typically does change as the simulation runs. In the
//! elevator example the number of people waiting to get on the elevator is data.
//!
//! Note that there is no fundamental difference between the two: having them both
//! simply better expresses intent and makes GUIs a bit nicer.
//!
//! _Getters_ take a &str key and return either an i64, an f64, or a &str. The key
//! is normally a path from the root component through the inner components to a setting
//! or data name. The value returned is that for the current time. Note that it is a
//! programmer error if the key or description is missing.
//!
//! _Setters_ set a value for the current time. To ensure thread safety setters are
//! invoked via LocalEnv.
use std::collections::HashMap;
use time::*;

#[allow(unused)]	// TODO: remove this
pub struct Store
{
	pub descriptions: HashMap<String, String>,

	pub int_settings: HashMap<String, (Time, i64)>,
	pub int_data: HashMap<String, (Time, i64)>,

	pub float_settings: HashMap<String, (Time, f64)>,
	pub float_data: HashMap<String, (Time, f64)>,

	pub string_settings: HashMap<String, (Time, String)>,
	pub string_data: HashMap<String, (Time, String)>,
}

#[allow(unused)]	// TODO: remove this
impl Store
{
	// --- descriptions ----------------------------------------------------------
	pub fn get_description(&self, key: &str) -> String
	{
		match self.descriptions.get(key) {
			Some(ref value) => return value.to_string(),
			_ => panic!("description for key '{}' is missing", key)
		}
	}

	pub fn set_description(&mut self, key: &str, value: i64, time: Time)
	{
		if let Some(_) = self.descriptions.insert(key.to_string(), value.to_string()) {
			panic!("description for key '{}' has already been set", key)
		}
	}
	
	// --- setting getters -------------------------------------------------------
	pub fn get_int_setting(&self, key: &str) -> i64
	{
		match self.int_settings.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("int key '{}' is missing", key)
		}
	}

	pub fn get_float_setting(&self, key: &str) -> f64
	{
		match self.float_settings.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("float key '{}' is missing", key)
		}
	}

	pub fn get_string_setting(&self, key: &str) -> String
	{
		match self.string_settings.get(key) {
			Some(ref value) => return value.1.clone(),
			_ => panic!("string key '{}' is missing", key)
		}
	}

	// --- data getters ----------------------------------------------------------
	pub fn get_int_data(&self, key: &str) -> i64
	{
		match self.int_data.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("int key '{}' is missing", key)
		}
	}

	pub fn get_float_data(&self, key: &str) -> f64
	{
		match self.float_data.get(key) {
			Some(ref value) => return value.1,
			_ => panic!("float key '{}' is missing", key)
		}
	}

	pub fn get_string_data(&self, key: &str) -> String
	{
		match self.string_data.get(key) {
			Some(ref value) => return value.1.clone(),
			_ => panic!("string key '{}' is missing", key)
		}
	}

	// --- private methods -------------------------------------------------------
	#[doc(hidden)]
	pub fn _new() -> Store
	{
		Store{
			descriptions: HashMap::new(),
			
			int_settings: HashMap::new(),
			int_data: HashMap::new(),
			
			float_settings: HashMap::new(),
			float_data: HashMap::new(),
			
			string_settings: HashMap::new(),
			string_data: HashMap::new()
		}
	}
	
		// TODO: call this when the simulation exits
	// #[doc(hidden)]
	//	pub fn _check_descriptions(&self, local: &mut LocalEnv)
	//	{
	//		// instead of a LocalEnv take a logger trait
	//		// then for testing can uise a special version
	//		// if any values are missing a description then call a log method on LocalEnv
	//		// should use an "error" topic
	//	}
	
	#[doc(hidden)]
	pub fn _set_int_setting(&mut self, key: &str, value: i64, time: Time)
	{
		if let Some(old) = self.int_settings.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				// If it becomes annoying to be unable to set a value more than once then
				// we could add change methods (or maybe weaken the precondition by allowing
				// people to set the same value more than once).
				panic!("int key '{}' has already been set", key)
			}
		}
	}
	
	#[doc(hidden)]
	pub fn _set_float_setting(&mut self, key: &str, value: f64, time: Time)
	{
		if let Some(old) = self.float_settings.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				panic!("float key '{}' has already been set", key)
			}
		}
	}
	
	#[doc(hidden)]
	pub fn _set_string_setting(&mut self, key: &str, value: &str, time: Time)
	{
		if let Some(old) = self.string_settings.insert(key.to_string(), (time, value.to_string())) {
			if old.0 == time {
				panic!("string key '{}' has already been set", key)
			}
		}
	}
	
	#[doc(hidden)]
	pub fn _set_int_data(&mut self, key: &str, value: i64, time: Time)
	{
		if let Some(old) = self.int_data.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				panic!("int key '{}' has already been set", key)
			}
		}
	}
	
	#[doc(hidden)]
	pub fn _set_float_data(&mut self, key: &str, value: f64, time: Time)
	{
		if let Some(old) = self.float_data.insert(key.to_string(), (time, value)) {
			if old.0 == time {
				panic!("float key '{}' has already been set", key)
			}
		}
	}
		
	#[doc(hidden)]
	pub fn _set_string_data(&mut self, key: &str, value: &str, time: Time)
	{
		if let Some(old) = self.string_data.insert(key.to_string(), (time, value.to_string())) {
			if old.0 == time {
				panic!("string key '{}' has already been set", key)
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
		let store = Store::_new();
		store.get_int_setting("foo");
	}
	
	#[test]
	fn has_value()
	{
		let mut store = Store::_new();
		store._set_int_setting("weight", 120, Time(0));
		let weight = store.get_int_setting("weight");
		assert_eq!(weight, 120);
	}
	
	#[test]
	fn has_new_value()
	{
		let mut store = Store::_new();
		store._set_int_setting("weight", 120, Time(0));
		store._set_int_setting("weight", 130, Time(1));
		let weight = store.get_int_setting("weight");
		assert_eq!(weight, 130);
	}
	
	#[test]
	#[should_panic(expected = "already been set")]
	fn changing_value()
	{
		let mut store = Store::_new();
		store._set_int_setting("weight", 120, Time(1));
		store._set_int_setting("weight", 130, Time(1));
	}
}
