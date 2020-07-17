use libra::vm::file_format::CompiledScript;
use libra::vm::CompiledModule;
use libra::vm::errors::BinaryLoaderResult;
use super::{deserialize_module, deserialize_script};

#[derive(Debug)]
pub enum CompiledMove {
    Script(CompiledScript),
    Module(CompiledModule),
}

#[derive(Debug, Clone)]
pub enum CompiledMoveRef<'a> {
    Script(&'a CompiledScript),
    Module(&'a CompiledModule),
}

impl From<CompiledScript> for CompiledMove {
    fn from(b: CompiledScript) -> Self {
        CompiledMove::Script(b)
    }
}

impl From<CompiledModule> for CompiledMove {
    fn from(b: CompiledModule) -> Self {
        CompiledMove::Module(b)
    }
}

impl<'a> From<&'a CompiledScript> for CompiledMoveRef<'a> {
    fn from(b: &'a CompiledScript) -> Self {
        CompiledMoveRef::Script(b)
    }
}

impl<'a> From<&'a CompiledModule> for CompiledMoveRef<'a> {
    fn from(b: &'a CompiledModule) -> Self {
        CompiledMoveRef::Module(b)
    }
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

pub use access::*;
mod access {
    use super::*;
    use libra::libra_types::account_address::AccountAddress;
    use libra::move_core_types::language_storage::ModuleId;
    use libra::move_core_types::identifier::Identifier;
    use libra::move_core_types::identifier::IdentStr;
    use libra::libra_types::access_path::AccessPath;
    use libra::vm::access::{ScriptAccess, ModuleAccess};
    use libra::vm::file_format::{
        ModuleHandle, ModuleHandleIndex, StructDefInstantiationIndex, FunctionInstantiationIndex,
        FieldInstantiationIndex, FunctionInstantiation, FieldInstantiation, Signature,
        SignatureIndex, IdentifierIndex, AddressIdentifierIndex, ConstantPoolIndex, Constant,
        StructDefinition, StructDefinitionIndex, FieldHandleIndex, FieldHandle,
        StructDefInstantiation, FunctionHandleIndex, FunctionHandle, StructHandleIndex,
        StructHandle, FunctionDefinitionIndex, FunctionDefinition,
    };
    use crate::types::{FnAddr, ModAddr};

    /// Represents accessors for a compiled move binary.
    pub trait MoveAccess: Sync {
        fn is_script(&self) -> bool;
        fn is_module(&self) -> bool;

        fn name(&self) -> &IdentStr;
        fn name_str(&self) -> &str;
        fn address(&self) -> AccountAddress;

        fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle;
        fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle;
        fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle;
        fn function_instantiation_at(
            &self,
            idx: FunctionInstantiationIndex,
        ) -> &FunctionInstantiation;
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

    pub const DEFAULT_SCRIPT_MAINFN_NAME: &'static str = "main";
    pub const DEFAULT_SCRIPT_NAME: &'static str = "script";
    pub const DEFAULT_SCRIPT_ADDRESS: [u8; AccountAddress::LENGTH] = [255; AccountAddress::LENGTH];

    type IntoModAddr = (AccountAddress, &'static str);
    pub const fn default_script_address() -> IntoModAddr /* :impl Into<ModAddr> */ {
        (
            AccountAddress::new(DEFAULT_SCRIPT_ADDRESS),
            DEFAULT_SCRIPT_NAME,
        )
    }

    type IntoFnAddr = (IntoModAddr, &'static str);
    pub const fn default_script_fn_address() -> IntoFnAddr /* :impl Into<FnAddr> */ {
        (default_script_address(), DEFAULT_SCRIPT_MAINFN_NAME)
    }

    impl MoveAccess for CompiledMove {
        fn is_script(&self) -> bool {
            matches!(self, Self::Script(_))
        }
        fn is_module(&self) -> bool {
            matches!(self, Self::Module(_))
        }

        /// Returns the name of the module.
        fn name(&self) -> &IdentStr {
            match self {
                Self::Script(_) => IdentStr::new(DEFAULT_SCRIPT_NAME).unwrap(),
                Self::Module(b) => b.name(),
            }
        }

        /// Returns the name of the module.
        fn name_str(&self) -> &str {
            match self {
                Self::Script(_) => DEFAULT_SCRIPT_NAME,
                Self::Module(b) => b.name().as_str(),
            }
        }

        /// Returns the address of the module.
        fn address(&self) -> AccountAddress {
            match self {
                Self::Script(_) => AccountAddress::new(DEFAULT_SCRIPT_ADDRESS),
                Self::Module(b) => b.address().to_owned(),
            }
        }

        fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
            match self {
                Self::Script(b) => b.module_handle_at(idx),
                Self::Module(b) => b.module_handle_at(idx),
            }
        }

        fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
            match self {
                Self::Script(b) => b.struct_handle_at(idx),
                Self::Module(b) => b.struct_handle_at(idx),
            }
        }

        fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
            match self {
                Self::Script(b) => b.function_handle_at(idx),
                Self::Module(b) => b.function_handle_at(idx),
            }
        }

        fn function_instantiation_at(
            &self,
            idx: FunctionInstantiationIndex,
        ) -> &FunctionInstantiation {
            match self {
                Self::Script(b) => b.function_instantiation_at(idx),
                Self::Module(b) => b.function_instantiation_at(idx),
            }
        }

        fn signature_at(&self, idx: SignatureIndex) -> &Signature {
            match self {
                Self::Script(b) => b.signature_at(idx),
                Self::Module(b) => b.signature_at(idx),
            }
        }

        fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
            match self {
                Self::Script(b) => b.identifier_at(idx),
                Self::Module(b) => b.identifier_at(idx),
            }
        }

        fn address_identifier_at(&self, idx: AddressIdentifierIndex) -> &AccountAddress {
            match self {
                Self::Script(b) => b.address_identifier_at(idx),
                Self::Module(b) => b.address_identifier_at(idx),
            }
        }

        fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
            match self {
                Self::Script(b) => b.constant_at(idx),
                Self::Module(b) => b.constant_at(idx),
            }
        }

        fn module_handles(&self) -> &[ModuleHandle] {
            match self {
                Self::Script(b) => b.module_handles(),
                Self::Module(b) => b.module_handles(),
            }
        }

        fn struct_handles(&self) -> &[StructHandle] {
            match self {
                Self::Script(b) => b.struct_handles(),
                Self::Module(b) => b.struct_handles(),
            }
        }

        fn function_handles(&self) -> &[FunctionHandle] {
            match self {
                Self::Script(b) => b.function_handles(),
                Self::Module(b) => b.function_handles(),
            }
        }

        fn function_instantiations(&self) -> &[FunctionInstantiation] {
            match self {
                Self::Script(b) => b.function_instantiations(),
                Self::Module(b) => b.function_instantiations(),
            }
        }

        fn signatures(&self) -> &[Signature] {
            match self {
                Self::Script(b) => b.signatures(),
                Self::Module(b) => b.signatures(),
            }
        }

        fn constant_pool(&self) -> &[Constant] {
            match self {
                Self::Script(b) => b.constant_pool(),
                Self::Module(b) => b.constant_pool(),
            }
        }

        fn identifiers(&self) -> &[Identifier] {
            match self {
                Self::Script(b) => b.identifiers(),
                Self::Module(b) => b.identifiers(),
            }
        }

        fn address_identifiers(&self) -> &[AccountAddress] {
            match self {
                Self::Script(b) => b.address_identifiers(),
                Self::Module(b) => b.address_identifiers(),
            }
        }
    }

    // impl<'a> MoveAccess for CompiledMoveRef<'a> {
    impl MoveAccess for CompiledMoveRef<'_> {
        fn is_script(&self) -> bool {
            matches!(self, Self::Script(_))
        }
        fn is_module(&self) -> bool {
            matches!(self, Self::Module(_))
        }

        /// Returns the name of the module.
        fn name(&self) -> &IdentStr {
            match self {
                Self::Script(_) => IdentStr::new(DEFAULT_SCRIPT_NAME).unwrap(),
                Self::Module(b) => b.name(),
            }
        }

        /// Returns the name of the module.
        fn name_str(&self) -> &str {
            match self {
                Self::Script(_) => DEFAULT_SCRIPT_NAME,
                Self::Module(b) => b.name().as_str(),
            }
        }

        /// Returns the address of the module.
        fn address(&self) -> AccountAddress {
            match self {
                Self::Script(_) => AccountAddress::new(DEFAULT_SCRIPT_ADDRESS),
                Self::Module(b) => b.address().to_owned(),
            }
        }

        fn module_handle_at(&self, idx: ModuleHandleIndex) -> &ModuleHandle {
            match self {
                Self::Script(b) => b.module_handle_at(idx),
                Self::Module(b) => b.module_handle_at(idx),
            }
        }

        fn struct_handle_at(&self, idx: StructHandleIndex) -> &StructHandle {
            match self {
                Self::Script(b) => b.struct_handle_at(idx),
                Self::Module(b) => b.struct_handle_at(idx),
            }
        }

        fn function_handle_at(&self, idx: FunctionHandleIndex) -> &FunctionHandle {
            match self {
                Self::Script(b) => b.function_handle_at(idx),
                Self::Module(b) => b.function_handle_at(idx),
            }
        }

        fn function_instantiation_at(
            &self,
            idx: FunctionInstantiationIndex,
        ) -> &FunctionInstantiation {
            match self {
                Self::Script(b) => b.function_instantiation_at(idx),
                Self::Module(b) => b.function_instantiation_at(idx),
            }
        }

        fn signature_at(&self, idx: SignatureIndex) -> &Signature {
            match self {
                Self::Script(b) => b.signature_at(idx),
                Self::Module(b) => b.signature_at(idx),
            }
        }

        fn identifier_at(&self, idx: IdentifierIndex) -> &IdentStr {
            match self {
                Self::Script(b) => b.identifier_at(idx),
                Self::Module(b) => b.identifier_at(idx),
            }
        }

        fn address_identifier_at(&self, idx: AddressIdentifierIndex) -> &AccountAddress {
            match self {
                Self::Script(b) => b.address_identifier_at(idx),
                Self::Module(b) => b.address_identifier_at(idx),
            }
        }

        fn constant_at(&self, idx: ConstantPoolIndex) -> &Constant {
            match self {
                Self::Script(b) => b.constant_at(idx),
                Self::Module(b) => b.constant_at(idx),
            }
        }

        fn module_handles(&self) -> &[ModuleHandle] {
            match self {
                Self::Script(b) => b.module_handles(),
                Self::Module(b) => b.module_handles(),
            }
        }

        fn struct_handles(&self) -> &[StructHandle] {
            match self {
                Self::Script(b) => b.struct_handles(),
                Self::Module(b) => b.struct_handles(),
            }
        }

        fn function_handles(&self) -> &[FunctionHandle] {
            match self {
                Self::Script(b) => b.function_handles(),
                Self::Module(b) => b.function_handles(),
            }
        }

        fn function_instantiations(&self) -> &[FunctionInstantiation] {
            match self {
                Self::Script(b) => b.function_instantiations(),
                Self::Module(b) => b.function_instantiations(),
            }
        }

        fn signatures(&self) -> &[Signature] {
            match self {
                Self::Script(b) => b.signatures(),
                Self::Module(b) => b.signatures(),
            }
        }

        fn constant_pool(&self) -> &[Constant] {
            match self {
                Self::Script(b) => b.constant_pool(),
                Self::Module(b) => b.constant_pool(),
            }
        }

        fn identifiers(&self) -> &[Identifier] {
            match self {
                Self::Script(b) => b.identifiers(),
                Self::Module(b) => b.identifiers(),
            }
        }

        fn address_identifiers(&self) -> &[AccountAddress] {
            match self {
                Self::Script(b) => b.address_identifiers(),
                Self::Module(b) => b.address_identifiers(),
            }
        }
    }
}
