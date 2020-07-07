use libra::vm::file_format::{FunctionHandle, CompiledScript, FunctionHandleIndex};
use libra::vm::CompiledModule;
use libra::vm::access::ModuleAccess;
use libra::vm::access::ScriptAccess;
use libra::vm::errors::BinaryLoaderResult;
use crate::deps::map::ModInfo;
use crate::disasm::*;
use crate::types::*;
use super::mod_addr::ExtractModAddr;
use super::*;

impl ExtractFrom<FnAddr, FunctionHandle> for CompiledModule {
    fn extract_from(&self, other: &FunctionHandle) -> FnAddr {
        let bc = self;
        let fh = other;
        let module = {
            let mh = bc.module_handle_at(fh.module);
            let name = bc.identifier_at(mh.name);
            let address = bc.address_identifier_at(mh.address);
            ModAddr::new(address.to_owned(), name)
        };
        (module, bc.identifier_at(fh.name)).into()
    }
}

impl ExtractFrom<FnAddr, FunctionHandleIndex> for CompiledModule {
    fn extract_from(&self, other: &FunctionHandleIndex) -> FnAddr {
        let fh = self.function_handle_at(*other);
        self.extract_from(fh)
    }
}
