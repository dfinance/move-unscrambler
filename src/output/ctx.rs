use serde::Serialize;
use crate::types::*;
use crate::cli::Dialect;
use crate::extract::prelude::*;
use crate::disasm::CompiledMove;
use crate::deps::map::ModMap;

pub struct Db {
    pub dialect: Dialect,
    pub root: CompiledMove,
    pub root_type: MoveType,
    pub entry_points: Vec<FnAddr>,

    pub modules: ModMap,
    pub functions: FnMap,
    pub structs: StructMap,

    pub missed_modules: Vec<ModAddr>,
}

#[derive(Debug, Serialize)]
pub struct Ctx {
    root: Root,
    // functions: FnMap,
    structs: StructMap,
    // dependencies: dependencies,
}

/// Contains user's input
#[derive(Debug, Serialize)]
pub struct Root {
    is_script: bool,
    address: ModAddr,
    entry_points: Vec<(FnAddr, () /* FunctionInfo */)>,
}

pub fn create(db: &Db) -> Ctx {
    let root = Root {
        is_script: matches!(db.root_type, MoveType::Script),
        address: db.root.extract(),
        // entry_points: entry_points
        //     .iter()
        //     .filter_map(|addr| fn_map.get(addr).map(|f| (addr, f)))
        //     .collect(),
        entry_points: vec![],
    };

    let ctx = Ctx {
        root,
        structs: Default::default(),
        // functions: Default::default(),
    };

    ctx
}
// pub fn create(root: &CompiledMove, entry_points: &[FnAddr], fn_map: FnMap) -> Ctx {
//     let root = Root {
//         is_script: matches!(root.extract(), MoveType::Script),
//         address: root.extract(),
//         // entry_points: entry_points
//         //     .iter()
//         //     .filter_map(|addr| fn_map.get(addr).map(|f| (addr, f)))
//         //     .collect(),
//         entry_points: vec![],
//     };

//     let ctx = Ctx {
//         root,
//         structs: Default::default(),
//         // functions: Default::default(),
//     };

//     ctx
// }
