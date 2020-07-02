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


fn log_level_from_num(level: u8) -> String {
	let self_level = self_log_level_from_num(level);
	let deps_level = deps_log_level_from_num(level);
	let prefered = std::env::var("RUST_LOG").unwrap_or_default();

	#[rustfmt::skip]
	let self_filters = [
			env!("CARGO_PKG_NAME"),
			&env!("CARGO_PKG_NAME").replace("-", "_")
		].join(&format!("={},", self_level)) + &format!("={}", self_level);

	format!(
	        "{deps_level},{self_filters},{prefered}",
	        self_filters = self_filters,
	        deps_level = deps_level,
	        prefered = prefered,
	)
}

fn self_log_level_from_num(level: u8) -> Level {
	match level {
		0 => Info,
		1 => Debug,
		2 | _ => Trace,
	}
}

fn deps_log_level_from_num(level: u8) -> Level {
	match level {
		0 => Info,
		1 => Info,
		2 => Info,
		3 => Debug,
		4 | _ => Trace,
	}
}


#[cfg(feature = "console_log")]
pub use console::*;
#[cfg(feature = "console_log")]
mod console {
	use super::*;
	extern crate console_log;

	pub type Error = log::SetLoggerError;

	pub fn try_init(level: u8) -> Result<(), self::Error> {
		console_log::init_with_level(log_level_from_num(level))
	}
	pub fn init(level: u8) {
		console_log::init_with_level(log_level_from_num(level)).expect("Unable to initialize logger")
	}
}


#[cfg(any(feature = "env_logger", feature = "pretty_env_logger"))]
pub use logger::*;
use std::path::Path;
#[cfg(any(feature = "env_logger", feature = "pretty_env_logger"))]
mod logger {
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
		env_logger::try_init().map(print_intro_message)?;
		Ok(())
	}

	pub fn init(level: u8) {
		apply_env_log_level(level);
		env_logger::init();
		print_intro_message(());
	}

	fn print_intro_message<T>(v: T) -> T {
		debug!("Debug logging enabled. Happy debugging!");
		v
	}
}


pub fn path_to_string<P: AsRef<Path>>(path: P) -> String {
	use std::env::current_dir;
	let p: &Path = path.as_ref();
	if p.is_absolute() {
		current_dir().and_then(|pb| p.strip_prefix(pb).or_else(|_| Ok(p)))
		             .unwrap()
		             .display()
		             .to_string()
	} else {
		p.display().to_string()
	}
}
