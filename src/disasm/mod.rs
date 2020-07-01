use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;

pub mod disassembler;


pub fn deserialize_module<T: AsRef<[u8]>>(bytecode: T) -> CompiledModule {
	CompiledModule::deserialize(bytecode.as_ref()).expect("Module can't be deserialized")
}

pub fn deserialize_script<T: AsRef<[u8]>>(bytecode: T) -> CompiledScript {
	CompiledScript::deserialize(bytecode.as_ref()).expect("Script blob can't be deserialized")
}
