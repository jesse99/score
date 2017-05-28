/// To better support deterministic execution time is stored
/// using 64-bit integers. By default the units are in micro-
/// seconds.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Time(pub i64);	// unsigned would give us more range, but makes it awkward to use times in the past
