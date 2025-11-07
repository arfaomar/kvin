use crate::{BorrowedValue, Value};
use crc32fast::Hasher;

pub fn compute_crc32(payload: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(payload);
    hasher.finalize()
}

/// On-disk type tags (1 byte)
const TAG_INT: u8 = 1;
const TAG_TEXT: u8 = 2;
const TAG_BOOL: u8 = 3;
const TAG_BLOB: u8 = 4;

/// Entry header layout (little-endian):
/// [length: u64][checksum: u32][tag: u8] (=> 13 Bytes) followed by the tagged payload:
/// - Int:  [i64: 8]
/// - Bool: [u8: 1]
/// - Text: [len: u64][bytes: len]
/// - Blob: [len: u64][bytes: len]
///
/// `length` counts bytes after the u64 length field itself (i.e. checksum + tag + payload).
#[repr(C, packed)]
#[derive(Debug, Clone, Copy)]
pub struct RawHeader {
    pub length: u64,
    pub tag: u8,
}

pub const HEADER_SIZE: usize = std::mem::size_of::<RawHeader>();



#[inline]
fn write_u64_le(dst: &mut Vec<u8>, v: u64) {
    dst.extend_from_slice(&v.to_le_bytes());
}
#[inline]
fn read_u64_le(src: &[u8]) -> u64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&src[..8]);
    u64::from_le_bytes(a)
}
#[inline]
fn write_i64_le(dst: &mut Vec<u8>, v: i64) {
    dst.extend_from_slice(&v.to_le_bytes());
}
#[inline]
fn read_i64_le(src: &[u8]) -> i64 {
    let mut a = [0u8; 8];
    a.copy_from_slice(&src[..8]);
    i64::from_le_bytes(a)
}


pub fn serialize_value(buf: &mut Vec<u8>, value: &Value) -> usize {
    let start_pos = buf.len();

    // Reserve room: [length:u64][checksum:u32][tag:u8] + payload
    // First, push a placeholder for len to fill later:
    let length_pos = buf.len();
    write_u64_le(buf, 0); // placeholder

    // Tag
    match value {
        Value::Int(i) => {
            buf.push(TAG_INT);
            write_i64_le(buf, *i);
        }
        Value::Bool(b) => {
            buf.push(TAG_BOOL);
            buf.push(if *b { 1 } else { 0 });
        }
        Value::Text(s) => {
            buf.push(TAG_TEXT);
            let bytes = s.as_bytes();
            write_u64_le(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
        Value::Blob(bytes) => {
            buf.push(TAG_BLOB);
            write_u64_le(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
    }

    // total_len = (current_len - after_len_field)
    let after_len = length_pos + 8;
    let total_len = (buf.len() - after_len) as u64;
    // write it back
    buf[length_pos..length_pos + 8].copy_from_slice(&total_len.to_le_bytes());

    start_pos
}


/// Returns (borrowed_value, total_entry_len_bytes)
pub fn deserialize_borrowed<'a>(src: &'a [u8]) -> Option<(BorrowedValue<'a>, usize)> {
    // need at least 8 (len) + 1 (tag)
    if src.len() < 9 {
        return None;
    }

    let total_len = read_u64_le(&src[..8]) as usize;
    // total_len counts tag + payload, so full entry size is 8 + total_len
    if total_len == 0 {
        return None;
    } // invalid: must contain at least the tag
    if src.len() < 8 + total_len {
        return None;
    } // ensure full entry is available

    let tag = src[8];
    let payload_len = total_len - 1; // exclude tag
    let payload = &src[9..9 + payload_len];

    match tag {
        TAG_INT => {
            if payload.len() < 8 {
                return None;
            }
            Some((
                BorrowedValue::Int(read_i64_le(&payload[..8])),
                8 + total_len,
            ))
        }
        TAG_BOOL => {
            if payload.is_empty() {
                return None;
            }
            Some((BorrowedValue::Bool(payload[0] != 0), 8 + total_len))
        }
        TAG_TEXT | TAG_BLOB => {
            if payload.len() < 8 {
                return None;
            }
            let n = read_u64_le(&payload[..8]) as usize;
            if payload.len() < 8 + n {
                return None;
            }
            let bytes = &payload[8..8 + n];
            if tag == TAG_TEXT {
                let s = std::str::from_utf8(bytes).ok()?;
                Some((BorrowedValue::Text(s), 8 + total_len))
            } else {
                Some((BorrowedValue::Blob(bytes), 8 + total_len))
            }
        }
        _ => None,
    }
}