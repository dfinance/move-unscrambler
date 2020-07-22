use crate::{disasm::CompiledMove, types::*};
use super::*;

impl Extract<MoveType> for CompiledModule {
    fn extract(&self) -> MoveType {
        MoveType::Module
    }
}

impl Extract<MoveType> for CompiledScript {
    fn extract(&self) -> MoveType {
        MoveType::Script
    }
}

impl Extract<MoveType> for CompiledMove {
    fn extract(&self) -> MoveType {
        match self {
            CompiledMove::Module(_) => MoveType::Module,
            CompiledMove::Script(_) => MoveType::Script,
        }
    }
}
