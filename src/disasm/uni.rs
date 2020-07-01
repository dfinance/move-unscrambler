use std::ops::{Deref, DerefMut};
use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::errors::BinaryLoaderResult;
use super::{deserialize_module, deserialize_script};

pub enum CompiledMove {
	Script(CompiledScript),
	Module(CompiledModule),
}

pub enum CompiledMoveRef<'a> {
	Script(&'a CompiledScript),
	Module(&'a CompiledModule),
}

impl From<CompiledScript> for CompiledMove {
	fn from(b: CompiledScript) -> Self { CompiledMove::Script(b) }
}

impl From<CompiledModule> for CompiledMove {
	fn from(b: CompiledModule) -> Self { CompiledMove::Module(b) }
}

impl CompiledMove {
	pub fn deserialize(bytecode: &[u8]) -> BinaryLoaderResult<Self> {
		match deserialize_script(&bytecode) {
			Ok(b) => Ok(CompiledMove::Script(b)),
			Err(_) => deserialize_module(&bytecode).map(CompiledMove::Module),
		}
	}
}

impl<'a> CompiledMove {
	pub fn as_ref(&'a self) -> CompiledMoveRef<'a> {
		match self {
			CompiledMove::Script(b) => CompiledMoveRef::Script(b),
			CompiledMove::Module(b) => CompiledMoveRef::Module(b),
		}
	}
}

impl<'a> CompiledMoveRef<'a> {
	pub fn to_owned(self) -> CompiledMove {
		match self {
			CompiledMoveRef::Script(b) => CompiledMove::Script(b.to_owned()),
			CompiledMoveRef::Module(b) => CompiledMove::Module(b.to_owned()),
		}
	}
}


mod access {
	use super::*;
	use libra::libra_types::account_address::AccountAddress;
	use libra::move_core_types::language_storage::ModuleId;
	use libra::move_core_types::identifier::Identifier;
	use libra::move_core_types::identifier::IdentStr;
	use libra::libra_types::access_path::AccessPath;
	use libra::vm::access::{ScriptAccess, ModuleAccess};
	use libra::vm::file_format::{ModuleHandle, ModuleHandleIndex, StructDefInstantiationIndex,
	                         FunctionInstantiationIndex, FieldInstantiationIndex, FunctionInstantiation,
	                         FieldInstantiation, Signature, SignatureIndex, IdentifierIndex,
	                         AddressIdentifierIndex, ConstantPoolIndex, Constant, StructDefinition,
	                         StructDefinitionIndex, FieldHandleIndex, FieldHandle, StructDefInstantiation,
	                         FunctionHandleIndex, FunctionHandle, StructHandleIndex, StructHandle,
	                         FunctionDefinitionIndex, FunctionDefinition};


	/// Represents accessors for a compiled move binary.
	pub trait MoveAccess: Sync {
		fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle;
		fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle;
		fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle;
		fn function_instantiation_at(&self, idx: FunctionInstantiationIndex) -> &FunctionInstantiation;
		fn signature_at(&self, idx: SignatureIndex) -> &Signature;
		fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr;
		fn address_identifier_at(&self, idx: AddressIdentifierIndex) -> &AccountAddress;
		fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant;
		fn module_handles(&self) -> &[ModuleHandle];
		fn struct_handles(&self) -> &[StructHandle];
		fn function_handles(&self) -> &[FunctionHandle];
		fn function_instantiations(&self) -> &[FunctionInstantiation];
		fn signatures(&self) -> &[Signature];
		fn constant_pool(&self) -> &[Constant];
		fn identifiers(&self) -> &[Identifier];
		fn address_identifiers(&self) -> &[AccountAddress];
	}

	impl MoveAccess for CompiledMove {
		fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
			match self {
				CompiledMove::Script(b) => b.module_handle_at(idx),
				CompiledMove::Module(b) => b.module_handle_at(idx),
			}
		}

		fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
			match self {
				CompiledMove::Script(b) => b.struct_handle_at(idx),
				CompiledMove::Module(b) => b.struct_handle_at(idx),
			}
		}

		fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
			match self {
				CompiledMove::Script(b) => b.function_handle_at(idx),
				CompiledMove::Module(b) => b.function_handle_at(idx),
			}
		}

		fn function_instantiation_at(&self, idx: FunctionInstantiationIndex) -> &FunctionInstantiation {
			match self {
				CompiledMove::Script(b) => b.function_instantiation_at(idx),
				CompiledMove::Module(b) => b.function_instantiation_at(idx),
			}
		}


		fn signature_at(&self, idx: SignatureIndex) -> &Signature {
			match self {
				CompiledMove::Script(b) => b.signature_at(idx),
				CompiledMove::Module(b) => b.signature_at(idx),
			}
		}

		fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
			match self {
				CompiledMove::Script(b) => b.identifier_at(idx),
				CompiledMove::Module(b) => b.identifier_at(idx),
			}
		}

		fn address_identifier_at(&self, idx: AddressIdentifierIndex) -> &AccountAddress {
			match self {
				CompiledMove::Script(b) => b.address_identifier_at(idx),
				CompiledMove::Module(b) => b.address_identifier_at(idx),
			}
		}

		fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
			match self {
				CompiledMove::Script(b) => b.constant_at(idx),
				CompiledMove::Module(b) => b.constant_at(idx),
			}
		}


		fn module_handles(&self) -> &[ModuleHandle] {
			match self {
				CompiledMove::Script(b) => b.module_handles(),
				CompiledMove::Module(b) => b.module_handles(),
			}
		}

		fn struct_handles(&self) -> &[StructHandle] {
			match self {
				CompiledMove::Script(b) => b.struct_handles(),
				CompiledMove::Module(b) => b.struct_handles(),
			}
		}

		fn function_handles(&self) -> &[FunctionHandle] {
			match self {
				CompiledMove::Script(b) => b.function_handles(),
				CompiledMove::Module(b) => b.function_handles(),
			}
		}


		fn function_instantiations(&self) -> &[FunctionInstantiation] {
			match self {
				CompiledMove::Script(b) => b.function_instantiations(),
				CompiledMove::Module(b) => b.function_instantiations(),
			}
		}


		fn signatures(&self) -> &[Signature] {
			match self {
				CompiledMove::Script(b) => b.signatures(),
				CompiledMove::Module(b) => b.signatures(),
			}
		}

		fn constant_pool(&self) -> &[Constant] {
			match self {
				CompiledMove::Script(b) => b.constant_pool(),
				CompiledMove::Module(b) => b.constant_pool(),
			}
		}

		fn identifiers(&self) -> &[Identifier] {
			match self {
				CompiledMove::Script(b) => b.identifiers(),
				CompiledMove::Module(b) => b.identifiers(),
			}
		}

		fn address_identifiers(&self) -> &[AccountAddress] {
			match self {
				CompiledMove::Script(b) => b.address_identifiers(),
				CompiledMove::Module(b) => b.address_identifiers(),
			}
		}
	}
}
