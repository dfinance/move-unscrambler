use libra::vm::file_format::{CompiledModule, CodeUnit};
use libra::vm::access::ModuleAccess;
use crate::types::{
    extract_ty, Ty, FnAddr, ModAddr, IntoModAddr, TypeParamKind, extract_type_param_kind,
    StructAddr,
};
use std::collections::HashMap;

pub struct FunctionInfo {
    parameters: Vec<Ty>,
    type_parameters: Vec<TypeParamKind>,
    returns: Vec<Ty>,
    acquires: Vec<StructAddr>,
    is_public: bool,
    is_native: bool,
    code: Option<CodeUnit>,
}

pub fn extract_functions(compiled_mod: &CompiledModule) -> HashMap<FnAddr, FunctionInfo> {
    let mut functions_map = HashMap::new();
    for function_def in compiled_mod.function_defs() {
        let function_handle = compiled_mod.function_handle_at(function_def.function);
        let name = compiled_mod
            .identifier_at(function_handle.name)
            .as_str()
            .to_string();
        let parameters = compiled_mod
            .signature_at(function_handle.parameters)
            .0
            .iter()
            .map(|param| extract_ty(param, compiled_mod))
            .collect();
        let type_parameters = function_handle
            .type_parameters
            .iter()
            .map(|param| extract_type_param_kind(param.clone()))
            .collect();
        let returns = compiled_mod
            .signature_at(function_handle.return_)
            .0
            .iter()
            .map(|ty| extract_ty(ty, compiled_mod))
            .collect();
        let mut acqs = vec![];
        for acq in function_def.acquires_global_resources.clone() {
            let struct_def = compiled_mod.struct_def_at(acq);
            let struct_handle = compiled_mod.struct_handle_at(struct_def.struct_handle);
            let module_id = compiled_mod
                .module_id_for_handle(compiled_mod.module_handle_at(struct_handle.module));
            let name = compiled_mod.identifier_at(struct_handle.name);
            let acq_struct_addr = StructAddr::new(module_id.into_mod_addr(), name);
            acqs.push(acq_struct_addr)
        }
        let is_public = function_def.is_public();
        let is_native = function_def.is_native();
        let fn_addr = FnAddr::new(compiled_mod.self_id(), name);
        functions_map.insert(
            fn_addr,
            FunctionInfo {
                parameters,
                type_parameters,
                returns,
                acquires: acqs,
                is_public,
                is_native,
                code: function_def.code.clone(),
            },
        );
    }
    functions_map
}
