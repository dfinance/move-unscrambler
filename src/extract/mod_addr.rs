use crate::types::*;
use crate::deps::map::ModInfo;
use crate::disasm::MoveAccess;
use crate::disasm::CompiledMoveRef;
use crate::types::FnAddr;
use super::*;

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
