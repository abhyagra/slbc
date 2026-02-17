//! Numeral encoding and decoding (§6.3).
//!
//! Bhāṣā layer: SAṄKHYĀ_START + ULEB128 count + R→L digit-word padas.
//! Lipi layer: NUM + L→R digit glyphs.

use crate::types::*;
use crate::container;

/// The closed digit-word vocabulary (§6.3.2).
/// Each entry is the SLBC bhāṣā encoding of the prātipadika.
const DIGIT_WORDS: [&[u8]; 10] = [
    &[0x29, 0x88, 0x1C, 0x31, 0x40], // 0: śūnya
    &[0x85, 0x00, 0x40],             // 1: eka
    &[0x1A, 0x32, 0x44],             // 2: dvi
    &[0x18, 0x33, 0x44],             // 3: tri
    &[0x08, 0x40, 0x18, 0x48, 0x33], // 4: catur
    &[0x20, 0x40, 0x0C, 0x08, 0x40], // 5: pañca
    &[0x2A, 0x40, 0x2A],             // 6: ṣaṣ
    &[0x2B, 0x40, 0x20, 0x18, 0x40], // 7: sapta
    &[0x40, 0x2A, 0x10, 0x40],       // 8: aṣṭa
    &[0x1C, 0x40, 0x32, 0x40],       // 9: nava
];

/// Encode a numeral string (e.g. "108") into both SAṄKHYĀ and NUM spans.
pub fn encode_numeral(digits: &str, out: &mut Vec<u8>) {
    let digit_chars: Vec<u32> = digits
        .chars()
        .map(|c| c.to_digit(10).expect("non-digit in numeral"))
        .collect();

    let count = digit_chars.len();

    // ── Bhāṣā layer: SAṄKHYĀ span ──
    out.push(SANKHYA_START);
    container::write_uleb128(out, count as u64);

    // Emit digits R→L (units first) per aṅkānāṃ vāmato gatiḥ
    for &d in digit_chars.iter().rev() {
        out.push(PADA_START);
        out.extend_from_slice(DIGIT_WORDS[d as usize]);
        out.push(PADA_END);
    }

    // ── Lipi layer: NUM span ──
    out.push(NUM);
    // Digit glyphs L→R (visual order)
    for &d in &digit_chars {
        out.push(d as u8); // 0x00–0x09
    }
    // Termination is implicit: next byte ≥ 0x10 exits the span
}

/// Decode a SAṄKHYĀ span from a byte slice starting at `pos`.
/// Returns (digit_vector_L2R, bytes_consumed).
pub fn decode_sankhya(data: &[u8], pos: usize) -> Result<(Vec<u8>, usize), String> {
    let mut i = pos;

    if data[i] != SANKHYA_START {
        return Err(format!("expected SAṄKHYĀ_START at offset {}", i));
    }
    i += 1;

    let (count, consumed) = container::read_uleb128(&data[i..])
        .map_err(|e| format!("ULEB128 error at offset {}: {}", i, e))?;
    i += consumed;

    let mut digits = Vec::with_capacity(count as usize);

    for _ in 0..count {
        if data[i] != PADA_START {
            return Err(format!("expected PADA_START at offset {}", i));
        }
        i += 1;

        // Find PADA_END
        let pada_start = i;
        while i < data.len() && data[i] != PADA_END {
            i += 1;
        }
        if i >= data.len() {
            return Err("unterminated digit-pada".into());
        }
        let pada_bytes = &data[pada_start..i];
        i += 1; // skip PADA_END

        let digit = lookup_digit_word(pada_bytes)
            .ok_or_else(|| format!("invalid digit-word at offset {}", pada_start))?;
        digits.push(digit);
    }

    // Reverse: R→L encoding → L→R value
    digits.reverse();

    Ok((digits, i - pos))
}

/// Decode a NUM (digit-glyph) span from a byte slice starting at `pos`.
/// Returns (digit_vector_L2R, bytes_consumed).
pub fn decode_num(data: &[u8], pos: usize) -> Result<(Vec<u8>, usize), String> {
    let mut i = pos;

    if data[i] != NUM {
        return Err(format!("expected NUM at offset {}", i));
    }
    i += 1;

    let mut digits = Vec::new();
    while i < data.len() && data[i] < 0x10 {
        digits.push(data[i]);
        i += 1;
    }

    Ok((digits, i - pos))
}

/// Look up a pada's byte content against the digit-word vocabulary.
/// Returns the digit value (0–9) or None.
fn lookup_digit_word(pada_bytes: &[u8]) -> Option<u8> {
    for (digit, &word) in DIGIT_WORDS.iter().enumerate() {
        if pada_bytes == word {
            return Some(digit as u8);
        }
    }
    None
}

/// IAST names for digits (for inspection / decode display).
pub const DIGIT_IAST: [&str; 10] = [
    "śūnya", "eka", "dvi", "tri", "catur",
    "pañca", "ṣaṣ", "sapta", "aṣṭa", "nava",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_108() {
        let mut out = Vec::new();
        encode_numeral("108", &mut out);

        // Should start with SAṄKHYĀ_START
        assert_eq!(out[0], SANKHYA_START);
        // Count = 3
        assert_eq!(out[1], 0x03);
        // First digit-pada should be aṣṭa (8, units)
        assert_eq!(out[2], PADA_START);

        // Should contain NUM span
        assert!(out.contains(&NUM));
    }

    #[test]
    fn test_roundtrip_sankhya() {
        let mut out = Vec::new();
        encode_numeral("108", &mut out);

        let (digits, _) = decode_sankhya(&out, 0).unwrap();
        assert_eq!(digits, vec![1, 0, 8]);
    }
}
