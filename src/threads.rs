//! Functions to be used with `Component` threads.
use component::*;
use effector::*;
use event::*;
use logging::*;
use sim_state::*;
use store::*;

/// `Component` threads that want to support changing the location of their component
/// can call this to handle "set-location" or "offset-location" `Event`s. The event
/// should have an (f64, f64) payload and "location-x" and "location-y" float data is
/// updated.
pub fn handle_location_event(id: ComponentID, state: &SimState, event: &Event, effector: &mut Effector) -> bool
{
	let cname = &(*state.components).get(id).name;
	let ename = &event.name;
	if ename == "init 0" {
		effector.set_description("location-x", "The x coordinate of the component.");
		effector.set_description("location-y", "The y coordinate of the component.");
		true
		
	} else if ename == "set-location" {
		let loc = event.expect_payload::<(f64, f64)>(&format!("component {} set-location should have an (f64, f64) payload", cname));
		log_info!(effector, "setting location to {:.1}, {:.1}", loc.0, loc.1);
		effector.set_float_data("location-x", loc.0);
		effector.set_float_data("location-y", loc.1);
		true
		
	} else if ename == "offset-location" {
		let path = state.components.path(id);
		let x = state.store.get_float_data(&(path.clone() + ".location-x"));
		let y = state.store.get_float_data(&(path + ".location-y"));

		let loc = event.expect_payload::<(f64, f64)>(&format!("component {} offset-location should have an (f64, f64) payload", cname));
		log_info!(effector, "setting location to {:.1}, {:.1}", x+loc.0, y+loc.1);
		effector.set_float_data("location-x", x+loc.0);
		effector.set_float_data("location-y", y+loc.1);
		true
		
	} else {
		false
	}
}
