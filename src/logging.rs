#![macro_use]

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, RustcEncodable)]
pub enum LogLevel
{
	Error,	// update log_levels if this changes
	Warning,
	Info,
	Debug,
	Excessive
}

/// For use in --help messages.
pub fn log_levels() -> &'static str
{
	"error, warning, info, debug, or excessive"
}

impl LogLevel
{
	pub fn with_str(text: &str) -> Option<LogLevel>
	{
		let text = text.to_lowercase();
		match text.to_lowercase().as_ref() {
			"error" => Some(LogLevel::Error),
			"warning" => Some(LogLevel::Warning),
			"info" => Some(LogLevel::Info),
			"debug" => Some(LogLevel::Debug),
			"excessive" => Some(LogLevel::Excessive),
			_ => None,
		}
	}
}

impl fmt::Display for LogLevel {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		// Write strictly the first element into the supplied output
		// stream: `f`. Returns `fmt::Result` which indicates whether the
		// operation succeeded or failed. Note that `write!` uses syntax which
		// is very similar to `println!`.
		
		match self {
		&LogLevel::Error => write!(f, "{}", "error"),
		&LogLevel::Warning => write!(f, "{}", "warning"),
		&LogLevel::Info => write!(f, "{}", "info"),
		&LogLevel::Debug => write!(f, "{}", "debug"),
		&LogLevel::Excessive => write!(f, "{}", "excessive"),
	}
	}
}

/// Generic macro that calls the `Effector` log method. More often you'll use one of
/// the other macros like log_info!.
#[macro_export]
macro_rules! log_at
{
	// Typically it is nice to skip formatting if the log message wouldn't appear.
	// But in our case log messages are normally always persisted.
	($effector:expr, $l:expr) => ($effector.log(level, ""));
	($effector:expr, $l:expr, $msg:expr) => ($effector.log(level, $msg));
	($effector:expr, $l:expr, $fmt:expr, $($arg:tt)*) => ($effector.log(level, &format!($fmt, $($arg)*)));
}

#[macro_export]
macro_rules! log_error
{
	($effector:expr) => ($effector.log(LogLevel::Error, ""));
	($effector:expr, $msg:expr) => ($effector.log(LogLevel::Error, $msg));
	($effector:expr, $fmt:expr, $($arg:tt)*) => ($effector.log(LogLevel::Error, &format!($fmt, $($arg)*)));
}

#[macro_export]
macro_rules! log_warning
{
	($effector:expr) => ($effector.log(LogLevel::Warning, ""));
	($effector:expr, $msg:expr) => ($effector.log(LogLevel::Warning, $msg));
	($effector:expr, $fmt:expr, $($arg:tt)*) => ($effector.log(LogLevel::Warning, &format!($fmt, $($arg)*)));
}

/// # Examples
///
/// ```rust
/// #[macro_use]
/// extern crate score;
///
/// log_info!(effector);						// logs an empty line
/// log_info!(effector, "hello");			// logs a string
/// log_info!(effector, "x = {:?}", x);	// logs using a format string
/// ```
#[macro_export]
macro_rules! log_info
{
	($effector:expr) => ($effector.log(LogLevel::Info, ""));
	($effector:expr, $msg:expr) => ($effector.log(LogLevel::Info, $msg));
	($effector:expr, $fmt:expr, $($arg:tt)*) => ($effector.log(LogLevel::Info, &format!($fmt, $($arg)*)));
}

#[macro_export]
macro_rules! log_debug
{
	($effector:expr) => ($effector.log(LogLevel::Debug, ""));
	($effector:expr, $msg:expr) => ($effector.log(LogLevel::Debug, $msg));
	($effector:expr, $fmt:expr, $($arg:tt)*) => ($effector.log(LogLevel::Debug, &format!($fmt, $($arg)*)));
}

#[macro_export]
macro_rules! log_excessive
{
	($effector:expr) => ($effector.log(LogLevel::Excessive, ""));
	($effector:expr, $msg:expr) => ($effector.log(LogLevel::Excessive, $msg));
	($effector:expr, $fmt:expr, $($arg:tt)*) => ($effector.log(LogLevel::Excessive, &format!($fmt, $($arg)*)));
}



