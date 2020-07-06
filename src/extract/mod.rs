use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::access::ModuleAccess;
use libra::vm::access::ScriptAccess;
use libra::vm::errors::BinaryLoaderResult;
use crate::deps::map::ModInfo;
use crate::disasm::MoveAccess;
use crate::disasm::CompiledMoveRef;
use crate::disasm::*;
use crate::types::*;


pub mod mod_addr;
pub mod mod_handles;
pub mod fn_handles;
pub mod struct_map;

pub mod prelude {
	pub use super::{Extract, ExtractRef, ExtractMut};
	pub use super::mod_handles::*;
	pub use super::fn_handles::*;
}


pub trait Extract<T> {
	fn extract(&self) -> T;
}

pub trait ExtractRef<T> {
	fn extract_ref(&self) -> &T;
}

pub trait ExtractMut<T> {
	fn extract_mut(&mut self) -> &mut T;
}

pub trait ExtractWith<T, K> {
	fn extract_with(&self, other: K) -> T;
}


