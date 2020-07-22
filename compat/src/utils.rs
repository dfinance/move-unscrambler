pub use libra::vm::file_format_common::*;
use anyhow::Result;

pub fn write_u8(binary: &mut BinaryData, val: u8) -> Result<()> {
    binary.push(val)?;
    Ok(())
}
