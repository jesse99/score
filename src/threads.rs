//! Some component threads that are generally useful.
use component::*;
use effector::*;
use event::*;
use std::sync::mpsc;
use std::thread;

/// A thread to use for a top-level component that maintains a location
/// within "location-x" and "location-y" float data. To change the location
/// send a "set-location" event with an (f64, f64) payload.
pub fn locatable_thread(id: ComponentID, rx_event: mpsc::Receiver<DispatchedEvent>,
	tx_reply: mpsc::Sender<Effector>)
{
	thread::spawn(move || {
		for dispatched in rx_event {
			let mut effector = Effector::new();

			let cname = &(*dispatched.components).get(id).name;
			let ename = &dispatched.event.name;
			if ename == "set-location" {
				let loc = dispatched.expect_payload::<(f64, f64)>(&format!("component {} set-location should have an (f64, f64) payload", cname));
				effector.log(LogLevel::Debug, &format!("setting location to {:.1}, {:.1}", loc.0, loc.1));
				
			} else if ename.starts_with("init ") {
				effector.log(LogLevel::Excessive, "is ignoring init"); 
			
			} else {
				panic!("component {} can't handle event {}", cname, ename);
			}

			let _ = tx_reply.send(effector);
		}
	});
}
