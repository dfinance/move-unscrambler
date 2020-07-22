// #![warn(missing_docs)]
#![allow(unused_imports)] // temporarily in R&D state
#![allow(unused_variables)] // temporarily in R&D state
#![allow(unused_mut)] // temporarily in R&D state
#![allow(dead_code)] // temporarily in R&D state

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;
extern crate clap;

mod cli;
// mod cfg;
mod deps;
mod types;
mod disasm;
mod extract;
mod data;
mod analyse;
mod output;

use anyhow::{bail, format_err, Result};
use cli::InputType;
use disasm::{default_script_fn_address, CompiledMove};
use deps::map::ModMap;
use deps::map::{DependencyMap, AsMap};
use deps::resolver::UnresolvedMap;
use types::MoveType;
use types::{FnAddr, ModAddr};
use extract::prelude::*;
use output::{
    ctx::{Ctx, IntoContext},
    utils::path_to_string,
};
use data::{DbRoot, Db};

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

        if opts.input.online.offline {
            info!("Offline mode requested, so passes node (data-source) URI will be ignored");
            for ds in opts.input.online.ds.drain(..) {
                debug!("\t\t- {}", ds);
            }
        } else {
            match (opts.input.online.ds.len(), opts.input.dialect) {
                (0, Dialect::Libra) => {
                    info!("Offline mode turned on 'cause of node URI is missed.")
                }
                (0, Dialect::Dfinance) => {
                    info!("Offline mode turned on 'cause of node URI is missed.")
                    // TODO: set up default endpoint URI? Are we should share own node?
                    // opts.input.ds = Some("?");
                }
                _ => {}
            }
        }
    }

    trace!("cfg & env validated");
    Ok(opts)
}

fn run(opts: cli::Opts) {
    let (input_type, input, input_deps) = read_input(&opts);
    let detected_input_type = input.extract();
    let (deps, missed_deps) = read_deps(&opts.input, &input_deps);

    if input_type != detected_input_type {
        warn!(
            "Requested source type ({:?}) is not equal detected ({:?}).",
            input_type, detected_input_type
        );
    }

    // extract structs, resources
    let struct_map: StructMap = {
        let deps_iter = deps
            .iter()
            .map(|(_, dep)| extract_struct_map(dep.bytecode()).into_iter())
            .flatten();
        if let CompiledMove::Module(compiled_mod) = &input {
            extract_struct_map(&compiled_mod)
                .into_iter()
                .chain(deps_iter)
                .collect()
        } else {
            deps_iter.collect()
        }
    };

    // extract functions
    let fn_map: FnMap = {
        let deps_iter = deps
            .iter()
            .map(|(_, dep)| extract_functions(dep.bytecode()).into_iter())
            .flatten();
        match &input {
            CompiledMove::Module(bc) => extract_functions(&bc)
                .into_iter()
                .chain(deps_iter)
                .collect(),

            CompiledMove::Script(bc) => extract_script_main_fn(&bc)
                .into_iter()
                .chain(deps_iter)
                .collect(),
        }
    };

    // get entry point(s)
    let entry_points: Vec<FnAddr> = {
        let root_mod: ModAddr = input.extract();
        fn_map
            .iter()
            .filter(|(k, v)| k.addr() == &root_mod && v.is_public)
            .map(|(k, _)| k)
            .cloned()
            .collect()
    };

    for (addr, fi) in fn_map.iter() {
        debug!("FN: {} => {:#?}", addr, fi);
    }

    for ep in &entry_points {
        debug!("entry point: {:x}", ep);
    }

    let mut db = Db {
        dialect: opts.input.dialect,
        root: DbRoot {
            bc: input,
            kind: detected_input_type,
            entry_points,
        },
        modules: deps,
        functions: fn_map,
        structs: struct_map,
        missed_modules: missed_deps.iter().map(|(addr, _)| addr).cloned().collect(),
    };

    // TODO: filter out unused entries

    // TODO: analyze

    // render
    let ctx = db.into_context();
    output::tmt::render(&opts.output, ctx)
        .map_err(|err| error!("{}", err))
        .ok();
}

fn read_deps(opts: &cli::Input, input_deps: &[ModAddr]) -> (ModMap, UnresolvedMap<ModAddr>) {
    let deps_local = read_offline_deps(&opts);

    // create online deps-resolver(s), resolve all deps in DependencyMap recursively, then destroy.
    let (deps, missed_deps) = if !opts.online.offline {
        use deps::online::*;
        let searchers = opts
            .online
            .ds
            .iter()
            .cloned()
            .map(deps::online::OnlineDependencySearch::new);
        let mut resolver = deps::resolver::DependencyResolverMap::new(deps_local);
        searchers.for_each(|s| resolver.add_searcher(s));
        resolver.prefetch_deps(&input_deps);
        resolver.prefetch_deps_recursively();
        resolver.split()
    } else {
        (deps_local, Default::default())
    };

    for (dep, err) in missed_deps.iter() {
        warn!("{:#x} not found, Err: '{}'", dep, err);
    }

    (deps, missed_deps)
}

fn read_input(opts: &cli::Opts) -> (MoveType, CompiledMove, Vec<ModAddr>) {
    let mut bytes = std::fs::read(&opts.input.offline.path).expect("Unable to read input bytecode");
    compat::adapt(&mut bytes);

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
        }
    };

    let root =
        disasm::CompiledMove::deserialize(&bytes).expect("Input bytecode can't be deserialized");
    let root_deps = extract_mod_handles(&root);

    #[rustfmt::skip]
	debug!("input.deps: ({}) [{}]", root_deps.len(), root_deps.iter().map(|m| format!("{:#x}", m)).collect::<Vec<_>>().join(", "));

    (source_type, root, root_deps)
}

fn read_offline_deps(opts: &cli::Input) -> ModMap {
    let mut index = ModMap::default();
    let deps = deps::offline::OfflineDependencySearch::new_from_opts(&opts.offline);
    deps.into_load_all().for_each(|(k, v)| match v {
        Ok(mut bytes) => {
            compat::adapt(&mut bytes);
            index.insert_file(k, bytes)
        }
        Err(err) => error!("Unable to load {} : {}", path_to_string(&k), err),
    });
    //
    // here can add some more
    // e.g. cache or std
    //
    index.build_deps_links();
    index
}
