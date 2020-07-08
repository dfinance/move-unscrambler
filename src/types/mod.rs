mod mod_addr;
mod fn_addr;
mod struct_addr;
mod block_addr;

pub use mod_addr::*;
pub use fn_addr::*;
pub use struct_addr::*;
pub use block_addr::*;

use libra::vm::file_format::{CompiledModule, SignatureToken, Kind};
use libra::vm::access::ModuleAccess;

#[derive(Debug)]
pub enum MoveType {
    Script,
    Module,
}

#[derive(Debug)]
pub enum TypeParamKind {
    All,
    Resource,
    Copyable,
}

pub fn extract_type_param_kind(tp_kind: Kind) -> TypeParamKind {
    match tp_kind {
        Kind::All => TypeParamKind::All,
        Kind::Copyable => TypeParamKind::Copyable,
        Kind::Resource => TypeParamKind::Resource,
    }
}

#[derive(Debug)]
pub enum Ty {
    Bool,
    U8,
    U64,
    U128,
    Address,
    Signer,
    Vector(Box<Ty>),

    /// Reference to a type.
    Reference(Box<Ty>),

    /// Mutable reference to a type.
    MutableReference(Box<Ty>),

    /// MOVE user type, resource or copyable
    Struct(StructAddr),
    // StructInstantiation(StructHandleIndex, Vec<SignatureToken>),
    // Type parameter.
    TypeParameter(u16),
}

pub fn extract_ty(sign_token: &SignatureToken, compiled_mod: &CompiledModule) -> Ty {
    match sign_token {
        SignatureToken::Bool => Ty::Bool,
        SignatureToken::U8 => Ty::U8,
        SignatureToken::U64 => Ty::U64,
        SignatureToken::U128 => Ty::U128,
        SignatureToken::Address => Ty::Address,
        SignatureToken::Signer => Ty::Signer,
        SignatureToken::Vector(ty) => Ty::Vector(Box::new(extract_ty(ty, compiled_mod))),
        SignatureToken::Reference(ty) => Ty::Reference(Box::new(extract_ty(ty, compiled_mod))),
        SignatureToken::MutableReference(ty) => {
            Ty::MutableReference(Box::new(extract_ty(ty, compiled_mod)))
        }
        SignatureToken::Struct(idx) => {
            let struct_handle = compiled_mod.struct_handle_at(idx.clone());
            let struct_name = compiled_mod
                .identifier_at(struct_handle.name)
                .as_str()
                .to_string();
            let module_id = compiled_mod
                .module_id_for_handle(compiled_mod.module_handle_at(struct_handle.module));
            Ty::Struct(StructAddr::new(module_id, struct_name))
        }
        SignatureToken::TypeParameter(idx) => Ty::TypeParameter(idx.to_owned()),
        _ => todo!(),
    }
}
