mod mod_addr;
mod fn_addr;
mod struct_addr;
mod block_addr;

pub use mod_addr::*;
pub use fn_addr::*;
pub use struct_addr::*;
pub use block_addr::*;


#[derive(Debug)]
pub enum MoveType {
	Script,
	Module,
}
