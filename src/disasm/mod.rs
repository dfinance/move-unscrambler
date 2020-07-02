use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::access::ModuleAccess;
use libra::vm::access::ScriptAccess;
use libra::vm::errors::BinaryLoaderResult;
use crate::deps::map::DependencyMapKey;

pub mod disassembler;
mod uni;
pub use uni::*;


#[derive(Debug)]
pub enum MoveType {
	Script,
	Module,
}


pub fn is_script<T: AsRef<[u8]>>(bytecode: T) -> bool { deserialize_script(&bytecode).is_ok() }

pub fn deserialize_module<T: AsRef<[u8]>>(bytecode: T) -> BinaryLoaderResult<CompiledModule> {
	CompiledModule::deserialize(bytecode.as_ref())
}

pub fn deserialize_script<T: AsRef<[u8]>>(bytecode: T) -> BinaryLoaderResult<CompiledScript> {
	CompiledScript::deserialize(bytecode.as_ref())
}

pub fn deserialize<T: AsRef<[u8]>>(bytecode: T) -> BinaryLoaderResult<uni::CompiledMove> {
	uni::CompiledMove::deserialize(bytecode.as_ref())
}


pub fn deserialize_module_deps(bc: &CompiledModule) -> Vec<DependencyMapKey> {
	let self_name = bc.name();
	let self_address = bc.address();
	let mut deps = Vec::new();
	for mh in bc.as_inner().module_handles.iter() {
		let name = bc.identifier_at(mh.name);
		let address = bc.address_identifier_at(mh.address);
		if name != self_name && address != self_address {
			deps.push((address.to_owned(), name.to_string()))
		}
	}
	deps
}

pub fn deserialize_script_deps(bc: &CompiledScript) -> Vec<DependencyMapKey> {
	let mut deps = Vec::new();
	for mh in bc.as_inner().module_handles.iter() {
		deps.push((bc.address_identifier_at(mh.address).to_owned(), bc.identifier_at(mh.name).to_string()))
	}
	deps
}

pub fn deserialize_deps(bc: &CompiledMove) -> Vec<DependencyMapKey> {
	match bc {
		CompiledMove::Module(b) => deserialize_module_deps(b),
		CompiledMove::Script(b) => deserialize_script_deps(b),
	}
}
