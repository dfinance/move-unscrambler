use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::errors::BinaryLoaderResult;

pub mod disassembler;
mod uni;

pub use uni::*;

pub fn is_script<T: AsRef<[u8]>>(bytecode: T) -> bool {
    deserialize_script(&bytecode).is_ok()
}

pub fn deserialize_module<T: AsRef<[u8]>>(bytecode: T) -> BinaryLoaderResult<CompiledModule> {
    CompiledModule::deserialize(bytecode.as_ref())
}

pub fn deserialize_script<T: AsRef<[u8]>>(bytecode: T) -> BinaryLoaderResult<CompiledScript> {
    CompiledScript::deserialize(bytecode.as_ref())
}

pub fn deserialize<T: AsRef<[u8]>>(bytecode: T) -> BinaryLoaderResult<uni::CompiledMove> {
    uni::CompiledMove::deserialize(bytecode.as_ref())
}
