use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::access::ModuleAccess;
use crate::deps::index::IndexKey;

pub mod disassembler;


pub fn deserialize_module<T: AsRef<[u8]>>(bytecode: T) -> CompiledModule {
	CompiledModule::deserialize(bytecode.as_ref()).expect("Module can't be deserialized")
}

pub fn deserialize_module_handles(bc: &CompiledModule) -> Vec<IndexKey> {
	// bytecode.module_handle_at(i)
	let mut deps = Vec::new();
	for mh in bc.as_inner().module_handles.iter() {
		deps.push((bc.address_identifier_at(mh.address).to_owned(), bc.identifier_at(mh.name).to_string()))
	}
	deps
}


pub fn deserialize_script<T: AsRef<[u8]>>(bytecode: T) -> CompiledScript {
	CompiledScript::deserialize(bytecode.as_ref()).expect("Script can't be deserialized")
}
