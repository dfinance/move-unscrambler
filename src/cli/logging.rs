use clap::Clap;
use log::{Level, Level::*};


/// Logging configuration.
#[derive(Clap, Debug)]
pub struct Opts {
	/// A level of verbosity, and can be used multiple times.
	/// Possible valuable maximum are three `v`.
	#[clap(short, long, parse(from_occurrences))]
	pub verbose: u8,
}


fn log_level_from_num(level: u8) -> Level {
	match level {
		0 => Info,
		1 => Debug,
		2 | _ => Trace,
		// TODO: 3 => enable trace-level for network deps such as tokio, hyper.
		// TODO: 4 => enable trace-level for low-level deps such as mio.
	}
}


#[cfg(feature = "console_log")]
pub use _console_log::*;
#[cfg(feature = "console_log")]
mod _console_log {
	use super::*;
	extern crate console_log;

	pub type Error = log::SetLoggerError;

	pub fn try_init(level: u8) -> Result<(), self::Error> { console_log::init_with_level(log_level_from_num(level)) }
	pub fn init(level: u8) { console_log::init_with_level(log_level_from_num(level)).expect("Unable to initialize logger") }
}


#[cfg(any(feature = "env_logger", feature = "pretty_env_logger"))]
pub use _env_logger::*;
#[cfg(any(feature = "env_logger", feature = "pretty_env_logger"))]
mod _env_logger {
	// TODO: init pretty_env_logger when interactive tty only
	use super::*;
	#[cfg(all(feature = "env_logger", not(feature = "pretty_env_logger")))]
	extern crate env_logger;
	#[cfg(feature = "pretty_env_logger")]
	extern crate pretty_env_logger as env_logger;

	pub type Error = log::SetLoggerError;

	fn apply_env_log_level(level: u8) { std::env::set_var("RUST_LOG", log_level_from_num(level).to_string()); }

	pub fn try_init(level: u8) -> Result<(), self::Error> {
		apply_env_log_level(level);
		env_logger::try_init()?;
		Ok(())
	}

	pub fn init(level: u8) {
		apply_env_log_level(level);
		env_logger::init();
	}
}
