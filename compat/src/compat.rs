pub use libra::vm::file_format_common::*;
use anyhow::Result;

#[cfg(feature = "dfi-libra-compat")]
pub use internal::*;
#[cfg(feature = "dfi-libra-compat")]
mod internal {
    use super::*;
    use std::io::{Read, Cursor};
    use std::convert::TryInto;

    pub const TABLE_COUNT_MAX: u64 = 255;
    pub const TABLE_OFFSET_MAX: u64 = 0xffff_ffff;
    pub const TABLE_SIZE_MAX: u64 = 0xffff_ffff;

    pub fn read_uleb128_as_u64(cursor: &mut Cursor<&[u8]>) -> Result<u64> {
        read_uleb128_as_u16(cursor).map(|v| v.try_into().unwrap())
    }

    pub fn write_u64_as_uleb128(binary: &mut BinaryData, val: u64) -> Result<()> {
        write_u16_as_uleb128(binary, val.try_into()?).map(|v| v.try_into().unwrap())
    }

    pub fn read_u8(cursor: &mut Cursor<&[u8]>) -> Result<u8> {
        let mut buf = [0; 1];
        cursor.read_exact(&mut buf)?;
        Ok(buf[0])
    }
}

pub fn write_u8(binary: &mut BinaryData, val: u8) -> Result<()> {
    binary.push(val)?;
    Ok(())
}
