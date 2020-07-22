use serde::Serialize;
use crate::types::*;
use crate::cli::Dialect;
use crate::extract::prelude::*;
use crate::disasm::{CompiledMoveRef, CompiledMove};
use crate::{
    output::ctx::{ContextRoot, Context, StructKnowledge, FnKnowledge, FnKnowledgeBasic},
    deps::map::ModMap,
};
use std::collections::HashMap;

/// Storage for intermediate results
pub struct Db {
    pub dialect: Dialect,
    pub root: DbRoot,

    pub modules: ModMap,
    pub functions: FnMap,
    pub structs: StructMap,

    pub missed_modules: Vec<ModAddr>,
}

pub struct DbRoot {
    pub bc: CompiledMove,
    pub kind: MoveType,
    pub entry_points: Vec<FnAddr>,
}

impl Extract<Dialect> for Db {
    fn extract(&self) -> Dialect {
        self.dialect
    }
}

impl Extract<MoveType> for DbRoot {
    fn extract(&self) -> MoveType {
        self.kind.clone()
    }
}

impl ExtractRef<CompiledMove> for DbRoot {
    fn extract_ref(&self) -> &CompiledMove {
        &self.bc
    }
}

impl ExtractRef<DbRoot> for Db {
    fn extract_ref(&self) -> &DbRoot {
        &self.root
    }
}

impl ExtractRef<ModMap> for Db {
    fn extract_ref(&self) -> &ModMap {
        &self.modules
    }
}

impl ExtractRef<FnMap> for Db {
    fn extract_ref(&self) -> &FnMap {
        &self.functions
    }
}

impl Extract<StructMap> for Db {
    fn extract(&self) -> StructMap {
        self.structs.to_owned()
    }
}
impl ExtractRef<StructMap> for Db {
    fn extract_ref(&self) -> &StructMap {
        &self.structs
    }
}

impl Context<StructInfo> for Db {
    type Root = DbRoot;
    fn root(&self) -> &DbRoot {
        &self.root
    }
}

impl ContextRoot for DbRoot {
    fn entry_points(&self) -> &[FnAddr] {
        &self.entry_points
    }
}

// XXX: temp part

impl Extract<StructKind> for StructInfo {
    fn extract(&self) -> StructKind {
        // XXX: opt this, ret ref
        self.kind.to_owned()
    }
}

impl ExtractRef<[TypeParamKind]> for StructInfo {
    fn extract_ref(&self) -> &[TypeParamKind] {
        &self.type_params
    }
}

impl StructKnowledge for StructInfo {
    fn is_native(&self) -> bool {
        self.is_native
    }
    fn fields(&self) -> &HashMap<String, Ty> {
        &self.fields
    }
}

impl Extract<FnKnowledgeBasic> for (&FnAddr, &FunctionInfo) {
    fn extract(&self) -> FnKnowledgeBasic {
        FnKnowledgeBasic {
            address: self.0.to_owned(),
            parameters: self.1.parameters.clone(),
            type_parameters: self.1.type_parameters.clone(),
            returns: self.1.returns.clone(),
            acquires: self.1.acquires.clone(),
            is_public: self.1.is_public,
            is_native: self.1.is_native,
            // code:
        }
    }
}
