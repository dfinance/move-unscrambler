use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::access::ModuleAccess;
use libra::vm::access::ScriptAccess;
use libra::vm::errors::BinaryLoaderResult;
use crate::deps::map::ModInfo;
use crate::disasm::*;
use crate::types::*;
use super::mod_addr::ExtractModAddr;
use super::*;

type ModHandleAddrs = Vec<ModAddr>;

impl<T> Extract<ModHandleAddrs> for T
where
    T: MoveAccess,
{
    fn extract(&self) -> ModHandleAddrs {
        let self_name = self.name();
        let self_address = &self.address();
        let mut deps = Vec::new();
        for mh in self.module_handles().iter() {
            let name = self.identifier_at(mh.name);
            let address = self.address_identifier_at(mh.address);
            if name != self_name && address != self_address {
                deps.push((address, name).into())
            }
        }
        deps
    }
}

pub fn extract_module_mod_handles(bc: &CompiledModule) -> Vec<ModAddr> {
    let self_name = bc.name();
    let self_address = bc.address();
    let mut deps = Vec::new();
    for mh in bc.as_inner().module_handles.iter() {
        let name = bc.identifier_at(mh.name);
        let address = bc.address_identifier_at(mh.address);
        if name != self_name && address != self_address {
            deps.push((address, name).into())
        }
    }
    deps
}

pub fn extract_script_mod_handles(bc: &CompiledScript) -> Vec<ModAddr> {
    let mut deps = Vec::new();
    for mh in bc.as_inner().module_handles.iter() {
        deps.push(
            (
                bc.address_identifier_at(mh.address),
                bc.identifier_at(mh.name),
            )
                .into(),
        )
    }
    deps
}

pub fn extract_mod_handles(bc: &CompiledMove) -> Vec<ModAddr> {
    match bc {
        CompiledMove::Module(b) => extract_module_mod_handles(b),
        CompiledMove::Script(b) => extract_script_mod_handles(b),
    }
}
