// #![warn(missing_docs)]
#![allow(unused_imports)] // temporarily in R&D state

#[macro_use]
extern crate log;
extern crate clap;

mod cli;
// mod cfg;
mod disasm;
mod analyze;
mod output;

fn main() {
	let opts = cli::try_init().and_then(|opts| validate_config(opts));
	match opts {
		Ok(opts) => run(opts),
		Err(err) => eprintln!("{}", err),
	}
	trace!("shutdown");
}


fn validate_config(opts: cli::Opts) -> Result<cli::Opts, String> {
	trace!("cfg & env validated (TODO)");
	Ok(opts)
}


fn run(opts: cli::Opts) {
	debug!("args: {:#?}", opts);
}
