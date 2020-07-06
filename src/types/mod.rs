mod mod_addr;
mod fn_addr;
mod block_addr;

pub use mod_addr::*;
pub use fn_addr::*;
pub use block_addr::*;


#[derive(Debug)]
pub enum MoveType {
	Script,
	Module,
}

#[derive(Debug, Hash, Eq, PartialEq)]
pub struct StructAddr {
	module: ModAddr,
	name: String,
}

impl StructAddr {
	pub fn new(mod_addr: ModAddr, name: String) -> StructAddr {
		StructAddr { module: mod_addr,
		             name }
	}
}
