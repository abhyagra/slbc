//! SLBC → IAST / Devanāgarī decoder.
//!
//! Walks a PHON chunk payload byte-by-byte, emitting text.
//! Devanāgarī output follows §4.2 explicit vowel convention.

use crate::types::*;
use crate::numeral;

/// Output script target.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Script {
    Iast,
    Devanagari,
}

/// Decode a PHON chunk payload to text.
pub fn decode_phon(payload: &[u8], script: Script) -> Result<String, String> {
    match script {
        Script::Iast => decode_to_iast(payload),
        Script::Devanagari => decode_to_devanagari(payload),
    }
}

// ═══════════════════════════════════════════════
//  IAST decoder
// ═══════════════════════════════════════════════

fn decode_to_iast(data: &[u8]) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0;

    while i < data.len() {
        let b = data[i];

        // ── Bhāṣā controls ──
        if is_bhasha_control(b) {
            match b {
                PADA_START | PADA_END | PHON_START | PHON_END => {
                    i += 1;
                    continue;
                }
                META_START => {
                    // Skip META block (not present in pāṭha, but defensive)
                    i += 1;
                    while i < data.len() && data[i] != META_END {
                        i += 1;
                    }
                    i += 1; // skip META_END
                    continue;
                }
                SANKHYA_START => {
                    let (digits, consumed) = numeral::decode_sankhya(data, i)?;
                    for d in &digits {
                        out.push(char::from_digit(*d as u32, 10).unwrap());
                    }
                    i += consumed;
                    // Skip the following NUM span (lipi-layer)
                    if i < data.len() && data[i] == NUM {
                        let (_, num_consumed) = numeral::decode_num(data, i)?;
                        i += num_consumed;
                    }
                    continue;
                }
                _ => {
                    i += 1;
                    continue;
                }
            }
        }

        // ── Lipi controls ──
        if is_lipi_control(b) {
            match b {
                SPACE => out.push(' '),
                DANDA => out.push('|'),
                DOUBLE_DANDA => out.push_str("||"),
                AVAGRAHA => out.push('\''),
                NUM => {
                    // Standalone NUM span (shouldn't appear without SAṄKHYĀ in pāṭha,
                    // but handle gracefully)
                    let (digits, consumed) = numeral::decode_num(data, i)?;
                    for d in &digits {
                        out.push(char::from_digit(*d as u32, 10).unwrap());
                    }
                    i += consumed;
                    continue;
                }
                _ => {}
            }
            i += 1;
            continue;
        }

        // ── Svara ──
        if is_svara(b) {
            out.push_str(byte_to_iast(b));
            i += 1;
            continue;
        }

        // ── Vyañjana ──
        if is_vyanjana(b) {
            out.push_str(byte_to_iast(b));
            i += 1;
            continue;
        }

        // Unknown byte
        return Err(format!("unexpected byte 0x{:02X} at offset {}", b, i));
    }

    Ok(out)
}

// ═══════════════════════════════════════════════
//  Devanāgarī decoder
// ═══════════════════════════════════════════════

fn decode_to_devanagari(data: &[u8]) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0;
    let mut consonant_pending = false;

    while i < data.len() {
        let b = data[i];

        // ── Bhāṣā controls ──
        if is_bhasha_control(b) {
            match b {
                PADA_START => {
                    i += 1;
                    continue;
                }
                PADA_END => {
                    // Pada end: if consonant pending, add virāma
                    if consonant_pending {
                        out.push('्');
                        consonant_pending = false;
                    }
                    i += 1;
                    continue;
                }
                PHON_START | PHON_END => {
                    i += 1;
                    continue;
                }
                META_START => {
                    i += 1;
                    while i < data.len() && data[i] != META_END {
                        i += 1;
                    }
                    i += 1;
                    continue;
                }
                SANKHYA_START => {
                    if consonant_pending {
                        out.push('्');
                        consonant_pending = false;
                    }
                    // Skip SAṄKHYĀ span (bhāṣā layer), use NUM span for glyphs
                    let (_, consumed) = numeral::decode_sankhya(data, i)?;
                    i += consumed;
                    // Now read the NUM span for Devanāgarī digit glyphs
                    if i < data.len() && data[i] == NUM {
                        i += 1; // skip NUM marker
                        while i < data.len() && data[i] < 0x10 {
                            out.push(DEVANAGARI_DIGITS[data[i] as usize]);
                            i += 1;
                        }
                    }
                    continue;
                }
                _ => {
                    i += 1;
                    continue;
                }
            }
        }

        // ── Lipi controls ──
        if is_lipi_control(b) {
            if consonant_pending {
                out.push('्');
                consonant_pending = false;
            }
            match b {
                SPACE => out.push(' '),
                DANDA => out.push('।'),
                DOUBLE_DANDA => out.push_str("॥"),
                AVAGRAHA => out.push('ऽ'),
                NUM => {
                    i += 1;
                    while i < data.len() && data[i] < 0x10 {
                        out.push(DEVANAGARI_DIGITS[data[i] as usize]);
                        i += 1;
                    }
                    continue;
                }
                _ => {}
            }
            i += 1;
            continue;
        }

        // ── Svara ──
        if is_svara(b) {
            if consonant_pending {
                // Consonant + vowel: use mātrā (or bare for 'a')
                if b == 0x40 {
                    // 'a': inherent vowel — no mātrā
                } else if let Some(matra) = byte_to_devanagari_matra(b) {
                    out.push_str(matra);
                }
                consonant_pending = false;
            } else {
                // Standalone vowel: independent form
                out.push_str(byte_to_devanagari_independent(b));
            }
            i += 1;
            continue;
        }

        // ── Vyañjana ──
        if is_vyanjana(b) {
            // Visarga and anusvāra render as postfix marks, not as consonants
            if is_postfix_mark(b) {
                if consonant_pending {
                    // Consonant + visarga/anusvāra: no virāma needed
                    consonant_pending = false;
                }
                out.push_str(postfix_mark_devanagari(b));
                i += 1;
                continue;
            }

            if consonant_pending {
                // Consecutive consonants: insert virāma before new consonant
                out.push('्');
            }
            out.push_str(byte_to_devanagari_consonant(b));
            consonant_pending = true;
            i += 1;
            continue;
        }

        return Err(format!("unexpected byte 0x{:02X} at offset {}", b, i));
    }

    // Trailing consonant at end of stream
    if consonant_pending {
        out.push('्');
    }

    Ok(out)
}

// ═══════════════════════════════════════════════
//  IAST lookup table
// ═══════════════════════════════════════════════

/// Map an SLBC byte to its IAST representation.
pub fn byte_to_iast(b: u8) -> &'static str {
    if is_svara(b) {
        return svara_to_iast(b);
    }
    if is_vyanjana(b) {
        return vyanjana_to_iast(b);
    }
    "?"
}

fn svara_to_iast(b: u8) -> &'static str {
    // Mask out accent bits for lookup (A field = bits 5:4)
    let base = b & 0b11_00_11_11; // zero out accent
    match base {
        0x40 => "a",
        0x80 => "ā",
        0x44 => "i",
        0x84 => "ī",
        0x48 => "u",
        0x88 => "ū",
        0x4C => "ṛ",
        0x8C => "ṝ",
        0x4F => "ḷ",
        0x8F => "ḹ",
        0x85 => "e",
        0x86 => "ai",
        0x89 => "o",
        0x8A => "au",
        _ => "?",
    }
}

fn vyanjana_to_iast(b: u8) -> &'static str {
    match b {
        0x00 => "k",  0x01 => "kh", 0x02 => "g",  0x03 => "gh", 0x04 => "ṅ",
        0x08 => "c",  0x09 => "ch", 0x0A => "j",  0x0B => "jh", 0x0C => "ñ",
        0x10 => "ṭ",  0x11 => "ṭh", 0x12 => "ḍ",  0x13 => "ḍh", 0x14 => "ṇ",
        0x18 => "t",  0x19 => "th", 0x1A => "d",  0x1B => "dh", 0x1C => "n",
        0x20 => "p",  0x21 => "ph", 0x22 => "b",  0x23 => "bh", 0x24 => "m",
        0x29 => "ś",  0x2A => "ṣ",  0x2B => "s",
        0x31 => "y",  0x32 => "v",  0x33 => "r",  0x34 => "l",
        0x38 => "h",  0x39 => "ḥ",  0x3A => "ṃ",  0x3B => "ẖ",  0x3C => "ḫ",
        _ => "?",
    }
}

// ═══════════════════════════════════════════════
//  Devanāgarī tables
// ═══════════════════════════════════════════════

const DEVANAGARI_DIGITS: [char; 10] = [
    '०', '१', '२', '३', '४', '५', '६', '७', '८', '९',
];

fn byte_to_devanagari_consonant(b: u8) -> &'static str {
    match b {
        0x00 => "क",  0x01 => "ख",  0x02 => "ग",  0x03 => "घ",  0x04 => "ङ",
        0x08 => "च",  0x09 => "छ",  0x0A => "ज",  0x0B => "झ",  0x0C => "ञ",
        0x10 => "ट",  0x11 => "ठ",  0x12 => "ड",  0x13 => "ढ",  0x14 => "ण",
        0x18 => "त",  0x19 => "थ",  0x1A => "द",  0x1B => "ध",  0x1C => "न",
        0x20 => "प",  0x21 => "फ",  0x22 => "ब",  0x23 => "भ",  0x24 => "म",
        0x29 => "श",  0x2A => "ष",  0x2B => "स",
        0x31 => "य",  0x32 => "व",  0x33 => "र",  0x34 => "ल",
        0x38 => "ह",
        _ => "?",
    }
}

fn byte_to_devanagari_independent(b: u8) -> &'static str {
    let base = b & 0b11_00_11_11;
    match base {
        0x40 => "अ",  0x80 => "आ",
        0x44 => "इ",  0x84 => "ई",
        0x48 => "उ",  0x88 => "ऊ",
        0x4C => "ऋ",  0x8C => "ॠ",
        0x4F => "ऌ",  0x8F => "ॡ",
        0x85 => "ए",  0x86 => "ऐ",
        0x89 => "ओ",  0x8A => "औ",
        _ => "?",
    }
}

fn byte_to_devanagari_matra(b: u8) -> Option<&'static str> {
    let base = b & 0b11_00_11_11;
    match base {
        0x40 => None, // 'a' — inherent, no mātrā
        0x80 => Some("ा"),
        0x44 => Some("ि"),
        0x84 => Some("ी"),
        0x48 => Some("ु"),
        0x88 => Some("ू"),
        0x4C => Some("ृ"),
        0x8C => Some("ॄ"),
        0x4F => Some("ॢ"),
        0x8F => Some("ॣ"),
        0x85 => Some("े"),
        0x86 => Some("ै"),
        0x89 => Some("ो"),
        0x8A => Some("ौ"),
        _ => None,
    }
}

/// Handle visarga and anusvāra in Devanāgarī context.
/// These are technically vyañjana bytes but render as post-vowel marks.
fn is_postfix_mark(b: u8) -> bool {
    matches!(b, 0x39 | 0x3A) // visarga, anusvāra
}

/// Devanāgarī rendering of visarga/anusvāra (appended after vowel).
fn postfix_mark_devanagari(b: u8) -> &'static str {
    match b {
        0x39 => "ः",  // visarga
        0x3A => "ं",  // anusvāra
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder;

    #[test]
    fn test_iast_roundtrip_simple() {
        let input = "dharma";
        let bytes = encoder::encode_iast(input).unwrap();
        let output = decode_phon(&bytes, Script::Iast).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_iast_roundtrip_multi_word() {
        let input = "na ca";
        let bytes = encoder::encode_iast(input).unwrap();
        let output = decode_phon(&bytes, Script::Iast).unwrap();
        assert_eq!(output, input);
    }

    #[test]
    fn test_devanagari_ka() {
        // ka = 0x00(k) + 0x40(a) → क
        let bytes = encoder::encode_iast("ka").unwrap();
        let output = decode_phon(&bytes, Script::Devanagari).unwrap();
        assert_eq!(output, "क");
    }

    #[test]
    fn test_devanagari_ki() {
        // ki = 0x00(k) + 0x44(i) → कि
        let bytes = encoder::encode_iast("ki").unwrap();
        let output = decode_phon(&bytes, Script::Devanagari).unwrap();
        assert_eq!(output, "कि");
    }

    #[test]
    fn test_devanagari_cluster() {
        // kṛ = k + ṛ → क + ृ = कृ
        let bytes = encoder::encode_iast("kṛ").unwrap();
        let output = decode_phon(&bytes, Script::Devanagari).unwrap();
        assert_eq!(output, "कृ");
    }
}
