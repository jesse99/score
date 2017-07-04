#![macro_use]

#[derive(Debug, PartialEq, PartialOrd)]
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



