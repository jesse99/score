use std;
use std::fmt;

/// `Component`s are the building blocks of a `Simulation`. They are arranged into
/// a tree and use a thread to respond to `Event`s which execute at some particular
/// `Time`. Note that, in general, all state managed within components should be
/// stored in the `Store`. This makes it possible to use GUI tools to see what is
/// happening within components and even more importantly allows the Simulation to
/// perform speculative execution of components.
///
/// Typically type safe structs are defined for components with the aid of [`OutPort`],
/// [`InPort`], [`IntValue`], etc.
pub struct Component
{
	/// The name of the component. Note that, in general, these are not unique.
	pub name: String,
	
	/// ID for the component's parent. The root component will return NO_COMPONENT.
	pub parent: ComponentID,
	
	pub children: Vec<ComponentID>,
}

/// To make lifetime management easier components are referenced using a small
/// integer instead of a rust reference.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ComponentID(pub usize);

/// The id of the root component.
pub const NO_COMPONENT: ComponentID = ComponentID(std::usize::MAX);

impl fmt::Display for ComponentID
{
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result
	{
		write!(formatter, "{}", self.0)
	}
}

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
