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
use cli::InputType;
use disasm::MoveType;
use disasm::CompiledMove;
use deps::map::DependencyMap;
use deps::map::DependencyMapKey;


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

	opts.input.path = canonicalize(&opts.input.path)?;

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
	let (input_type, input, input_deps) = read_input(&opts);
	let mut deps = read_offline_deps(&opts);
	// TODO: add online deps-resolver into the (or wrap it) DependencyMap (deps)

	// TODO: to be continued.
}

fn read_input(opts: &cli::Opts) -> (MoveType, CompiledMove, Vec<DependencyMapKey>) {
	let bytes = std::fs::read(&opts.input.path).expect("Unable to read input bytecode");

	let source_type = match opts.input.offline.kind {
		InputType::Script => MoveType::Script,
		InputType::Module => MoveType::Module,
		InputType::Auto => {
			let t = if disasm::is_script(&bytes) {
				MoveType::Script
			} else {
				MoveType::Module
			};
			info!("input type auto detected: {:?}", t);
			t
		},
	};
	let input = disasm::CompiledMove::deserialize(&bytes).expect("Input bytecode can't be deserialized");
	let deps = disasm::deserialize_deps(&input);
	#[rustfmt::skip]
	debug!("input.deps: {}", deps.iter().map(|(a, n)|format!("{}.{}",a,n)).collect::<Vec<_>>().join(", "));

	(source_type, input, deps)
}

fn read_offline_deps(opts: &cli::Opts) -> DependencyMap {
	let mut index = DependencyMap::default();
	let deps = deps::offline::OfflineDependencySearch::new_from_opts(&opts.input.offline);
	deps.into_load_all().for_each(|(k, v)| {
		                    match v {
			                    Ok(bytes) => index.insert_file(k, bytes),
		                       Err(err) => error!("Unable to load {} : {}", k.as_path().display(), err),
		                    }
	                    });
	index.build_deps_links();
	index
}
