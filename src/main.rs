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
	let opts = validate_config(cli::init());

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

	{
		use cli::Dialect;

		match (opts.input.online, opts.input.ds.as_ref(), opts.input.dialect) {
			(true, None, Dialect::Libra) => {
				warn!("Online mode requested but node URI missed, so online mode will be disabled.");
				opts.input.online = false;
			},
			(true, None, Dialect::Dfinance) => {
				// TODO: set up default endpoint URI? Are we should share own node?
			},
			(false, Some(_), _) => {
				warn!("Online mode disabled because haven't been requested with --online.");
			},
			_ => {},
		}
	}

	trace!("cfg & env validated (TODO)");
	Ok(opts)
}


fn run(opts: cli::Opts) {
	debug!("args: {:#?}", opts);

	use libra::libra_types::account_address::AccountAddress;
	// net::get(AccountAddress::random(), &"Foo", opts.output)
}
