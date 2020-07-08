use std::collections::HashMap;

use libra::vm::file_format::{
    CompiledModule, StructFieldInformation, TypeSignature, SignatureToken, Kind, StructHandle,
};
use libra::vm::access::ModuleAccess;
use std::ops::Deref;
use crate::types::{ModAddr, StructAddr};

pub type StructMap = HashMap<StructAddr, StructInfo>;

#[derive(Debug)]
pub enum TypeParamKind {
    All,
    Resource,
    Copyable,
}

#[derive(Debug)]
pub enum StructKind {
    HasResourceAsType,
    Resource,
    Copyable,
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

#[derive(Debug)]
pub struct StructInfo {
    is_native: bool,
    kind: StructKind,
    type_params: Vec<TypeParamKind>,
    fields: HashMap<String, Ty>,
}

impl StructInfo {
    pub fn new(
        kind: StructKind,
        is_native: bool,
        type_params: Vec<TypeParamKind>,
        fields: HashMap<String, Ty>,
    ) -> StructInfo {
        StructInfo {
            is_native,
            kind,
            type_params,
            fields,
        }
    }
}

pub fn extract_type_param_kind(tp_kind: Kind) -> TypeParamKind {
    match tp_kind {
        Kind::All => TypeParamKind::All,
        Kind::Copyable => TypeParamKind::Copyable,
        Kind::Resource => TypeParamKind::Resource,
    }
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

pub fn extract_struct_map(compiled_mod: &CompiledModule) -> HashMap<StructAddr, StructInfo> {
    let mut structs = HashMap::new();
    for struct_def in compiled_mod.struct_defs() {
        let struct_handle = compiled_mod.struct_handle_at(struct_def.struct_handle.clone());
        let name = compiled_mod
            .identifier_at(struct_handle.name)
            .as_str()
            .to_string();
        let kind = if struct_handle.is_nominal_resource {
            StructKind::Resource
        } else if struct_handle
            .type_parameters
            .iter()
            .any(|param| matches!(param, Kind::Resource))
        {
            StructKind::HasResourceAsType
        } else {
            StructKind::Copyable
        };
        let type_params = struct_handle
            .type_parameters
            .iter()
            .map(|tp| extract_type_param_kind(tp.to_owned()))
            .collect();
        let struct_info = match &struct_def.field_information {
            StructFieldInformation::Native => {
                StructInfo::new(kind, true, type_params, HashMap::default())
            }
            StructFieldInformation::Declared(fields) => {
                let mut fields_map = HashMap::new();
                for field in fields {
                    let name = compiled_mod
                        .identifier_at((&field.name).to_owned())
                        .as_str()
                        .to_string();
                    let ty = extract_ty(&field.signature.0, compiled_mod);
                    fields_map.insert(name, ty);
                }
                StructInfo::new(kind, false, type_params, fields_map)
            }
        };
        structs.insert(StructAddr::new(compiled_mod.self_id(), name), struct_info);
    }
    structs
}
