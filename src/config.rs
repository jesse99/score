use glob::Pattern;
use logging::*;
use std::collections::HashMap;
use std::f64::INFINITY;
use std::str::FromStr;

/// Used to configure the `Simulation`.
pub struct Config
{
	/// If set then score will startup a web server to control the simulation
	/// and serve up this file when a browser hits the root. Files relative
	/// to roots directory will also be served up but not files above the
	/// directory.
	pub root: String,
	
	/// The address of the web server (used if root is set). Defaults to
	/// "127.0.0.1:9000".
	pub address: String,
	
	/// Use 1_000.0 for ms, 1.0 for seconds, 0.1667 for minutes, etc.
	/// Note that larger time units may allow for additional parallelism.
	/// Defaults to micro-second resolution.
	pub time_units: f64,
	
	/// Maximum time to run the simulation for. Defaults to INFINITY.
	pub max_secs: f64,
	
	/// Number of times to send an "init N" event to active components.
	/// Defaults to 1.
	pub num_init_stages: i32,
	
	/// Random number generator seed. Defaults to 0 which means seed with
	/// entropy. Note that if you want deterministic results you should
	/// use a fixed seed.
	pub seed: u32,
	
	/// Default log level to use. Defaults to Info.
	pub log_level: LogLevel,

	/// Overrides log_level when the glob `Pattern` matches a `Component`s
	/// name. Defaults to empty. Note that only the first matching pattern
	/// is used.
	pub log_levels: HashMap<Pattern, LogLevel>,
	
	/// Maximum number of characters to use when logging component paths to
	/// stdout. If a path exceeds this then it is truncated from the left and
	/// prepended with an ellipsis. Zero means always use full paths. Defaults
	/// to 20.
	pub max_log_path: usize,
	
	/// Use escape sequences to color code stdout. Defaults to true.
	pub colorize: bool,

	/// Used when logging to stdout when colorize is on. Defaults to bright
	/// red. See See https://en.wikipedia.org/wiki/ANSI_escape_code#Colors
	/// and https://aweirdimagination.net/2015/02/21/256-color-terminals for
	/// information on color escape codes.
	pub error_escape_code: String,

	/// Used when logging to stdout when colorize is on. Defaults to red.
	pub warning_escape_code: String,

	/// Used when logging to stdout when colorize is on. Defaults to bold black.
	pub info_escape_code: String,

	/// Used when logging to stdout when colorize is on. Defaults to black.
	pub debug_escape_code: String,

	/// Used when logging to stdout when colorize is on. Defaults to light gray.
	pub excessive_escape_code: String,
}

impl Config
{
	pub fn new() -> Config
	{
		Config {
			root: "".to_string(),
			address: "127.0.0.1:9000".to_string(),
			time_units: 1_000_000.0,
			max_secs: INFINITY,
			num_init_stages: 1,
			seed: 0,
			log_level: LogLevel::Info,
			log_levels: HashMap::new(),
			max_log_path: 20,
			colorize: true,
			error_escape_code: "\x1b[31;1m".to_string(),
			warning_escape_code: "\x1b[31m".to_string(),
			info_escape_code: "\x1b[30;1m".to_string(),
			debug_escape_code: "".to_string(),
			excessive_escape_code: "\x1b[1;38;5;244m".to_string(),
		}
	}

	/// Helper for parsing command line options. Returns an error if the
	/// string was not able to be parsed.
	pub fn parse_max_secs(&mut self, text: &str) -> Option<&'static str>
	{
		let mut text = text.to_string();
		let units = text.pop().unwrap();
		if let Ok(base) = f64::from_str(&text) {
			match units {	// update time_suffixes if this changes
				's' => {self.max_secs = base; None},
				'm' => {self.max_secs = 60.0*base; None},
				'h' => {self.max_secs = 60.0*60.0*base; None},
				'd' => {self.max_secs = 24.0*60.0*60.0*base; None},
				'w' => {self.max_secs = 7.0*24.0*60.0*60.0*base; None},
				_  => Some("--max-secs should have an s, m, h, d, or w suffix")
			}
		} else {
			Some("--max-secs should have an f64 value followed by a suffix")
		}
	}

	/// Helper for parsing command line options. Returns an error if the
	/// string was not able to be parsed.
	pub fn parse_log_level(&mut self, level: &str) -> Option<&'static str>
	{
		match do_parse_log_level(level) {
			Ok(value) => {
				self.log_level = value;
				None
			},
			Err(message) => Some(message)
		}
	}

	/// Helper for parsing command line options. Returns an error if any of the
	/// strings was not able to be parsed. The strings are assumed to be formatted
	/// as "LEVEL:GLOB".
	pub fn parse_log_levels(&mut self, values: Vec<&str>) -> Option<String>
	{
		for entry in values {
			let parts: Vec<&str> = entry.splitn(2, ':').collect();
			if parts.len() == 2 {
				match do_parse_log_level(parts[0]) {
					Ok(level) => {
						if let Ok(pattern) = Pattern::new(parts[1]) {
							self.log_levels.insert(pattern, level);	// could check for dupes but it's not really an error and could happen if tooling is assembling command lines
						} else {
							return Some(format!("--log={} has a malformed glob", entry));
						}
					},
					Err(message) => {return Some(message.to_string());}
				}
			} else {
				return Some(format!("--log={} should be formatted as LEVEL:GLOB", entry));
			}
		}
		None
	}
}

/// For use in --help messages.
pub fn time_suffixes() -> &'static str
{
	"s, m, h, d, or w"
}

fn do_parse_log_level(level: &str) -> Result<LogLevel, &'static str>
{
	match level {
		"error" => Ok(LogLevel::Error),
		"warning" => Ok(LogLevel::Warning),
		"info" => Ok(LogLevel::Info),
		"debug" => Ok(LogLevel::Debug),
		"excessive" => Ok(LogLevel::Excessive),
		_ => Err("--log-level should be error, warning, info, debug, or excessive"),
	}
}
