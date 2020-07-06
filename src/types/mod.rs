mod acc_addr;
mod mod_addr;
mod fn_addr;
mod block_addr;

pub use acc_addr::*;
pub use mod_addr::*;
pub use fn_addr::*;
pub use block_addr::*;


#[derive(Debug)]
pub enum MoveType {
	Script,
	Module,
}
