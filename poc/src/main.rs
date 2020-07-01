// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0
#![allow(dead_code)] // temporarily in R&D state
#![allow(unused_imports)] // temporarily in R&D state
#![allow(unused_variables)] // temporarily in R&D state
#![forbid(unsafe_code)]

mod disassembler;
use bytecode_source_map::{mapping::SourceMapping,
                      source_map::SourceMap,
                      utils::{remap_owned_loc_to_loc, source_map_from_file, OwnedLoc}};
use disassembler::{Disassembler, DisassemblerOptions};
// use move_coverage::coverage_map::CoverageMap;
use move_ir_types::location::Spanned;
use std::{fs, path::Path};
use structopt::StructOpt;
use vm::{IndexKind,
     file_format::{CompiledModule, CompiledScript, ModuleHandleIndex, TableIndex, StructDefinitionIndex,
                   FunctionDefinitionIndex, FunctionDefinition, Bytecode, StructHandle, FieldHandleIndex,
                   StructHandleIndex},
     access::ModuleAccess};

#[derive(Debug, StructOpt)]
#[structopt(name = "Move Bytecode Disassembler",
            about = "Print a human-readable version of Move bytecode (.mv files)")]
struct Args {
	/// Skip printing of private functions.
	#[structopt(long = "skip-private")]
	pub skip_private: bool,

	/// Do not print the disassembled bytecodes of each function.
	#[structopt(long = "skip-code")]
	pub skip_code: bool,

	/// Do not print locals of each function.
	#[structopt(long = "skip-locals")]
	pub skip_locals: bool,

	/// Do not print the basic blocks of each function.
	#[structopt(long = "skip-basic-blocks")]
	pub skip_basic_blocks: bool,

	/// Treat input file as a script (default is to treat file as a module)
	#[structopt(short = "s", long = "script")]
	pub is_script: bool,

	/// The path to the bytecode file to disassemble; let's call it file.mv. We assume that two
	/// other files reside under the same directory: a source map file.mvsm (possibly) and the Move
	/// source code file.move.
	#[structopt(short = "b", long = "bytecode")]
	pub bytecode_file_path: String,
	//  /// (Optional) Path to a coverage file for the VM in order to print trace information in the
	//  /// disassembled output.
	//  #[structopt(short = "c", long = "move-coverage-path")]
	//  pub code_coverage_path: Option<String>,
	#[structopt(short = "d", long = "dep")]
	pub dep_bytecode_file_path: Option<String>,
}

fn main() {
	let args = Args::from_args();


	let mut doc_main = Document::default();
	describe_to_doc(&args.bytecode_file_path, args.is_script, &mut doc_main);


	if let Some(path) = args.dep_bytecode_file_path {
		let mut doc_sub = Document::default();
		describe_to_doc(&path, false, &mut doc_sub);

		// println!("---");
		// println!("mod: {:?}", doc_sub);

		// merge

		for (key, value) in doc_sub.table.drain() {
			doc_main.table.insert(key, value);
		}
	}

	// TODO: render document
	println!("scr: {:?}", doc_main);
	render_doc(&doc_main, "./out");
}

fn render_doc(doc: &Document, out_dir: &str) {
	let out_dir = Path::new(out_dir);
	fs::create_dir_all(out_dir).ok();

	let mut res = String::new();

	let main_key = &doc.main.as_ref().unwrap();

	// head
	if doc.is_script {
		res.push_str("# Script \n\n");
	} else {
		let (addr, name) = &main_key;
		res.push_str(&format!("# Module {} \n\n", name));
		res.push_str(&format!("Address: `0x{}` \n\n", addr));
	}

	let main = doc.get_main_section().unwrap();
	let r = render_sec(&main_key.0, &main_key.1, main, out_dir);
	res.push_str(&r);

	let other = doc.table.iter().filter(|(key, _value)| key != main_key);
	for (key, value) in other {
		// println!("::: {:?}", key);
		let (addr, name) = &key;
		let r = render_sec(addr, name, value, out_dir);
		res.push_str(&r);
	}


	// save
	let doc_path = out_dir.to_owned().join("main.md");
	fs::write(doc_path, res).ok();
}

fn render_sec(addr: &AccountAddress, name: &str, sec: &Section, out_dir: &Path) -> String {
	let mut res = String::new();
	let sec = match sec {
		Section::Unresolved => return res,
		Section::Module(m) => m,
	};

	for (fname, fsig) in sec.fn_names.iter() {
		// TODO: by sig: if let Some(sec) = sec.functions.get(fsig) {
		if let Some(sec) = sec.functions.get(fname) {
			println!(":::: {} {} {} {}", fname, fsig, sec.name, sec.signature);

			if fname == DEF_SCRIPT_FN_NAME {
				res.push_str(&format!("## Function `{}` \n\n", "main"));
			} else {
				res.push_str(&format!("## Function `{}` \n\n", fname));
				res.push_str(&format!("Signature: `{}` \n\n", fsig));
				res.push_str(&format!("Address: `0x{}:{}:{}` \n\n", addr, name, fname));
			}


			// meta:
			let meta = &sec.meta;

			if meta.knoleges.len() > 0 {
				res.push_str(&format!("### What it can do\n\n"));

				for s in meta.knoleges.iter() {
					res.push_str(&format!("- {}\n", s));
				}
				res.push_str("\n\n");
			}


			res.push_str(&format!("\n\n- - -\n\n"));

			if let Some(asm) = &sec.asm {
				res.push_str(&format!("### Dissassembled \n\n"));
				res.push_str(&format!("```\n{}\n```\n\n", asm));

				if meta.cf_used {
					res.push_str(&format!("Contains control-flow ops\n\n"));
					if let Some(cfg) = &meta.cfg {
						// save
						let san_fname = if fname == DEF_SCRIPT_FN_NAME {
							"main"
						} else {
							fname
						};
						let svg_fname = format!("{}.asm.svg", san_fname);
						let doc_path = out_dir.to_owned().join(&svg_fname);
						fs::write(doc_path, cfg.to_string()).ok();

						// res.push_str(&cfg.to_string());
						res.push_str(&format!("![{} control flow graph]({})", fname, svg_fname));
						res.push_str("\n\n");
					}
				}
			}
		}
	}

	res
}

fn describe_to_doc(bytecode_file_path: &str, is_script: bool, mut doc: &mut Document) {
	let mv_bytecode_extension = "mv";
	let source_map_extension = "mvsm";

	let source_path = Path::new(&bytecode_file_path);
	let extension = source_path.extension()
	                           .expect("Missing file extension for bytecode file");
	if extension != mv_bytecode_extension {
		println!(
		         "Bad source file extension {:?}; expected {}",
		         extension, mv_bytecode_extension
		);
		std::process::exit(1);
	}

	let bytecode_bytes = fs::read(&bytecode_file_path).expect("Unable to read bytecode file");

	//  let source_path = Path::new(&bytecode_file_path).with_extension(move_extension);
	//  let source = fs::read_to_string(&source_path).ok();
	let source_map = source_map_from_file::<OwnedLoc>(
        &Path::new(&bytecode_file_path).with_extension(source_map_extension),
    )
    .map(remap_owned_loc_to_loc);

	let mut disassembler_options = DisassemblerOptions::new();
	//  disassembler_options.print_code = !args.skip_code;
	//  disassembler_options.only_public = args.skip_private;
	//  disassembler_options.print_basic_blocks = !args.skip_basic_blocks;
	//  disassembler_options.print_locals = !args.skip_locals;
	disassembler_options.print_code = true;
	disassembler_options.only_public = false;
	disassembler_options.print_basic_blocks = true;
	disassembler_options.print_locals = true;


	let no_loc = Spanned::unsafe_no_loc(()).loc;
	let source_mapping = if is_script {
		println!("# Transaction (script)");
		doc.is_script = true;
		let compiled_script =
			CompiledScript::deserialize(&bytecode_bytes).expect("Script blob can't be deserialized");
		source_map.or_else(|_| SourceMap::dummy_from_script(&compiled_script, no_loc))
		          .and_then(|source_map| Ok(SourceMapping::new_from_script(source_map, compiled_script)))
		          .expect("Unable to build source mapping for compiled script")
	} else {
		doc.is_script = false;
		let compiled_module =
			CompiledModule::deserialize(&bytecode_bytes).expect("Module blob can't be deserialized");
		source_map.or_else(|_| SourceMap::dummy_from_module(&compiled_module, no_loc))
		          .and_then(|source_map| Ok(SourceMapping::new(source_map, compiled_module)))
		          .expect("Unable to build source mapping for compiled module")
	};

	extract_meta(&source_mapping, &mut doc);
	doc.create_main_section();

	let disassembler = Disassembler::new(source_mapping, disassembler_options);

	extract_fns(&disassembler, &mut doc);

	// let dissassemble_string = disassembler.disassemble().expect("Unable to dissassemble");
	// println!("{}", dissassemble_string);
	// println!("- - -");
}


use std::collections::HashMap;
use libra_types::account_address::AccountAddress;
use bytecode_verifier::control_flow_graph::VMControlFlowGraph;

#[derive(Default, Debug)]
struct Document {
	is_script: bool,
	main: Option<(AccountAddress, String)>,
	table: HashMap<(AccountAddress, String), Section>,
}

impl Document {
	fn set_main<S: AsRef<str>>(&mut self, address: &AccountAddress, name: S) {
		self.main = Some((address.clone(), name.as_ref().to_string()));
	}

	fn create_main_section(&mut self) {
		debug_assert!(self.main.is_some());
		let doc_main = self.table.get_mut(self.main.as_ref().unwrap());
		debug_assert!(doc_main.is_some());
		let doc_main = doc_main.unwrap();

		match doc_main {
			Section::Unresolved => {
				*doc_main = Section::Module(ModuleSec { ..Default::default() });
			},
			Section::Module { .. } => {},
		};
	}

	fn get_main_section_mut(&mut self) -> Option<&mut Section> {
		debug_assert!(self.main.is_some());
		self.table.get_mut(self.main.as_ref().unwrap())
	}

	fn get_main_section(&self) -> Option<&Section> {
		debug_assert!(self.main.is_some());
		self.table.get(self.main.as_ref().unwrap())
	}

	fn add_section<S: AsRef<str>>(&mut self, address: &AccountAddress, name: S) {
		let key = (address.clone(), name.as_ref().to_string());
		if !self.table.contains_key(&key) {
			self.table.insert(key, Section::Unresolved);
		}
	}
}

#[derive(Debug)]
enum Section {
	Unresolved,

	Module(ModuleSec),
}

#[derive(Default, Debug)]
struct ModuleSec {
	// structs: ...
	fn_names: HashMap<String, String>,
	functions: HashMap<String, FunctionSec>,
}

#[derive(Default, Debug)]
struct FunctionSec {
	// is_public: bool,
	// is_native: bool,
	name: String,
	signature: String,
	def: FunctionDefinition,
	meta: FunctionMeta,
	// body: (),
	asm: Option<String>,
}

#[derive(Default, Debug)]
struct FunctionMeta {
	cf_used: bool,
	cfg: Option<vis::CtrlFlowGraph>,
	knoleges: Vec<String>,
}


static DEF_SCRIPT_FN_NAME: &str = "<SELF>";
static DEF_SCRIPT_ADDRESS: [u8; 16] = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF_u128.to_ne_bytes();


fn extract_meta<Loc: Clone + Eq>(map: &SourceMapping<Loc>, doc: &mut Document) {
	let bc = &map.bytecode;
	// module_handles
	let self_addr = bc.address();

	debug_assert_eq!(
	                 self_addr == &AccountAddress::new(DEF_SCRIPT_ADDRESS),
	                 doc.is_script
	);

	// println!(" self addr: {}", self_addr);
	{
		debug_assert_eq!(
		                 bc.kind_count(IndexKind::ModuleHandle),
		                 bc.as_inner().module_handles.len()
		);

		for (i, mh) in bc.as_inner().module_handles.iter().enumerate() {
			let addr = bc.address_identifier_at(mh.address);
			let name = bc.identifier_at(mh.name).as_str();

			println!("ref mod {} addr: {}.{}", i, addr, name);

			doc.add_section(&addr, name);

			if doc.main.is_none() && self_addr == addr {
				if doc.is_script && name == DEF_SCRIPT_FN_NAME {
					doc.set_main(&addr, name);
				} else {
					let (addr, name) = map.source_map.module_name_opt.as_ref().unwrap();
					doc.set_main(addr, name.as_str());
				}
			}
		}
	}


	println!("- - -");
}

fn extract_fns<Loc: Clone + Eq>(mut dis: &Disassembler<Loc>, mut doc: &mut Document) {
	let map = &dis.source_mapper;
	let bc = &map.bytecode;
	let is_script = doc.is_script;


	// signature
	// asm
	// knoleges:
	// 	- cals: ->
	// 	- usages: addresses
	// 	- returns

	let mut functions = Vec::new();
	if is_script {
		let fd = &bc.function_defs()[0];
		let f = extract_fn(0, fd, &mut dis, &mut doc);
		debug_assert_eq!(f.name, DEF_SCRIPT_FN_NAME);
		functions.push(f);
	} else {
		for (i, fd) in bc.function_defs().iter().enumerate() {
			let f = extract_fn(i, fd, &mut dis, &mut doc);
			functions.push(f);
		}
	}

	let main_sec = doc.get_main_section_mut();
	debug_assert!(main_sec.is_some());
	let (fnn, fns) = match main_sec.unwrap() {
		Section::Unresolved => unreachable!(),
		Section::Module(ModuleSec { fn_names, functions }) => (fn_names, functions),
	};

	for f in functions {
		fnn.entry(f.name.to_owned())
		   .or_insert_with(|| f.signature.to_owned());
		fns.entry(f.name.to_owned()).or_insert_with(move || f);
	}
}

fn extract_fn<Loc: Clone + Eq>(i: usize,
                               fd: &FunctionDefinition,
                               dis: &Disassembler<Loc>,
                               doc: &mut Document)
                               -> FunctionSec
{
	println!("fd: {:?}", fd);
	let id = FunctionDefinitionIndex(i as TableIndex);
	let (name, signature) = dis.disassemble_function_def_signature(id.clone()).unwrap();
	println!("name: {}", name);
	println!("sig: {}", signature);

	let asm = dis.disassemble_function_def(id).ok();
	// if let Some(asm) = &asm { println!("ASM: {}", asm) }

	let mut meta = FunctionMeta::default();


	if let Some(code) = &fd.code {
		use bytecode_verifier::control_flow_graph::ControlFlowGraph;

		// describe cfg:
		let cfg = VMControlFlowGraph::new(&code.code);
		meta.cf_used = cfg.num_blocks() > 1;
		meta.cfg = vis::create_railroad(&fd);

		let zblock = {
			let zid = cfg.entry_block_id();
			let start = cfg.block_start(zid.to_owned());
			let end = cfg.block_end(zid.to_owned());
			start..=end
		};
		// for (block_number, block_id) in cfg.blocks().iter().enumerate() {
		// 	let i = *block_id as usize + block_number;
		// 	let b = format!("B{}:", block_number);
		// 	// instrs.insert(*block_id as usize + block_number, format!("B{}:", block_number));
		// 	println!("cfg: {}: {}", i, b);
		// 	// render_bc(&code.code[..]);
		// }

		// let bytecode = dis.disassemble_bytecode(id).unwrap();
		// println!("bc: {:#?}", bytecode);

		let bc = &dis.source_mapper.bytecode;

		let mod_handle_sig = |idx: &ModuleHandleIndex| {
			let mh = bc.module_handle_at(*idx);
			let mod_addr = bc.address_identifier_at(mh.address).to_string();
			let mod_name = bc.identifier_at(mh.name).as_str();
			format!("0x{}:{}", mod_addr, mod_name)
		};

		let struct_handle_sig = |shidx: &StructHandleIndex| {
			let sh = bc.struct_handle_at(*shidx);
			let mod_sig = mod_handle_sig(&sh.module);
			let name = bc.identifier_at(sh.name);
			let kind = if sh.is_nominal_resource {
				"resource"
			} else {
				"structure"
			};
			format!("{} {}:{}", kind, mod_sig, name)
		};

		let struct_field_sig = |fidx: &FieldHandleIndex| {
			let fh = bc.field_handle_at(*fidx);
			let sd = bc.struct_def_at(fh.owner);
			let sh_sig = struct_handle_sig(&sd.struct_handle);
			// let sh_sig = struct_handle_sig(&sh);
			// let mod_sig = mod_handle_sig(&sh.module);
			// let name = bc.identifier_at(sh.name);
			// let kind = if sh.is_nominal_resource { "resource" } else { "structure" };
			format!("{} field [{}] (todo: name)", sh_sig, fh.field)
		};

		for (iop, op) in code.code.iter().enumerate() {
			let pre_can_call = if meta.cf_used && !zblock.contains(&(iop as u16)) {
				"can call"
			} else {
				"__calls__"
			};
			let pre_can_modf = if meta.cf_used && !zblock.contains(&(iop as u16)) {
				"can modify"
			} else {
				"__modifies__"
			};

			match op {
				Bytecode::Call(idx) => {
					let fh = bc.function_handle_at(*idx);
					let nh = bc.identifier_at(fh.name).as_str();
					let mh = mod_handle_sig(&fh.module);
					// println!("CALL: {}.{}", mh, nh);
					meta.knoleges
					    .push(format!("{} function `0x{}:{}`", pre_can_call, mh, nh));
				},

				Bytecode::CallGeneric(idx) => {
					let fi = bc.function_instantiation_at(*idx);
					let fh = bc.function_handle_at(fi.handle);
					let nh = bc.identifier_at(fh.name).as_str();
					let mh = mod_handle_sig(&fh.module);

					// TODO: with T-params
					let tys = bc.signature_at(fi.type_parameters);
					// TODO: render T-params (tys.0 = [SignatureToken])
					// dis.struct_type_info(, tys.0);

					// println!("CALL: {}:{}", mh, nh);
					meta.knoleges
					    .push(format!("{} function {}:{}", pre_can_call, mh, nh));
				},

				Bytecode::MutBorrowGlobal(sidx) => {
					let sd = bc.struct_def_at(*sidx);
					// let styi = dis.struct_type_info(*sidx, SIG);
					// let sh = bc.struct_handle_at(sd.struct_handle);
					let sh_sig = struct_handle_sig(&sd.struct_handle);
					meta.knoleges.push(format!("{} {}", pre_can_modf, sh_sig));
				},
				Bytecode::MutBorrowField(fidx) => {
					let f_sig = struct_field_sig(fidx);
					meta.knoleges.push(format!("{} {}", pre_can_modf, f_sig));
				},
				_ => {},
			}
		}
	}

	let f = FunctionSec { name,
	                      signature,
	                      def: fd.clone(),
	                      meta,
	                      //  body: (),
	                      asm };
	f
}

// fn render_bc(bc: &[Bytecode]) { }


mod vis {
	use railroad::*;
	use bytecode_verifier::control_flow_graph::{VMControlFlowGraph, ControlFlowGraph};
	use vm::file_format::FunctionDefinition;

	pub type CtrlFlowGraph = Diagram<Sequence>;

	pub fn create_railroad(fd: &FunctionDefinition) -> Option<CtrlFlowGraph> {
		if let Some(code) = &fd.code {
			let cfg = VMControlFlowGraph::new(&code.code);
			if cfg.num_blocks() > 1 {
				let mut seq = Sequence::default();
				seq.push(Box::new(SimpleStart));
				// .push(Box::new(NonTerminal::new("var data".to_owned())))

				// last successor:
				let last_block = cfg.blocks()[cfg.num_blocks() as usize - 1];
				// let last_successors = cfg.successors(last_block);
				// println!("last_successors: {:?}", last_successors);

				// add entry block:
				{
					let block_id = cfg.entry_block_id();
					let start = cfg.block_start(block_id.to_owned());
					let end = cfg.block_end(block_id.to_owned());
					let b = format!("B{}: {}..{}", block_id, start, end);
					// println!("+: {}", b);
					seq.push(Box::new(Terminal::new(b)));

					let successors = cfg.successors(block_id.to_owned());
					if successors.len() > 1 {
						let mut choice = Choice::new(Vec::new());
						for block_id in successors {
							let mut sub = Sequence::default();
							let start = cfg.block_start(block_id.to_owned());
							let end = cfg.block_end(block_id.to_owned());
							let b = format!("B{}: {}..{}", block_id, start, end);
							sub.push(Box::new(Terminal::new(b)));

							let successors = cfg.successors(block_id.to_owned());
							if successors.len() == 0 {
								sub.push(Box::new(End));
							}

							if successors.len() > 1 {
							} else if successors[0] != last_block {
								let block_id = successors[0];
								let start = cfg.block_start(block_id.to_owned());
								let end = cfg.block_end(block_id.to_owned());
								let b = format!("B{}: {}..{}", block_id, start, end);
								sub.push(Box::new(Terminal::new(b)));
							}

							choice.push(Box::new(sub));
						}
						seq.push(Box::new(choice));

						{
							let start = cfg.block_start(last_block.to_owned());
							let end = cfg.block_end(last_block.to_owned());
							let b = format!("B{}: {}..{}", last_block, start, end);
							seq.push(Box::new(Terminal::new(b)));
						}
					}
				}

				let blocks = cfg.blocks();
				for (i, id) in blocks.iter().enumerate() {
					println!(
					         "\t block_start ({} : {}) : {}",
					         i,
					         id,
					         cfg.block_start(id.to_owned())
					);
					println!("\t block_end ({} : {}) : {}", i, id, cfg.block_end(id.to_owned()));
					println!(
					         "\t successors ({} : {}) : {:?}",
					         i,
					         id,
					         cfg.successors(id.to_owned())
					);
				}

				seq.push(Box::new(SimpleEnd));
				let mut dia = Diagram::new(seq);
				dia.add_element(svg::Element::new("style").set("type", "text/css")
				                                          .text(DEFAULT_CSS));
				// println!("{}", dia.to_string());
				Some(dia)
			} else {
				None
			}
		} else {
			None
		}
	}
}
// fn attach_fns_asm<Loc: Clone + Eq>(dis: &Disassembler<Loc>, doc: &mut Document) {
// 	let map = &dis.source_mapper;
// 	let bc = &map.bytecode;
// 	let is_script = doc.is_script;
// 	let main_sec = doc.get_main_section_mut();
// 	debug_assert!(main_sec.is_some());

// 	let (fnn, fns) = match main_sec.unwrap() {
// 		Section::Unresolved => unreachable!(),
// 		Section::Module(ModuleSec { fn_names, functions }) => (fn_names, functions),
// 	};

// 	if is_script {
// 		let fd = &bc.function_defs()[0];
// 	} else {
// 		for (i, fd) in bc.function_defs().iter().enumerate() {
// 			println!("fd: {:?}", fd);
// 			// signature
// 			// get fn by signature
// 			// asm

// 			let id = FunctionDefinitionIndex(i as TableIndex);
// 			let (name, sig) = dis.disassemble_function_def_signature(id.clone()).unwrap();
// 			println!("name: {}", name);
// 			println!("sig: {}", sig);

// 			let asm = dis.disassemble_function_def(FunctionDefinitionIndex(i as TableIndex));
// 			if let Ok(asm) = asm {
// 				println!("ASM: {}", asm);

// 				let key = sig;
// 				fns.entry(key).and_modify(|f| {
// 					              f.asm = Some(asm);
// 				              });
// 			}
// 		}
// 	}
// }


fn _describe_source_mapping_mod(m: &SourceMapping<move_ir_types::location::Loc>) {
	let name_opt = m.source_map.module_name_opt.as_ref();
	let name = name_opt.map(|(addr, n)| format!("{} :: {}", addr, n.to_string()));
	// let name = name_opt.map(|(addr, n)| format!("{}.{}", addr.short_str(), n.to_string()));

	// TODO: None for script
	println!("# Module `{}`", name.unwrap());

	_describe_module(&m.bytecode);
}


fn _describe_module(m: &CompiledModule) {
	// println!("CodeDefinition: {}", m.kind_count(IndexKind::CodeDefinition));
	// println!("FieldDefinition: {}", m.kind_count(IndexKind::FieldDefinition));
	// println!("TypeParameter: {}", m.kind_count(IndexKind::TypeParameter));
	// println!("MemberCount: {}", m.kind_count(IndexKind::MemberCount));

	// let m = m.as_inner();

	println!("ModuleHandle: {}", m.kind_count(IndexKind::ModuleHandle));
	println!("StructHandle: {}", m.kind_count(IndexKind::StructHandle));
	println!("FunctionHandle: {}", m.kind_count(IndexKind::FunctionHandle));
	println!("FieldHandle: {}", m.kind_count(IndexKind::FieldHandle));
	println!("Signature: {}", m.kind_count(IndexKind::Signature));
	println!(
	         "AddressIdentifier: {}",
	         m.kind_count(IndexKind::AddressIdentifier)
	);
	println!("Identifier: {}", m.kind_count(IndexKind::Identifier));

	// PROTO
	println!("----");
	// for h in &m.module_handles {
	// 	// let ident = Identifier::new(h.name).unwrap();
	// 	println!("mod: {:?} : {}", h.name, h.address);
	// }

	// FNs
	for fd in m.function_defs() {
		println!("fd: {:?}", fd);
	}

	// let function_defs: Vec<String> =
	// 	(0..m.bytecode.function_defs().len()).map(|i| self.disassemble_function_def(FunctionDefinitionIndex(i as TableIndex)))
	// 	                                     .collect::<Result<Vec<String>>>()?;

	println!("----");
}
