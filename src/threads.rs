//! Some component threads that are generally useful.
use effector::*;
use logging::*;
use store::*;
use std::thread;
use thread_data::*;

/// A thread to use for a top-level component that maintains a location
/// within "location-x" and "location-y" float data. To change the location
/// send a "set-location" or "offset-location" event with an (f64, f64) payload.
pub fn locatable_thread(data: ThreadData)
{
	thread::spawn(move || {
		for dispatched in data.rx {
			let mut effector = Effector::new();

			{
			let cname = &(*dispatched.components).get(data.id).name;
			let ename = &dispatched.event.name;
			if ename == "set-location" {
				let loc = dispatched.expect_payload::<(f64, f64)>(&format!("component {} set-location should have an (f64, f64) payload", cname));
				log_info!(effector, "setting location to {:.1}, {:.1}", loc.0, loc.1);
				effector.set_float_data("location-x", loc.0);
				effector.set_float_data("location-y", loc.1);
				
			} else if ename == "offset-location" {
				let path = dispatched.components.path(data.id);
				let x = dispatched.store.get_float_data(&(path.clone() + ".location-x"));
				let y = dispatched.store.get_float_data(&(path + ".location-y"));

				let loc = dispatched.expect_payload::<(f64, f64)>(&format!("component {} offset-location should have an (f64, f64) payload", cname));
				log_info!(effector, "setting location to {:.1}, {:.1}", x+loc.0, y+loc.1);
				effector.set_float_data("location-x", x+loc.0);
				effector.set_float_data("location-y", y+loc.1);
				
			} else if ename.starts_with("init ") {
				log_excessive!(effector, "is ignoring init");
			
			} else {
				panic!("component {} can't handle event {}", cname, ename);
			}
			}
			
			drop(dispatched);
			let _ = data.tx.send(effector);
		}
	});
}
