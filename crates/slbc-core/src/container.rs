//! Container format (.slbc) — header, chunk framing, ULEB128.
//!
//! §7: 14-byte header + chunk sequence + EOF chunk.

use crate::types::*;

// ── ULEB128 ──

/// Encode a u64 as ULEB128, appending to `out`.
pub fn write_uleb128(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80; // continuation bit
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
}

/// Decode a ULEB128 from a byte slice.
/// Returns (value, bytes_consumed).
pub fn read_uleb128(data: &[u8]) -> Result<(u64, usize), String> {
    let mut result: u64 = 0;
    let mut shift = 0;

    for (i, &byte) in data.iter().enumerate() {
        if i >= 5 {
            return Err("ULEB128 exceeds 5 bytes (max u32)".into());
        }
        result |= ((byte & 0x7F) as u64) << shift;
        shift += 7;
        if byte & 0x80 == 0 {
            if result > u32::MAX as u64 {
                return Err("ULEB128 value exceeds u32 range".into());
            }
            return Ok((result, i + 1));
        }
    }

    Err("truncated ULEB128".into())
}

// ── Header ──

/// Build a 14-byte .slbc header for pāṭha mode.
pub fn build_header(has_lipi: bool, has_meta_markers: bool, interleaved: bool) -> [u8; 14] {
    let mut header = [0u8; 14];

    // Magic
    header[0..4].copy_from_slice(MAGIC);

    // Version
    header[4..8].copy_from_slice(&VERSION);

    // Flags: bytes 8–10 reserved (0x00), byte 11 has flags
    let mut flags: u8 = 0;
    if has_lipi {
        flags |= FLAG_HAS_LIPI;
    }
    if has_meta_markers {
        flags |= FLAG_HAS_META;
    }
    if interleaved {
        flags |= FLAG_INTERLEAVED;
    }
    header[11] = flags;

    // Extended header length: 0x0000 (no extension)
    header[12] = 0x00;
    header[13] = 0x00;

    header
}

/// Write a chunk: type + ULEB128 payload length + payload bytes.
pub fn write_chunk(out: &mut Vec<u8>, chunk_type: u8, payload: &[u8]) {
    out.push(chunk_type);
    write_uleb128(out, payload.len() as u64);
    out.extend_from_slice(payload);
}

/// Write the EOF chunk (type=0xFF, length=0).
pub fn write_eof(out: &mut Vec<u8>) {
    out.push(CHUNK_EOF);
    out.push(0x00); // ULEB128 for 0
}

/// Build a complete .slbc file from a PHON payload (pāṭha mode).
pub fn build_slbc(phon_payload: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();

    // Header: pāṭha mode with lipi controls, meta markers, interleaved
    let header = build_header(true, true, true);
    out.extend_from_slice(&header);

    // PHON chunk
    write_chunk(&mut out, CHUNK_PHON, phon_payload);

    // EOF
    write_eof(&mut out);

    out
}

// ── Parsing ──

/// Parsed container header.
#[derive(Debug)]
pub struct SlbcHeader {
    pub version: [u8; 4],
    pub flags: u8,
    pub extended_header_len: u16,
}

impl SlbcHeader {
    pub fn has_lipi(&self) -> bool {
        self.flags & FLAG_HAS_LIPI != 0
    }
    pub fn has_meta(&self) -> bool {
        self.flags & FLAG_HAS_META != 0
    }
    pub fn is_interleaved(&self) -> bool {
        self.flags & FLAG_INTERLEAVED != 0
    }
    pub fn is_vedic(&self) -> bool {
        self.flags & FLAG_VEDIC != 0
    }
    pub fn has_vya(&self) -> bool {
        self.flags & FLAG_VYA != 0
    }
}

/// A parsed chunk.
#[derive(Debug)]
pub struct Chunk {
    pub chunk_type: u8,
    pub payload: Vec<u8>,
}

/// Parse a .slbc file into header + chunks.
pub fn parse_slbc(data: &[u8]) -> Result<(SlbcHeader, Vec<Chunk>), String> {
    if data.len() < 14 {
        return Err("file too short for SLBC header".into());
    }

    // Verify magic
    if &data[0..4] != MAGIC {
        return Err("invalid magic bytes (expected 'SLBC')".into());
    }

    let mut version = [0u8; 4];
    version.copy_from_slice(&data[4..8]);

    let flags = data[11];
    let ext_len = u16::from_le_bytes([data[12], data[13]]);

    let header = SlbcHeader {
        version,
        flags,
        extended_header_len: ext_len,
    };

    // Skip extended header
    let mut pos = 14 + ext_len as usize;
    let mut chunks = Vec::new();

    // Parse chunks
    while pos < data.len() {
        let chunk_type = data[pos];
        pos += 1;

        let (payload_len, consumed) = read_uleb128(&data[pos..])
            .map_err(|e| format!("chunk length ULEB128 error at offset {}: {}", pos, e))?;
        pos += consumed;

        let payload_len = payload_len as usize;
        if pos + payload_len > data.len() {
            return Err(format!(
                "chunk payload extends beyond file (offset {}, len {})",
                pos, payload_len
            ));
        }

        let payload = data[pos..pos + payload_len].to_vec();
        pos += payload_len;

        let is_eof = chunk_type == CHUNK_EOF;
        chunks.push(Chunk {
            chunk_type,
            payload,
        });

        if is_eof {
            break;
        }
    }

    Ok((header, chunks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uleb128_roundtrip() {
        for val in [0u64, 1, 127, 128, 300, 16383, 16384, 100_000] {
            let mut buf = Vec::new();
            write_uleb128(&mut buf, val);
            let (decoded, consumed) = read_uleb128(&buf).unwrap();
            assert_eq!(decoded, val);
            assert_eq!(consumed, buf.len());
        }
    }

    #[test]
    fn test_header_magic() {
        let header = build_header(true, true, true);
        assert_eq!(&header[0..4], b"SLBC");
    }

    #[test]
    fn test_roundtrip_container() {
        let payload = vec![0x26, 0x00, 0x40, 0x2E]; // PADA_START ka a PADA_END
        let slbc = build_slbc(&payload);
        let (header, chunks) = parse_slbc(&slbc).unwrap();
        assert!(header.has_lipi());
        assert_eq!(chunks.len(), 2); // PHON + EOF
        assert_eq!(chunks[0].chunk_type, CHUNK_PHON);
        assert_eq!(chunks[0].payload, payload);
        assert_eq!(chunks[1].chunk_type, CHUNK_EOF);
    }
}
