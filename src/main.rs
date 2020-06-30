// #![warn(missing_docs)]
#![allow(unused_imports)] // temporarily in R&D state

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;
extern crate clap;

mod cli;
// mod cfg;
mod disasm;
mod analyze;
mod output;

use anyhow::{bail, format_err, Result};


fn main() {
	let opts = cli::try_init().and_then(|opts| validate_config(opts));
	match opts {
		Ok(opts) => run(opts),
		Err(err) => eprintln!("{0:}\n{0:?}", err),
	}
	trace!("shutdown");
}


fn validate_config(mut opts: cli::Opts) -> Result<cli::Opts> {
	use std::fs::read_dir;
	use std::fs::create_dir_all;
	use std::fs::canonicalize;

	opts.input.path = canonicalize(&opts.input.path)?;

	for dep in opts.input.dependencies.iter_mut() {
		*dep = canonicalize(&dep)?;
	}

	create_dir_all(&opts.output.dir)?;
	opts.output.dir = canonicalize(&opts.output.dir)?;

	// TODO: read_dir for output dir & opts.output.force

	trace!("cfg & env validated (TODO)");
	Ok(opts)
}


fn run(opts: cli::Opts) {
	debug!("args: {:#?}", opts);
}
