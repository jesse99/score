use std::f64::INFINITY;

/// Used to configure the `Simulation`.
pub struct Config
{
	/// Use 1_000.0 for ms, 1.0 for seconds, 0.1667 for minutes, etc.
	/// Note that larger time units may allow for additional parallelism.
	/// Defaults to micro-second resolution.
	pub time_units: f64,
	
	/// Maximum time to run the simulation for.
	/// Defaults to INFINITY.
	pub max_secs: f64,
	
	/// Number of times to send an "init N" event to active components.
	/// Defaults to 1.
	pub num_init_stages: i32,
	
	/// Random number generator seed.
	/// Defaults to 0 which means seed with entropy. Note that if you want
	/// deterministic results you should use a fixed seed.
	pub seed: u32,
	
	/// Use escape sequences to color code stdout.
	/// Defaults to true.
	pub colorize: bool,

	/// Used when logging to stdout when colorize is on.
	/// Defaults to bright red. See See https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
	/// and https://aweirdimagination.net/2015/02/21/256-color-terminals for information on
	/// color escape codes.
	pub error_escape_code: String,

	/// Used when logging to stdout when colorize is on.
	/// Defaults to red.
	pub warning_escape_code: String,

	/// Used when logging to stdout when colorize is on.
	/// Defaults to bold black.
	pub info_escape_code: String,

	/// Used when logging to stdout when colorize is on.
	/// Defaults to black.
	pub debug_escape_code: String,

	/// Used when logging to stdout when colorize is on.
	/// Defaults to light gray.
	pub excessive_escape_code: String,
}

impl Config
{
	pub fn new() -> Config
	{
		Config {
			time_units: 1_000_000.0,
			max_secs: INFINITY,
			num_init_stages: 1,
			seed: 0,
			colorize: true,
			error_escape_code: "\x1b[31;1m".to_string(),
			warning_escape_code: "\x1b[31m".to_string(),
			info_escape_code: "\x1b[30;1m".to_string(),
			debug_escape_code: "".to_string(),
			excessive_escape_code: "\x1b[1;38;5;244m".to_string(),
		}
	}
}
