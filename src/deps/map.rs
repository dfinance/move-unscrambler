use std::path::{Path, PathBuf};
use std::{hash::Hash, collections::HashMap, borrow::Borrow};
use libra::{vm::CompiledModule, libra_types::account_address::AccountAddress};
use libra::{move_core_types::identifier::IdentStr, vm::access::ModuleAccess};
use crate::disasm;


pub type DependencyMapKey = (AccountAddress, /* name: */ String);


#[derive(Default)]
pub struct DependencyMap {
	// TODO: extend me here
	map: HashMap<DependencyMapKey, DependencyInfo>,
}

impl DependencyMap {
	pub fn mods_at_address(&self,
	                       addr: &AccountAddress)
	                       -> impl Iterator<Item = (&DependencyMapKey, &DependencyInfo)>
	{
		let addr = addr.clone();
		self.map.iter().filter(move |((a, _), _)| *a == addr).map(|a| a)
	}
}

pub struct DependencyInfo {
	source: DependencySource,
	bytecode: CompiledModule,
	dependencies: Vec<DependencyMapKey>,
}

impl DependencyInfo {
	pub fn source(&self) -> &DependencySource { &self.source }
	pub fn name(&self) -> &IdentStr { self.bytecode.name() }
	pub fn address(&self) -> &AccountAddress { self.bytecode.address() }
	pub fn bytecode(&self) -> &CompiledModule { &self.bytecode }
	pub fn dependencies(&self) -> &[DependencyMapKey] { &self.dependencies[..] }
	pub fn dependencies_mut(&mut self) -> &mut [DependencyMapKey] { &mut self.dependencies[..] }
}

#[derive(Debug, Clone)]
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

	pub fn insert_mod_bin<Src>(&mut self, source: Src, bytes: Vec<u8>)
		where Src: Into<DependencySource> {
		let source = source.into();
		debug!("inserting {} bytes by {:?}", bytes.len(), source);
		let m = disasm::deserialize_module(&bytes)
		// TODO: catch error and then add error plug
		.expect("Module can't be deserialized");
		self.insert_mod(source, m);
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
			#[rustfmt::skip]
			debug!("{}.deps: {}", key.1, deps.iter().map(|(a, n)| format!("{}.{}", a, n)).collect::<Vec<_>>().join(", "));
			info.dependencies = deps;
		}
	}
}


pub trait MapAccess<K, V>
	where K: Hash + Eq {
	// type Hasher = std::collections::hash_map::RandomState;

	fn as_map(&self) -> &HashMap<K, V>;
	fn as_map_mut(&mut self) -> &mut HashMap<K, V>;
}


impl MapAccess<DependencyMapKey, DependencyInfo> for DependencyMap {
	fn as_map(&self) -> &HashMap<DependencyMapKey, DependencyInfo> { &self.map }
	fn as_map_mut(&mut self) -> &mut HashMap<DependencyMapKey, DependencyInfo> { &mut self.map }
}

// impl<K, V, T> AsRef<HashMap<K, V>> for T where T: MapAccess<K, V> {
// 	fn as_ref(&self) -> &HashMap<K, V> {}
// }
// impl<K, V> AsRef<HashMap<K, V>> for dyn MapAccess<K, V> {
// 	fn as_ref(&self) -> &HashMap<K, V> {}
// }
