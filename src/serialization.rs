use crate::{BorrowedValue, Result, Value, anyhow};
use crc::{CRC_32_ISO_HDLC, Crc};

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct RawHeader {
    pub total_len: u64, // body length
    pub checksum: u32,  // CRC32 of body
    pub tag: u8,        // TAG_INT / TAG_TEXT / TAG_BOOL / TAG_BLOB
}

pub const RAW_HEADER_SIZE: usize = std::mem::size_of::<RawHeader>();

/// On-disk type tags (1 byte)
const TAG_INT: u8 = 1;
const TAG_TEXT: u8 = 2;
const TAG_BOOL: u8 = 3;
const TAG_BLOB: u8 = 4;

/// Entry header layout (little-endian):
/// [total_len: u64][checksum: u32][tag: u8]  followed by the tagged payload:
/// - Int:  [i64: 8]
/// - Bool: [u8: 1]
/// - Text: [len: u64][bytes: len]
/// - Blob: [len: u64][bytes: len]
///
/// Total header size: 13 bytes (`RAW_HEADER_SIZE`)
///
/// `total_len` counts the number of bytes in the payload (after the header),
/// and `checksum` is the CRC32 checksum of that payload. The `tag` identifies
/// the data type of the value being serialized.

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

/// SAFETY:
/// Caller must ensure that `dst` is valid for at least `RAW_HEADER_SIZE` bytes.
#[inline]
unsafe fn write_header_unchecked(dst: *mut u8, hdr: &RawHeader) {
    // SAFETY: caller guarantees dst..dst+RAW_HEADER_SIZE is valid and aligned enough for byte writes.
    unsafe {
        std::ptr::copy_nonoverlapping((hdr as *const RawHeader).cast::<u8>(), dst, RAW_HEADER_SIZE);
    }
}

/// SAFETY:
/// Caller must ensure that `src` is valid for at least `RAW_HEADER_SIZE` bytes.
#[inline]
unsafe fn read_header_unchecked(src: *const u8) -> RawHeader {
    // SAFETY: caller guarantees src..src+RAW_HEADER_SIZE is readable and may be unaligned.
    unsafe { std::ptr::read_unaligned(src.cast::<RawHeader>()) }
}

/// SAFETY:
/// The caller must ensure that `buf` is a valid, writable `Vec<u8>`.
/// This function performs unchecked pointer writes but internally resizes
/// the buffer to make space for the header.
pub unsafe fn serialize_unsafe(buf: &mut Vec<u8>, header: &RawHeader) -> usize {
    let start = buf.len();
    buf.resize(start + RAW_HEADER_SIZE, 0);
    // SAFETY: `buf` was resized to at least `start + RAW_HEADER_SIZE`, so writing
    // `RAW_HEADER_SIZE` bytes starting at `dst` is valid. We use an unaligned write.
    let dst = unsafe { buf.as_mut_ptr().add(start) };
    unsafe { write_header_unchecked(dst, header) };
    start
}

/// SAFETY:
/// The caller must ensure that `src` points to at least `RAW_HEADER_SIZE` readable bytes.
/// This function reads a `RawHeader` directly from the slice without alignment checks.
pub unsafe fn deserialize_unsafe(src: &[u8]) -> Result<RawHeader> {
    unsafe {
        if src.len() < RAW_HEADER_SIZE {
            return Err(anyhow!(
                "incomplete header: have {} bytes, need {}",
                src.len(),
                RAW_HEADER_SIZE
            ));
        }
        // SAFETY: caller promises src points to at least RAW_HEADER_SIZE bytes
        let header = read_header_unchecked(src.as_ptr());
        Ok(header)
    }
}

/// Serialize a `Value` into `buf`, returning the starting offset of the serialized entry.
pub fn serialize_value(buf: &mut Vec<u8>, value: &Value) -> usize {
    let start = buf.len();

    // 1) Reserve header space
    buf.resize(buf.len() + RAW_HEADER_SIZE, 0);

    // 2) Write Body (payload only; tag moved into header now)
    let body_start = buf.len();
    match value {
        Value::Int(i) => {
            // payload: i64
            write_i64_le(buf, *i);
            // tag is TAG_INT (in header)
        }
        Value::Bool(b) => {
            buf.push(if *b { 1 } else { 0 });
        }
        Value::Text(s) => {
            let bytes = s.as_bytes();
            write_u64_le(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
        Value::Blob(bytes) => {
            write_u64_le(buf, bytes.len() as u64);
            buf.extend_from_slice(bytes);
        }
    }
    let body_len = buf.len() - body_start;

    // 3) Build header (tag depends on the variant)
    let tag = match value {
        Value::Int(_) => TAG_INT,
        Value::Bool(_) => TAG_BOOL,
        Value::Text(_) => TAG_TEXT,
        Value::Blob(_) => TAG_BLOB,
    };

    let header = RawHeader {
        total_len: body_len as u64,
        checksum: crc32(&buf[body_start..]),
        tag,
    };

    // SAFETY: `buf` was just resized to make room for the header, so writing
    // `RAW_HEADER_SIZE` bytes into it is valid and properly aligned.
    unsafe {
        let dst = buf.as_mut_ptr().add(start);
        write_header_unchecked(dst, &header);
    }

    start
}

/// Returns (BorrowedValue, full_entry_size_including_header)
pub fn deserialize_borrowed<'a>(src: &'a [u8]) -> Result<(BorrowedValue<'a>, usize)> {
    // Need header first
    if src.len() < RAW_HEADER_SIZE {
        return Err(anyhow!(
            "incomplete entry header: have {} bytes, need {}",
            src.len(),
            RAW_HEADER_SIZE
        ));
    }

    // SAFETY: `src` is validated to be at least `RAW_HEADER_SIZE` bytes above,
    // so deserializing a `RawHeader` from it is safe.
    let header = unsafe { deserialize_unsafe(src)? };

    let body_len = header.total_len as usize;
    let full_size = RAW_HEADER_SIZE + body_len;

    if src.len() < full_size {
        return Err(anyhow!(
            "incomplete entry body: header says {} bytes, but buffer has {}",
            body_len,
            src.len().saturating_sub(RAW_HEADER_SIZE),
        ));
    }

    let body = &src[RAW_HEADER_SIZE..full_size];

    // 2) Verify checksum before parsing
    if crc32(body) != header.checksum {
        return Err(anyhow!("checksum mismatch in entry body"));
    }

    // 3) Decode by tag from header, parsing the Body only
    match header.tag {
        TAG_INT => {
            if body.len() < 8 {
                return Err(anyhow!(
                    "truncated int payload: need 8 bytes, have {}",
                    body.len()
                ));
            }
            let i = read_i64_le(&body[..8]);
            Ok((BorrowedValue::Int(i), full_size))
        }
        TAG_BOOL => {
            if body.is_empty() {
                return Err(anyhow!("truncated bool payload: need 1 byte, have 0"));
            }
            Ok((BorrowedValue::Bool(body[0] != 0), full_size))
        }
        TAG_TEXT | TAG_BLOB => {
            if body.len() < 8 {
                return Err(anyhow!(
                    "truncated length prefix: need 8 bytes, have {}",
                    body.len()
                ));
            }
            let n = read_u64_le(&body[..8]) as usize;
            if body.len() < 8 + n {
                return Err(anyhow!(
                    "truncated payload: length prefix says {} bytes, but body has {}",
                    n,
                    body.len().saturating_sub(8),
                ));
            }
            let bytes = &body[8..8 + n];
            if header.tag == TAG_TEXT {
                let s = std::str::from_utf8(bytes)
                    .map_err(|e| anyhow!("invalid utf-8 in text value: {e}"))?;
                Ok((BorrowedValue::Text(s), full_size))
            } else {
                Ok((BorrowedValue::Blob(bytes), full_size))
            }
        }
        _ => Err(anyhow!("unknown tag {}", header.tag)),
    }
}

/// CRC32 computation
const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

#[inline]
fn crc32(bytes: &[u8]) -> u32 {
    CRC32.checksum(bytes)
}
