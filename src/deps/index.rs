use std::path::{Path, PathBuf};
use std::collections::HashMap;
use libra::{vm::CompiledModule, libra_types::account_address::AccountAddress};
use libra::vm::access::ModuleAccess;
use crate::disasm;


// TODO: make/fill DependencyIndex from offline::OfflineDependencySearch


pub type IndexKey = (AccountAddress, /* name: */ String);


#[derive(Default)]
pub struct DependencyIndex {
	// key: (AccountAddress, name:String)
	// val: (opt<fs-path>, bc, meta: { script/mod,  })
	map: HashMap<IndexKey, DependencyInfo>,
}

pub struct DependencyInfo {
	bytecode: CompiledModule,
	source: DependencySource,
	dependencies: Vec<IndexKey>,
}

impl DependencyInfo {
	pub fn address(&self) -> &AccountAddress { self.bytecode.address() }
}

pub enum DependencySource {
	Fs(PathBuf),
	Net,
	None,
}

impl<T: Into<PathBuf>> From<T> for DependencySource {
	fn from(p: T) -> Self { DependencySource::Fs(p.into()) }
}


impl DependencyIndex {
	pub fn insert(&mut self, address: AccountAddress, name: String, info: DependencyInfo) {
		self.map.insert((address, name), info);
	}

	pub fn insert_file(&mut self, file_path: PathBuf, bytes: Vec<u8>) {
		trace!("insert: {}", file_path.as_path().display());
		let m = disasm::deserialize_module(&bytes);
		self.insert_mod(file_path, m);
	}

	pub fn insert_mod<Src>(&mut self, source: Src, bytecode: CompiledModule)
		where Src: Into<DependencySource> {
		let name = bytecode.name().to_string();
		let address = bytecode.address().to_owned();
		trace!("\t\t 0x{}.{}", address, name);

		let info = DependencyInfo { bytecode,
		                            source: source.into(),
		                            dependencies: Default::default() };
		self.insert(address, name, info);
	}


	pub fn build_deps_links(&mut self) {
		//
	}
}
