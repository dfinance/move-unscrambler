use std::path::{Path, PathBuf};
use std::collections::HashMap;
use libra::{vm::CompiledModule, libra_types::account_address::AccountAddress};
use libra::vm::access::ModuleAccess;
use crate::disasm;


// TODO: make/fill DependencyIndex from offline::OfflineDependencySearch


pub type DependencyMapKey = (AccountAddress, /* name: */ String);


#[derive(Default)]
pub struct DependencyMap {
	// key: (AccountAddress, name:String)
	// val: (opt<fs-path>, bc, meta: { script/mod,  })
	map: HashMap<DependencyMapKey, DependencyInfo>,
}

pub struct DependencyInfo {
	source: DependencySource,
	bytecode: CompiledModule,
	dependencies: Vec<DependencyMapKey>,
}

impl DependencyInfo {
	pub fn source(&self) -> &DependencySource { &self.source }
	pub fn address(&self) -> &AccountAddress { self.bytecode.address() }
	pub fn bytecode(&self) -> &CompiledModule { &self.bytecode }
	pub fn dependencies(&self) -> &[DependencyMapKey] { &self.dependencies[..] }
}

pub enum DependencySource {
	Fs(PathBuf),
	Net,
	None,
}

impl<T: Into<PathBuf>> From<T> for DependencySource {
	fn from(p: T) -> Self { DependencySource::Fs(p.into()) }
}


impl DependencyMap {
	pub fn insert(&mut self, address: AccountAddress, name: String, info: DependencyInfo) {
		self.map.insert((address, name), info);
	}

	pub fn insert_file(&mut self, file_path: PathBuf, bytes: Vec<u8>) {
		let m = disasm::deserialize_module(&bytes)
		// TODO: catch error and then add error plug
		.expect("Module can't be deserialized");
		self.insert_mod(file_path, m);
	}

	pub fn insert_mod<Src>(&mut self, source: Src, bytecode: CompiledModule)
		where Src: Into<DependencySource> {
		let name = bytecode.name().to_string();
		let address = bytecode.address().to_owned();
		debug!("add 0x{}.{}", address, name);

		let info = DependencyInfo { bytecode,
		                            source: source.into(),
		                            dependencies: Default::default() };
		self.insert(address, name, info);
	}


	pub fn build_deps_links(&mut self) {
		for (key, info) in self.map.iter_mut() {
			let deps = disasm::deserialize_module_deps(info.bytecode());
			debug!("{}.deps: {}", key.1, deps.iter() .map(|(a, n)| format!("{}.{}", a, n)) .collect::<Vec<_>>().join(", "));
			info.dependencies = deps;
		}
	}
}
