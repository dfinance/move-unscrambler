use vm::CompiledModule;
use vm::access::ModuleAccess;
use bytecode_verifier::control_flow_graph::{VMControlFlowGraph, ControlFlowGraph, BlockId};
use std::collections::HashSet;
use vm::file_format::{Bytecode, FunctionDefinition};


fn record_reachable_blocks(
	function_cfg: &VMControlFlowGraph,
	function_code: &Vec<Bytecode>,
	block_id: BlockId,
	reachables: &mut HashSet<BlockId>,
) {
	for successor_block_id in function_cfg.successors(block_id).to_owned() {
		if reachables.contains(&successor_block_id) {
			continue;
		}
		reachables.insert(successor_block_id);

		record_reachable_blocks(function_cfg, function_code, successor_block_id, reachables);
	}
}


pub fn analyse_code_reachability(compiled_module: CompiledModule) {
	for function_def in compiled_module.function_defs() {
		let function_code = function_def.code.clone().unwrap().code;
		let function_cfg = VMControlFlowGraph::new(&function_code);

		let entry_block_id = function_cfg.entry_block_id();
		let mut reachables = HashSet::new();
		reachables.insert(entry_block_id);

		record_reachable_blocks(&function_cfg, &function_code, entry_block_id, &mut reachables);

		let blocks = function_cfg.blocks().into_iter().collect::<HashSet<_>>();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use dialects::impls::LibraDialect;
	use dialects::base::Dialect;
	use dialects::shared::ProvidedAccountAddress;

	#[test]
	fn test_analyse_hello_world() {
		let source_code = r"
address 0x1 {
    module HelloWorld {
        fun greeting_number(): u64 {

       		loop {

       		}
        }
    }
}
        ";
		let fname = Box::leak(Box::new("module.move"));
		let dialect = LibraDialect::default();
		let (_, mut compiled_mods) = dialect
			.check_and_generate_bytecode(
				(fname, source_code.to_string()),
				&vec![],
				ProvidedAccountAddress::default(),
			)
			.unwrap();
		let compiled_mod = compiled_mods.remove(0);
		assert!(compiled_mods.is_empty());

		// analyse_code_reachability(compiled_mod);

		let function_def = compiled_mod.function_defs()[0].clone();
		let function_code = function_def.code.unwrap().code;
		let function_cfg = VMControlFlowGraph::new(&function_code);

		function_cfg.display();

		// for block_id in function_cfg.blocks() {
		// 	dbg!(block_id);
		// 	let instructions = function_code
		// 		[function_cfg.block_start(block_id) as usize..=function_cfg.block_end(block_id) as usize]
		// 		.to_vec();
		// 	dbg!(instructions);
		// }
		// for block_id in compiled_mod.bloc
	}
}
