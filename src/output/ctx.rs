use serde::Serialize;
use crate::types::*;
use crate::cli::Dialect;
use crate::extract::prelude::*;
use crate::disasm::CompiledMove;
use crate::{
    data::{DbRoot, Db},
    deps::map::ModMap,
};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct Ctx<Si> {
    root: Root,
    dependencies: Dependencies<Si>,
}

/// Contains user's input
#[derive(Debug, Serialize)]
pub struct Root {
    is_script: bool,
    address: ModAddr,
    entry_points: Vec<EntryPoint>,
}

#[derive(Debug, Serialize)]
pub struct EntryPoint {
    pub address: FnAddr,
    function: FnKnowledgeBasic,
}

/// Contains user's input
#[derive(Debug, Serialize)]
pub struct Dependencies<Si> {
    functions: FnKnowledgeMap<FnKnowledgeBasic>,
    structs: StructKnowledgeMap<Si>,
}

/// Storage for final results
pub trait Context<Si>: ExtractRef<FnMap> + Extract<StructKnowledgeMap<Si>> {
    type Root: ContextRoot;
    fn root(&self) -> &Self::Root;
}

pub trait ContextRoot: ExtractRef<CompiledMove> + Extract<MoveType> {
    fn entry_points(&self) -> &[FnAddr];
}

pub trait ContextFnKnowledge {}

type FnKnowledgeMap<Fi> = HashMap<String /* FnAddr */, Fi>;
type StructKnowledgeMap<Si> = HashMap<StructAddr, Si>;

pub trait StructKnowledge: Extract<StructKind> + ExtractRef<[TypeParamKind]> {
    fn is_native(&self) -> bool;

    fn kind(&self) -> StructKind {
        self.extract()
    }

    fn type_params(&self) -> &[TypeParamKind] {
        self.extract_ref()
    }

    fn fields(&self) -> &HashMap<String, Ty>;
}

pub trait FnKnowledge: ExtractRef<[TypeParamKind]> {
    fn address(&self) -> &FnAddr;

    fn is_public(&self) -> bool;
    fn is_native(&self) -> bool;

    fn acquires(&self) -> &[StructAddr];

    fn type_params(&self) -> &[TypeParamKind] {
        self.extract_ref()
    }

    fn parameters(&self) -> &[Ty];
    fn returns(&self) -> &[Ty];

    // TODO: fn code(&self) -> something serializable;
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize)]
pub struct FnKnowledgeBasic {
    pub address: FnAddr,
    pub is_public: bool,
    pub is_native: bool,
    pub parameters: Vec<Ty>,
    pub type_parameters: Vec<TypeParamKind>,
    pub returns: Vec<Ty>,
    pub acquires: Vec<StructAddr>,
    // pub code: Option<CodeUnit>,
}

impl ExtractRef<[TypeParamKind]> for FnKnowledgeBasic {
    fn extract_ref(&self) -> &[TypeParamKind] {
        &self.type_parameters
    }
}

impl FnKnowledge for FnKnowledgeBasic {
    fn address(&self) -> &FnAddr {
        &self.address
    }

    fn is_public(&self) -> bool {
        self.is_public
    }
    fn is_native(&self) -> bool {
        self.is_native
    }

    fn acquires(&self) -> &[StructAddr] {
        &self.acquires
    }

    fn type_params(&self) -> &[TypeParamKind] {
        self.extract_ref()
    }

    fn parameters(&self) -> &[Ty] {
        &self.parameters
    }
    fn returns(&self) -> &[Ty] {
        &self.returns
    }

    // TODO: fn code(&self) -> something serializable;
}

pub trait IntoContext<Si>
where
    Self: Context<Si>,
    Si: StructKnowledge,
{
    fn into_context(self) -> Ctx<Si>
    where
        Self: Sized,
    {
        let root = self.root();
        let root_bc = root.extract_ref();

        // XXX: opt
        let mut strct_map: StructKnowledgeMap<Si> = self.extract();
        let mut fn_map: FnKnowledgeMap<FnKnowledgeBasic> = self
            .extract_ref()
            .into_iter()
            .map(|(k, v)| (format!("{:#x}", k), (k, v).extract()))
            .collect();

        Ctx {
            root: Root {
                is_script: matches!(root.extract(), MoveType::Script),
                address: root_bc.extract(),
                entry_points: root
                    .entry_points()
                    .iter()
                    .filter_map(|addr| {
                        // XXX: opt
                        fn_map.get(&format!("{:#x}", addr)).map(|f| EntryPoint {
                            address: addr.to_owned(),
                            function: f.to_owned(),
                        })
                    })
                    .collect(),
            },
            dependencies: Dependencies {
                structs: {
                    strct_map.drain().all(|_| true);
                    strct_map
                },
                functions: {
                    // fn_map.drain().all(|_| true);
                    fn_map
                },
            },
        }
    }
}

impl<T, Si> IntoContext<Si> for T
where
    T: Context<Si>,
    Si: StructKnowledge,
{
}
