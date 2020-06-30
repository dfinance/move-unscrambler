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

		match (opts.input.offline, opts.input.ds.as_ref(), opts.input.dialect) {
			(true, Some(_), _) => {
				opts.input.ds = None;
				info!("Offline mode requested, so passes node (data-source) URI will be ignored.");
			},
			(false, None, Dialect::Libra) => info!("Offline mode turned on 'cause of node URI is missed."),
			(false, None, Dialect::Dfinance) => {
				// TODO: set up default endpoint URI? Are we should share own node?
				// opts.input.ds = Some("?");
			},
			_ => {},
		}
	}

	trace!("cfg & env validated");
	Ok(opts)
}


fn run(opts: cli::Opts) {
	debug!("args: {:#?}", opts);

	use libra::libra_types::account_address::AccountAddress;
	// net::get(AccountAddress::random(), &"Foo", opts.output)
}
