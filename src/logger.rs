use env::*;
use time::*;

pub trait Logger
{
	fn log_error(&self, topic: &str, message: &str);
	fn log_warning(&self, topic: &str, message: &str);
	fn log_info(&self, topic: &str, message: &str);
	fn log_debug(&self, topic: &str, message: &str);
	fn log_excessive(&self, topic: &str, message: &str);
}

// TODO: We'll need a logger to write to a file or something
// (the store doesn't seem like a great place because we need
// to record stuff with a fair amount of structure and because
// logging should not require a mutable reference).
impl Logger for Env
{
	fn log_error(&self, topic: &str, message: &str)
	{
		log_to_console(self.time, topic, message, &error_escape(), &end_escape());
	}

	fn log_warning(&self, topic: &str, message: &str)
	{
		log_to_console(self.time, topic, message, &warning_escape(), &end_escape());
	}

	fn log_info(&self, topic: &str, message: &str)
	{
		log_to_console(self.time, topic, message, &info_escape(), &end_escape());
	}

	fn log_debug(&self, topic: &str, message: &str)
	{
		log_to_console(self.time, topic, message, &debug_escape(), &end_escape());
	}

	fn log_excessive(&self, topic: &str, message: &str)
	{
		log_to_console(self.time, topic, message, &excessive_escape(), &end_escape());
	}
}

fn log_to_console(time: Time, topic: &str, message: &str, begin: &str, end: &str)
{
	// TODO: need to format time somehow
	let t = time.0;
	print!("{}{}   {}   {}{}\n", begin, t, topic, message, end);
}

// TODO: escapes should be some sort of config option
// See https://en.wikipedia.org/wiki/ANSI_escape_code#Colors and https://aweirdimagination.net/2015/02/21/256-color-terminals/
fn error_escape() -> String
{
	"\x1b[31;1m".to_string()	// bright red
}

fn warning_escape() -> String
{
	"\x1b[31m".to_string()		// red
}

fn info_escape() -> String
{
	"\x1b[30;1m".to_string()	// bold black
}

fn debug_escape() -> String
{
	"".to_string()				// black
}

fn excessive_escape() -> String
{
	"\x1b[1;38;5;244m".to_string()	// light gray
}

fn end_escape() -> String
{
	"\x1b[0m".to_string()
}