//! Algebraic operations on SLBC bytes (§5).
//!
//! Svara algebra: guṇa, vṛddhi, dīrgha, hrasva, savarṇa-dīrgha.
//! Vyañjana algebra: jaśtva, voice toggle, aspiration toggle, nasal.
//! Saṃprasāraṇa: sonorant ↔ svara.

use crate::types::*;

/// The result of an algebraic transformation.
#[derive(Debug)]
pub struct TransformResult {
    pub input_byte: u8,
    pub output_byte: u8,
    pub operation: &'static str,
    pub input_iast: String,
    pub output_iast: String,
}

// ═══════════════════════════════════════════════
//  Svara Algebra (§5.1)
// ═══════════════════════════════════════════════

/// Guṇa: set G := 01, Q := 10 (dīrgha). Accent preserved.
pub fn guna(b: u8) -> Result<TransformResult, String> {
    if !is_svara(b) {
        return Err(format!("0x{:02X} is not a svara", b));
    }
    let s = svara_s(b);
    if s == 0b00 {
        return Err("a-series has no guṇa transformation".into());
    }
    // Preserve accent (bits 5:4), set Q=10, G=01
    let accent = svara_a(b);
    let result = (0b10 << 6) | (accent << 4) | (s << 2) | 0b01;
    Ok(make_svara_result(b, result, "guṇa"))
}

/// Vṛddhi: set G := 10, Q := 10 (dīrgha). Accent preserved.
pub fn vrddhi(b: u8) -> Result<TransformResult, String> {
    if !is_svara(b) {
        return Err(format!("0x{:02X} is not a svara", b));
    }
    let s = svara_s(b);
    if s == 0b00 {
        // a → ā (vṛddhi of a is ā in the a-series)
        let accent = svara_a(b);
        let result = (0b10 << 6) | (accent << 4) | 0b10; // Q=10, A=accent, S=00(a), G=10(vṛddhi)
        return Ok(make_svara_result(b, result, "vṛddhi"));
    }
    let accent = svara_a(b);
    let result = (0b10 << 6) | (accent << 4) | (s << 2) | 0b10;
    Ok(make_svara_result(b, result, "vṛddhi"))
}

/// Dīrgha: set Q := 10. Everything else preserved.
pub fn dirgha(b: u8) -> Result<TransformResult, String> {
    if !is_svara(b) {
        return Err(format!("0x{:02X} is not a svara", b));
    }
    let result = (b & 0b00_11_11_11) | (0b10 << 6);
    Ok(make_svara_result(b, result, "dīrgha"))
}

/// Hrasva: set Q := 01. Everything else preserved.
pub fn hrasva(b: u8) -> Result<TransformResult, String> {
    if !is_svara(b) {
        return Err(format!("0x{:02X} is not a svara", b));
    }
    let result = (b & 0b00_11_11_11) | (0b01 << 6);
    Ok(make_svara_result(b, result, "hrasva"))
}

/// Savarṇa-dīrgha: if two svaras share the same series, produce dīrgha.
pub fn savarna_dirgha(a: u8, b: u8) -> Result<TransformResult, String> {
    if !is_svara(a) || !is_svara(b) {
        return Err("both inputs must be svaras".into());
    }
    if svara_s(a) != svara_s(b) {
        return Err("svaras are not savarṇa (different series)".into());
    }
    // Result: dīrgha of the series, preserving accent of first
    let accent = svara_a(a);
    let s = svara_s(a);
    let result = (0b10 << 6) | (accent << 4) | (s << 2); // Q=10(dīrgha), A=accent, S=series, G=00(śuddha)
    Ok(TransformResult {
        input_byte: a,
        output_byte: result,
        operation: "savarṇa-dīrgha",
        input_iast: format!(
            "{} + {}",
            crate::decoder::byte_to_iast(a),
            crate::decoder::byte_to_iast(b)
        ),
        output_iast: crate::decoder::byte_to_iast(result).to_string(),
    })
}

fn make_svara_result(input: u8, output: u8, op: &'static str) -> TransformResult {
    TransformResult {
        input_byte: input,
        output_byte: output,
        operation: op,
        input_iast: crate::decoder::byte_to_iast(input).to_string(),
        output_iast: crate::decoder::byte_to_iast(output).to_string(),
    }
}

// ═══════════════════════════════════════════════
//  Vyañjana Algebra (§5.2) — PLACE ∈ {0–4} only
// ═══════════════════════════════════════════════

fn require_varga(b: u8, op: &str) -> Result<(), String> {
    if !is_varga(b) {
        return Err(format!(
            "0x{:02X} is not a varga consonant — {} is defined only for PLACE ∈ {{0–4}}",
            b, op
        ));
    }
    Ok(())
}

/// Jaśtva: COL := 010 (voiced unaspirated).
pub fn jastva(b: u8) -> Result<TransformResult, String> {
    require_varga(b, "jaśtva")?;
    let result = (b & 0b11_111_000) | 0b010;
    Ok(make_vyanjana_result(b, result, "jaśtva"))
}

/// Toggle voice: COL ^= 010.
pub fn toggle_voice(b: u8) -> Result<TransformResult, String> {
    require_varga(b, "toggle voice")?;
    let result = b ^ 0b010;
    Ok(make_vyanjana_result(b, result, "toggle voice"))
}

/// Toggle aspiration: COL ^= 001.
pub fn toggle_aspiration(b: u8) -> Result<TransformResult, String> {
    require_varga(b, "toggle aspiration")?;
    let result = b ^ 0b001;
    Ok(make_vyanjana_result(b, result, "toggle aspiration"))
}

/// Make nasal: COL := 100.
pub fn make_nasal(b: u8) -> Result<TransformResult, String> {
    require_varga(b, "make nasal")?;
    let result = (b & 0b11_111_000) | 0b100;
    Ok(make_vyanjana_result(b, result, "make nasal"))
}

/// Homorganic nasal: copy PLACE from target, COL := 100.
pub fn homorganic_nasal(target: u8) -> Result<TransformResult, String> {
    require_varga(target, "homorganic nasal")?;
    let result = (target & 0b11_111_000) | 0b100;
    Ok(make_vyanjana_result(target, result, "homorganic nasal"))
}

fn make_vyanjana_result(input: u8, output: u8, op: &'static str) -> TransformResult {
    TransformResult {
        input_byte: input,
        output_byte: output,
        operation: op,
        input_iast: crate::decoder::byte_to_iast(input).to_string(),
        output_iast: crate::decoder::byte_to_iast(output).to_string(),
    }
}

// ═══════════════════════════════════════════════
//  Saṃprasāraṇa (§5.3)
// ═══════════════════════════════════════════════

/// Sonorant → svara (saṃprasāraṇa direction).
pub fn samprasarana_to_svara(b: u8) -> Result<TransformResult, String> {
    let result = match b {
        0x31 => 0x44, // ya → i
        0x32 => 0x48, // va → u
        0x33 => 0x4C, // ra → ṛ
        0x34 => 0x4F, // la → ḷ  (special case)
        _ => return Err(format!("0x{:02X} is not a sonorant (ya/va/ra/la)", b)),
    };
    Ok(TransformResult {
        input_byte: b,
        output_byte: result,
        operation: "saṃprasāraṇa (→svara)",
        input_iast: crate::decoder::byte_to_iast(b).to_string(),
        output_iast: crate::decoder::byte_to_iast(result).to_string(),
    })
}

/// Svara → sonorant (reverse saṃprasāraṇa).
pub fn samprasarana_to_sonorant(b: u8) -> Result<TransformResult, String> {
    let result = match b {
        0x44 => 0x31, // i → ya
        0x48 => 0x32, // u → va
        0x4C => 0x33, // ṛ → ra
        0x4F => 0x34, // ḷ → la  (special case)
        _ => return Err(format!("0x{:02X} is not a saṃprasāraṇa-eligible svara", b)),
    };
    Ok(TransformResult {
        input_byte: b,
        output_byte: result,
        operation: "saṃprasāraṇa (→sonorant)",
        input_iast: crate::decoder::byte_to_iast(b).to_string(),
        output_iast: crate::decoder::byte_to_iast(result).to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guna_i_to_e() {
        let r = guna(0x44).unwrap(); // i → e
        assert_eq!(r.output_byte, 0x85);
    }

    #[test]
    fn test_vrddhi_i_to_ai() {
        let r = vrddhi(0x44).unwrap(); // i → ai
        assert_eq!(r.output_byte, 0x86);
    }

    #[test]
    fn test_jastva_ka_to_ga() {
        let r = jastva(0x00).unwrap(); // ka → ga
        assert_eq!(r.output_byte, 0x02);
    }

    #[test]
    fn test_jastva_rejects_sibilant() {
        assert!(jastva(0x29).is_err()); // śa is not varga
    }

    #[test]
    fn test_samprasarana_ya_to_i() {
        let r = samprasarana_to_svara(0x31).unwrap();
        assert_eq!(r.output_byte, 0x44);
    }

    #[test]
    fn test_samprasarana_la_to_lr() {
        let r = samprasarana_to_svara(0x34).unwrap();
        assert_eq!(r.output_byte, 0x4F); // la → ḷ (special case)
    }

    #[test]
    fn test_toggle_voice() {
        let r = toggle_voice(0x00).unwrap(); // ka ↔ ga
        assert_eq!(r.output_byte, 0x02);
        let r2 = toggle_voice(0x02).unwrap(); // ga ↔ ka
        assert_eq!(r2.output_byte, 0x00);
    }

    #[test]
    fn test_accent_preserved_through_guna() {
        // i with udātta accent: Q=01, A=01, S=01, G=00 = 0x54
        let udatta_i = 0x54u8;
        let r = guna(udatta_i).unwrap();
        // Should be e with udātta: Q=10, A=01, S=01, G=01 = 0x95
        assert_eq!(r.output_byte, 0x95);
        assert_eq!(svara_a(r.output_byte), 0b01); // accent preserved
    }
}
