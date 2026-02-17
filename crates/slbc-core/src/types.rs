//! Core types and byte constants for SLBC.
//!
//! All byte values derive from the spec §2–§6.
//! Vyañjana: `00 PLACE[3] COLUMN[3]`
//! Svara:    `Q[2] A[2] S[2] G[2]`

// ── Bhāṣā Control Bytes (COLUMN = 110) ──

pub const META_START: u8 = 0x06;
pub const META_END: u8 = 0x0E;
pub const PHON_START: u8 = 0x16;
pub const PHON_END: u8 = 0x1E;
pub const PADA_START: u8 = 0x26;
pub const PADA_END: u8 = 0x2E;
// 0x36 reserved
pub const SANKHYA_START: u8 = 0x3E;

// ── Lipi Control Bytes (COLUMN = 111) ──

// 0x07 reserved
pub const DANDA: u8 = 0x0F;
pub const DOUBLE_DANDA: u8 = 0x17;
pub const SPACE: u8 = 0x1F;
pub const AVAGRAHA: u8 = 0x27;
pub const NUM: u8 = 0x2F;
pub const META_EXT: u8 = 0x37;
// 0x3F reserved

// ── Chunk Types (§7.3) ──

pub const CHUNK_PHON: u8 = 0x01;
pub const CHUNK_BHA: u8 = 0x02;
pub const CHUNK_LIPI: u8 = 0x03;
pub const CHUNK_META: u8 = 0x04;
pub const CHUNK_DICT: u8 = 0x05;
pub const CHUNK_IDX: u8 = 0x06;
pub const CHUNK_ANVY: u8 = 0x07;
pub const CHUNK_EXT: u8 = 0x10;
pub const CHUNK_EOF: u8 = 0xFF;

// ── Container ──

pub const MAGIC: &[u8; 4] = b"SLBC";
pub const VERSION: [u8; 4] = [0x00, 0x00, 0x00, 0x0A]; // v0.10

// ── Flag bits (byte 11) ──

pub const FLAG_HAS_LIPI: u8 = 0b1000_0000;
pub const FLAG_HAS_META: u8 = 0b0100_0000;
pub const FLAG_INTERLEAVED: u8 = 0b0010_0000;
pub const FLAG_VEDIC: u8 = 0b0001_0000;
pub const FLAG_VYA: u8 = 0b0000_1000;

// ── Byte Classification (§2) ──

/// Returns true if the byte is a svara (bits[7:6] ≠ 00).
#[inline]
pub fn is_svara(b: u8) -> bool {
    (b >> 6) != 0
}

/// Returns true if the byte is a vyañjana (bits[7:6] = 00, COLUMN ∈ 0–4).
#[inline]
pub fn is_vyanjana(b: u8) -> bool {
    (b >> 6) == 0 && (b & 0x07) <= 4
}

/// Returns true if the byte is a varga consonant (PLACE ∈ 0–4).
#[inline]
pub fn is_varga(b: u8) -> bool {
    (b >> 6) == 0 && ((b >> 3) & 0x07) <= 4 && (b & 0x07) <= 4
}

/// Returns true if the byte is a bhāṣā control (COLUMN = 110).
#[inline]
pub fn is_bhasha_control(b: u8) -> bool {
    (b >> 6) == 0 && (b & 0x07) == 6
}

/// Returns true if the byte is a lipi control (COLUMN = 111).
#[inline]
pub fn is_lipi_control(b: u8) -> bool {
    (b >> 6) == 0 && (b & 0x07) == 7
}

/// Extract PLACE field from a vyañjana byte.
#[inline]
pub fn place(b: u8) -> u8 {
    (b >> 3) & 0x07
}

/// Extract COLUMN field from a vyañjana byte.
#[inline]
pub fn column(b: u8) -> u8 {
    b & 0x07
}

/// Extract Q (quantity) field from a svara byte.
#[inline]
pub fn svara_q(b: u8) -> u8 {
    (b >> 6) & 0x03
}

/// Extract A (accent) field from a svara byte.
#[inline]
pub fn svara_a(b: u8) -> u8 {
    (b >> 4) & 0x03
}

/// Extract S (series) field from a svara byte.
#[inline]
pub fn svara_s(b: u8) -> u8 {
    (b >> 2) & 0x03
}

/// Extract G (grade) field from a svara byte.
#[inline]
pub fn svara_g(b: u8) -> u8 {
    b & 0x03
}
