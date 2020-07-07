use std::path::{Path, PathBuf};
use std::{hash::Hash, collections::HashMap, borrow::Borrow};
use libra::{vm::CompiledModule, libra_types::account_address::AccountAddress};
use libra::{move_core_types::identifier::IdentStr, vm::access::ModuleAccess};
use crate::disasm;
use crate::output::utils::path_to_string;
use crate::types::{IntoModAddr, ModAddr};
use crate::extract::prelude::*;
use super::DependencySource;

pub trait AsMap<K, V>
where
    K: Hash + Eq,
{
    fn as_map(&self) -> &HashMap<K, V>;
    fn as_map_mut(&mut self) -> &mut HashMap<K, V>;
}

impl AsMap<ModAddr, ModInfo> for ModMap {
    fn as_map(&self) -> &HashMap<ModAddr, ModInfo> {
        self
    }
    fn as_map_mut(&mut self) -> &mut HashMap<ModAddr, ModInfo> {
        self
    }
}

pub type ModMap = HashMap<ModAddr, ModInfo>;

pub trait DependencyMap {
    fn insert_file(&mut self, file_path: PathBuf, bytes: Vec<u8>);
    fn insert_mod_bin<Src>(&mut self, source: Src, bytes: Vec<u8>)
    where
        Src: Into<DependencySource>;
    fn insert_mod<Src>(&mut self, source: Src, bytecode: CompiledModule)
    where
        Src: Into<DependencySource>;
    fn build_deps_links(&mut self);

    fn build_deps_for(info: &mut ModInfo);

    // TODO: selectors:
    // fn mods_at_address(&self, addr: &AccountAddress) -> impl Iterator<Item = (&ModAddr, &DependencyInfo)>;
}

impl DependencyMap for ModMap {
    fn insert_file(&mut self, file_path: PathBuf, bytes: Vec<u8>) {
        debug!("inserting file {}", path_to_string(&file_path));
        let m = disasm::deserialize_module(&bytes)
            // TODO: catch error and then add error plug
            .expect("Module can't be deserialized");
        self.insert_mod(file_path, m);
    }

    fn insert_mod_bin<Src>(&mut self, source: Src, bytes: Vec<u8>)
    where
        Src: Into<DependencySource>,
    {
        let source = source.into();
        debug!("inserting {} bytes by {:?}", bytes.len(), source);
        let m = disasm::deserialize_module(&bytes)
            // TODO: catch error and then add error plug
            .expect("Module can't be deserialized");
        self.insert_mod(source, m);
    }

    fn insert_mod<Src>(&mut self, source: Src, bytecode: CompiledModule)
    where
        Src: Into<DependencySource>,
    {
        let addr = (bytecode.address(), bytecode.name()).into();
        let mut info = ModInfo {
            bytecode,
            source: source.into(),
            dependencies: Default::default(),
        };

        Self::build_deps_for(&mut info);
        debug!(
            "inserted {:#X} with ({}) deps: [{}]",
            addr,
            info.dependencies.len(),
            info.dependencies
                .iter()
                .map(|m| format!("{:#X}", m))
                .collect::<Vec<_>>()
                .join(", ")
        );
        self.insert(addr, info);
    }

    fn build_deps_for(info: &mut ModInfo) {
        if info.dependencies.is_empty() {
            let deps = extract_module_mod_handles(info.bytecode());
            info.dependencies.extend(deps);
        }
    }

    fn build_deps_links(&mut self) {
        for (_, mut info) in self.iter_mut() {
            Self::build_deps_for(&mut info);
        }
    }

    // fn mods_at_address(&self, addr: &AccountAddress) -> impl Iterator<Item = (&ModAddr, &DependencyInfo)> {
    // 	let addr = addr.clone();
    // 	self.map.iter().filter(move |(m, _)| m.addr() == &addr)
    // }
}

pub struct ModInfo {
    source: DependencySource,
    bytecode: CompiledModule,
    dependencies: Vec<ModAddr>,
}

impl ModInfo {
    pub fn source(&self) -> &DependencySource {
        &self.source
    }
    pub fn name(&self) -> &IdentStr {
        self.bytecode.name()
    }
    pub fn address(&self) -> &AccountAddress {
        self.bytecode.address()
    }
    pub fn mod_addr(&self) -> ModAddr {
        (self.address(), self.name()).into_mod_addr()
    }
    pub fn bytecode(&self) -> &CompiledModule {
        &self.bytecode
    }
    pub fn dependencies(&self) -> &[ModAddr] {
        &self.dependencies[..]
    }
    pub fn dependencies_mut(&mut self) -> &mut [ModAddr] {
        &mut self.dependencies[..]
    }
}
