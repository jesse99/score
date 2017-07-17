//! Functions to be used with `Component` threads.
use component::*;
use effector::*;
use event::*;
use logging::*;
use sim_state::*;
use store::*;


/// Typically `Component` threads will use this to cut down on the boiler plate involved in
/// processing dispatched `Event`s. Note that this will panic if it tries to process an
/// event that doesn't have an associated code block.
///
/// # Examples
///
/// ```
/// use score::*;
///
/// fn my_thread(data: ThreadData)
/// {
/// 	thread::spawn(move || {
/// 		process_events!(data, event, state, effector,
/// 			"init 0" => {
/// 				// Use the effector to change the simulation state.
/// 				let event = Event::new("timer");
/// 				effector.schedule_after_secs(event, data.id, 1.0);
/// 			},
/// 			"timer" => {
/// 				// Typically you'd re-schedule the timer here,
/// 				log_info!(effector, "timer fired!");
/// 			}
/// 		);
/// 	});
/// }
/// ```
#[macro_export]
macro_rules! process_events
{
	($data:expr, $event:ident, $state:ident, $effector:ident, $($name:pat => $code:expr),+) => ({
		for ($event, $state) in $data.rx.iter() {
			let mut $effector = Effector::new();
			{
				let ename = &$event.name;
				match ename.as_ref() {
					$($name => $code)+
					
					_ => {
						let cname = &(*$state.components).get($data.id).name;
						panic!("component {} can't handle event {}", cname, ename);
					}
				}
			}
			
			drop($state);	// we need to do this before the send to ensure that our references are dropped before the Simulator processes the send
			let _ = $data.tx.send($effector);
		}
	});
}

/// Helper for components that want to maintain an (x, y) position. This will handle the
/// following events:
/// * "set-location" - Sets the data using an (f64, f64) event payload.
/// * "offset-location" - Add values to the data using an (f64, f64) event payload.
/// Note that this will panic if sent an event not listed above.
///
/// If origin is missing GUIs should assume zero, if size is missing GUIs should assume
/// 1.0, if units are missing then GUIs should assume meters.
pub fn handle_location_event(id: ComponentID, state: &SimState, event: &Event, effector: &mut Effector)
{
	let cname = &(*state.components).get(id).name;
	let ename = &event.name;
	if ename == "set-location" {
		let loc = event.expect_payload::<(f64, f64)>(&format!("component {} set-location should have an (f64, f64) payload", cname));
		log_info!(effector, "setting location to {:.1}, {:.1}", loc.0, loc.1);
		effector.set_float_data("display-location-x", loc.0);
		effector.set_float_data("display-location-y", loc.1);
		
	} else if ename == "offset-location" {
		let path = state.components.path(id);
		let x = state.store.get_float_data(&(path.clone() + ".display-location-x"));
		let y = state.store.get_float_data(&(path + ".display-location-y"));

		let loc = event.expect_payload::<(f64, f64)>(&format!("component {} offset-location should have an (f64, f64) payload", cname));
		log_info!(effector, "setting location to {:.1}, {:.1}", x+loc.0, y+loc.1);
		effector.set_float_data("display-location-x", x+loc.0);
		effector.set_float_data("display-location-y", y+loc.1);
		
	} else {
		panic!("handle_location_event doesn't know how to handle {}", ename);
	}
}
