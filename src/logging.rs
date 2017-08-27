// Copyright (C) 2017 Jesse Jones
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 3, or (at your option)
// any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program; if not, write to the Free Software Foundation,
// Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301, USA.
#![macro_use]

use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, RustcEncodable)]
pub enum LogLevel
{
	Error = 0,	// update log_levels if this changes
	Warning = 1,
	Info = 2,
	Debug = 3,
	Excessive = 4
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
/// ```
/// use score::*;
///
/// let mut effector = Effector::new();
/// log_info!(effector);				// logs an empty line
/// log_info!(effector, "hello");		// logs a string
/// log_info!(effector, "x = {:?}", 5);	// logs using a format string
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



