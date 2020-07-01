// #![warn(missing_docs)]
#![allow(unused_imports)] // temporarily in R&D state

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;
extern crate clap;

mod cli;
// mod cfg;
mod deps;
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
	trace!("♥️");
}


fn validate_config(mut opts: cli::Opts) -> Result<cli::Opts> {
	use std::fs::read_dir;
	use std::fs::create_dir_all;
	use std::fs::canonicalize;

	opts.input.offline.path = canonicalize(&opts.input.offline.path)?;

	for dep in opts.input.offline.dependencies.iter_mut() {
		*dep = canonicalize(&dep)?;
	}

	create_dir_all(&opts.output.dir)?;
	opts.output.dir = canonicalize(&opts.output.dir)?;

	// TODO: read_dir for output dir & opts.output.force

	{
		use cli::Dialect;

		match (opts.input.online.offline, opts.input.online.ds.as_ref(), opts.input.dialect) {
			(true, Some(_), _) => {
				opts.input.online.ds = None;
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

	let get_dependency: Box<dyn Fn()> = if let Some(ds_uri) = opts.input.online.ds.as_ref() {
		Box::new(move || {
			let cfg = net::NetCfg::new(ds_uri);
			// let bc = net::get(&AccountAddress::random(), "Account".to_owned(), &cfg).unwrap();
			let bc = net::get(
			                  &AccountAddress::from_hex_literal("0x0").unwrap(),
			                  "Account".to_owned(),
			                  &cfg,
			).unwrap();
			debug!("bc: {:?}", bc);
		})
	} else {
		Box::new(|| {
			// TODO: find by args in the fs
		})
	};

	use libra::libra_types::account_address::AccountAddress;
	// net::get(AccountAddress::random(), &"Foo", opts.output)

	let mut deps = deps::offline::OfflineDependencySearch::new_from_opts(&opts.input.offline);

	let mut dg = deps::index::DependencyIndex::default();
	deps.into_load_all()
	   //  .map(|(k, v)| (k, v.map(disasm::deserialize_module)))
	    .for_each(|(k, v)| {
		    match v {

			    Ok(bytes) => dg.insert_file(k, bytes),
		       Err(err) => error!("Unable to load {} : {}", k.as_path().display(), err),
		    }
	    });
	dg.build_deps_links();

	get_dependency();
}


fn get_dependency_net() {}
