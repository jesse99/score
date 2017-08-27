About score
=============
score is a general purpose discrete event simulator written in rust. The key types are:
* *Simulation* is responsible for coordinating the execution of the simulation.
* *Component*s are used to define the structure of the simulation. *Active* components
have a thread used to handle events.
* *Event*s are named messages sheduled to be delivered to a component at a specific time.
Events may have am optional payload (which must satisfy the Any and Send traits).
* The *Store* is where components persist state. (Using the store allows state to be
viewed and changed using GUI tools like [sdebug](https://github.com/jesse99/sdebug) and
allows side effects to be carefully managed.)
* Components use an *Effector* to make changes. Components can use a an effector to log,
change their own state within the store, and schedule events to be sent to arbitrary
components.

score's goals include:
* It should be flexible enough to simulate pretty much any sort of discrete event simulation.
* It should be efficient and take advantage of multiple cores. Currently all components that
have a scheduled event at the same time process the event in parallel. It should also be
possible to leverage effectors to do speculative parallel execution.
* Side effects should be carefully controlled, In particular the *only* way for a component
to affect another component should be via an event.
* Simulation execution should be deterministic across different runs and across platforms.
* It should support off line analysis of simulation results. (This isn't in yet but shouldn't
be hard to implement).
* There should be a GUI tool to analyze simulations as they run.

versions:
* 0.1.0 - added an embedded REST server to support GUIs like sdevug
* 0.0.1 - initial release

Online documentation for the released version can be found on [crates.io](https://crates.io/crates/score).
