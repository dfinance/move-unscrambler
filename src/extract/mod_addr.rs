use crate::types::*;
use crate::deps::map::ModInfo;
use crate::disasm::MoveAccess;
use crate::disasm::CompiledMoveRef;
use crate::types::FnAddr;
use super::*;
use libra::vm::file_format::StructHandle;

impl Extract<ModAddr> for CompiledModule {
	fn extract(&self) -> ModAddr { (self.address(), self.name()).into_mod_addr() }
}

impl Extract<ModAddr> for ModInfo {
	fn extract(&self) -> ModAddr { (self.address(), self.name()).into_mod_addr() }
}

impl<T: MoveAccess> Extract<ModAddr> for T {
	fn extract(&self) -> ModAddr { (self.address(), self.name()).into_mod_addr() }
}

impl Extract<ModAddr> for FnAddr {
	fn extract(&self) -> ModAddr { self.addr().to_owned() }
}

impl ExtractRef<ModAddr> for FnAddr {
	fn extract_ref(&self) -> &ModAddr { self.addr() }
}


pub trait ExtractModAddr {
	fn mod_addr(&self) -> ModAddr;
}

impl<T> ExtractModAddr for T where T: Extract<ModAddr> {
	fn mod_addr(&self) -> ModAddr { self.extract() }
}


impl ExtractFrom<ModAddr, StructHandle> for CompiledModule {
	fn extract_from(&self, sh: &StructHandle) -> ModAddr {
		let mh = self.module_handle_at(sh.module);
		let name = self.identifier_at(sh.name);
		let address = self.address_identifier_at(mh.address);
		ModAddr::new(address.to_owned(), name)
	}
}
