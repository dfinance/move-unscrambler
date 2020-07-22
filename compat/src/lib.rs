#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;

use std::{
    io::Cursor,
    ops::{Range, RangeTo},
    convert::TryInto,
};
use anyhow::Result;
use libra::libra_types::account_address::AccountAddress;
use libra::vm::file_format_common::{TableType, BinaryConstants};
use libra::vm::file_format_common::*;

mod detect;
mod convert;
mod utils;
use utils::*;

type Cur<'a> = Cursor<&'a [u8]>;
pub type BinVersion = (u8, u8);

const NATIVE_ADDR_LEN: usize = AccountAddress::LENGTH;

const MAGIC_SIZE: usize = BinaryConstants::LIBRA_MAGIC_SIZE;
const MAGIC_POS: RangeTo<usize> = ..MAGIC_SIZE;
const VERSION_SIZE: usize = 2;
const VERSION_POS: Range<usize> = MAGIC_SIZE..(MAGIC_SIZE + VERSION_SIZE);

pub fn adapt(bytes: &mut Vec<u8>) {
    debug!("native address length: {}", NATIVE_ADDR_LEN);
    debug!("supported address length: ≦ {}", NATIVE_ADDR_LEN);

    let notify_err = |err| return error!("{}", err);

    let result = check(&bytes)
        .map_err(notify_err)
        .and_then(|_| read_tables(&bytes).map_err(notify_err))
        .and_then(|changes| apply_changes(changes, bytes).map_err(notify_err));

    {
        // final checks:
        debug_assert!(result.is_ok());
        debug_assert!(matches!(read_tables(&bytes), Ok(Changes::None)));
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Table {
    kind: u8, /* :TableType */
    offset: u32,
    length: u32,
}

fn check(bytes: &[u8]) -> Result<()> {
    check_magic(&bytes)?;
    check_version(&bytes)?;
    Ok(())
}

#[derive(Clone, Debug)]
enum Changes {
    /// When changes are not needed
    None,

    /// Contains all new tables mapped to old ones
    Some(Vec<(Table, Range<u64>)>, Vec<ChangedTable>),
}

#[derive(Clone, Debug)]
struct ChangedTable {
    /// index of original table head
    i: usize,
    /// modified body
    body: Vec<u8>,
    pos: PosInfo,
}

#[derive(Clone, Debug)]
struct PosInfo {
    heads: Range<u64>,
    target_head: Range<u64>,
    target_body: Range<u64>,
}

fn read_tables(bytes: &[u8]) -> Result<Changes> {
    let mut cursor = Cursor::new(&bytes[VERSION_POS.end..]);

    let tlen = read_uleb128_as_u64(&mut cursor)?;
    // debug!("reading {} tables", tlen,);
    assert!(TABLE_COUNT_MAX >= tlen);

    let the_start_of_heads = cursor.position() as usize + VERSION_POS.end;

    let mut tables = Vec::new();
    read_tables_heads(&mut cursor, tlen.try_into()?, &mut tables)?;

    // fix offset:
    tables.iter_mut().for_each(|t| {
        t.1.start += VERSION_POS.end as u64;
        t.1.end += VERSION_POS.end as u64;
    });

    let the_end_of_heads = cursor.position() as usize + VERSION_POS.end;

    let addr_tables = tables
        .iter()
        .enumerate()
        .filter(|(_, (t, _))| t.kind == TableType::ADDRESS_IDENTIFIERS as u8);

    let detected_addr_len = {
        let mut addr_tables_lengths = Vec::new();
        let address_tables = addr_tables.clone();
        address_tables.for_each(|(_, (t, _))| addr_tables_lengths.push(t.length));
        detect::address_length(&addr_tables_lengths[..])?
    };

    let result = if detected_addr_len as usize == NATIVE_ADDR_LEN {
        Changes::None
    } else {
        debug!("detected addr len: {}", detected_addr_len);

        let mut fixed_bodies = Vec::new();
        for (i, (t, p)) in addr_tables {
            let pos_info = {
                let heads = the_start_of_heads as u64..the_end_of_heads as u64;
                let target_head = p.to_owned();
                let target_body_from = the_end_of_heads as u64 + t.offset as u64;
                let target_body = target_body_from..(target_body_from + t.length as u64);
                PosInfo {
                    heads,
                    target_head,
                    target_body,
                }
            };

            let buf = &bytes[the_end_of_heads..(the_end_of_heads + (t.offset + t.length) as usize)];
            let new_body = expand_addr_table(buf, &t, detected_addr_len as usize);
            fixed_bodies.push(ChangedTable {
                i,
                body: new_body,
                pos: pos_info,
            });
        }

        Changes::Some(tables, fixed_bodies)
    };

    Ok(result)
}

fn apply_changes(changes: Changes, bytes: &mut Vec<u8>) -> Result<()> {
    match changes {
        Changes::None => {}
        Changes::Some(heads, changes) => {
            for item in changes {
                trace!("applying fixes for table {}", item.i);

                let (target_head, _target_head_pos) = &heads[item.i];
                let body = &item.body;
                let body_len_dif = body.len() - heads[item.i].0.length as usize;

                // fix offset for each head where offset > this body position
                // collect all heads where offset should be fixed:
                let mut heads_to_fix_offset = vec![item.i];
                {
                    for (i, (t, _)) in heads.iter().enumerate() {
                        if t.offset > target_head.offset {
                            heads_to_fix_offset.push(i);
                        }
                    }
                    heads_to_fix_offset.sort();
                    debug!("heads to fix offset: {:?}", heads_to_fix_offset);
                }

                let mut result = Vec::new();
                let mut cursor = Cursor::new(&bytes[..]);

                for i in heads_to_fix_offset {
                    let (head, head_pos) = &heads[i];

                    // read to target head start:
                    {
                        result.extend(bytes[result.len()..head_pos.start as usize].iter().cloned());
                        cursor.set_position(head_pos.start);
                        assert_eq!(result.len(), cursor.position() as usize);
                    }

                    // this head:
                    {
                        let mut this_head = read_table_head(&mut cursor)?;
                        assert_eq!(head, &this_head);
                        assert_eq!(head_pos.end, cursor.position());

                        // fix head values:
                        if i == item.i && &this_head == target_head {
                            this_head.length = body.len() as u32;
                            debug!("target head ({}) length fixed", i);
                        } else {
                            this_head.offset += body_len_dif as u32;
                            debug!("following head ({}) offset fixed", i);
                        }

                        // write new head:
                        let new_head_bin = write_table_head(&this_head)?;
                        result.extend(new_head_bin.into_iter());
                        assert_eq!(result.len(), cursor.position() as usize);

                        // check just written head:
                        {
                            cursor.set_position(head_pos.start);
                            let new_head = read_table_head(&mut cursor)?;

                            assert_eq!(this_head.kind, new_head.kind);

                            if i == item.i {
                                assert_eq!(this_head.offset, new_head.offset);
                                assert_ne!(this_head.length, new_head.length);
                            } else {
                                assert_ne!(this_head.offset, new_head.offset);
                                assert_eq!(this_head.length, new_head.length);
                            }
                        }
                    }

                    cursor.set_position(head_pos.end);
                    assert_eq!(result.len(), cursor.position() as usize);
                }

                // other following heads:
                {
                    cursor.set_position(item.pos.target_body.start);

                    result.extend(
                        bytes[result.len()..cursor.position() as usize]
                            .iter()
                            .cloned(),
                    );

                    assert_eq!(result.len(), cursor.position() as usize);
                }

                // write body:
                {
                    result.extend(body.iter().cloned());
                    cursor.set_position(item.pos.target_body.end);
                }

                // finally:
                result.extend(bytes[cursor.position() as usize..].iter().cloned());
                assert_eq!(result.len(), bytes.len() + body_len_dif);

                let result_len = result.len();

                // absolutely finally:
                bytes.clear();
                bytes.extend(result.into_iter());

                assert_eq!(result_len, bytes.len());
            }
        }
    }

    Ok(())
}

fn expand_addr_table(bytes: &[u8], table: &Table, addr_length: usize) -> Vec<u8> {
    // read addresses:
    let addrs = read_table_address_identifiers(bytes, table, addr_length);
    assert_eq!(table.length as usize / addr_length, addrs.len());

    let result = create_table_address_identifiers(&addrs);

    assert!(result.len() > table.length as usize);
    assert_eq!(
        result.len(),
        table.length as usize + ((NATIVE_ADDR_LEN - addr_length) * addrs.len())
    );

    result
}

fn create_table_address_identifiers(addrs: &[AccountAddress]) -> Vec<u8> {
    addrs
        .into_iter()
        .flat_map(|addr| addr.to_vec().into_iter())
        .collect()
}

fn read_table_address_identifiers(
    bytes: &[u8],
    table: &Table,
    addr_length: usize,
) -> Vec<AccountAddress> {
    trace!("reading addrs as {}-to-{}b", addr_length, NATIVE_ADDR_LEN);

    let mut result = Vec::new();

    if NATIVE_ADDR_LEN != addr_length {
        let mut start = table.offset as usize;

        for _ in 0..table.length as usize / addr_length {
            let end_addr = start + addr_length;

            let mut addr_bytes = [0; NATIVE_ADDR_LEN];

            convert::expand_addr(bytes[start..end_addr].iter().cloned(), addr_length)
                .enumerate()
                .for_each(|(i, b)| addr_bytes[i] = b);

            let addr = AccountAddress::new(addr_bytes);

            start = end_addr;
            result.push(addr);
        }
    } else {
        // actually unreachable part
        let mut start = table.offset as usize;
        for _ in 0..table.length as usize / addr_length {
            let end_addr = start + addr_length;
            let address: Result<AccountAddress> = (&bytes[start..end_addr]).try_into();
            match address {
                Ok(addr) => result.push(addr),
                Err(err) => error!("Cannot read address: {}", err),
            }
            start = end_addr;
        }
    }

    result
}

fn check_magic(bytes: &[u8]) -> Result<()> {
    if let Some(magic) = bytes.get(MAGIC_POS) {
        if magic != &BinaryConstants::LIBRA_MAGIC {
            // TODO: throw StatusCode::BAD_MAGIC
        }
    } else {
        return Err(anyhow!("Bad binary header"));
    }
    Ok(())
}

/// [major, minor]
fn check_version(bytes: &[u8]) -> Result<BinVersion> {
    if let Some(&[major, minor]) = bytes.get(VERSION_POS) {
        debug!("bytecode version: {}.{}", major, minor);
        Ok((major, minor))
    } else {
        Err(anyhow!("Unable to read version"))
    }
}

fn read_tables_heads(c: &mut Cur, len: usize, tables: &mut Vec<(Table, Range<u64>)>) -> Result<()> {
    for _ in 0..len {
        let start = c.position();
        let head = read_table_head(c)?;
        let end = c.position();
        tables.push((head, start..end));
    }
    Ok(())
}

fn read_table_head(r: &mut Cur) -> Result<Table> {
    // let start = r.position();
    let kind: u8 = read_u8(r)?.try_into()?;
    let offset: u32 = read_uleb128_as_u64(r)?.try_into()?;
    let count: u32 = read_uleb128_as_u64(r)?.try_into()?;
    // debug!("[R]  table head size: {}", r.position() - start);

    assert!(TABLE_OFFSET_MAX >= offset as u64);
    assert!(TABLE_SIZE_MAX >= count as u64);

    Ok(Table {
        kind,
        offset,
        length: count,
    })
}

fn write_table_head(t: &Table) -> Result<Vec<u8>> {
    let mut buf = BinaryData::new();
    write_u8(&mut buf, t.kind)?;
    write_u64_as_uleb128(&mut buf, t.offset as u64)?;
    write_u64_as_uleb128(&mut buf, t.length as u64)?;
    // debug!("[WR] table head size: {}", buf.as_inner().len());

    Ok(buf.into_inner())
}
