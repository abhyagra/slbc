//! Byte introspection — explains what any SLBC byte represents.

use crate::types::*;
use crate::decoder;

/// Detailed description of an SLBC byte.
#[derive(Debug)]
pub struct ByteInfo {
    pub byte: u8,
    pub hex: String,
    pub binary: String,
    pub class: String,
    pub description: String,
    pub fields: Vec<(String, String)>,
}

/// Inspect a single SLBC byte and return its full description.
pub fn inspect_byte(b: u8) -> ByteInfo {
    let hex = format!("0x{:02X}", b);
    let binary = format!("{:08b}", b);

    if is_svara(b) {
        return inspect_svara(b, hex, binary);
    }

    if is_vyanjana(b) {
        return inspect_vyanjana(b, hex, binary);
    }

    if is_bhasha_control(b) {
        return inspect_bhasha_control(b, hex, binary);
    }

    if is_lipi_control(b) {
        return inspect_lipi_control(b, hex, binary);
    }

    // Reserved column (COLUMN = 101)
    ByteInfo {
        byte: b,
        hex,
        binary,
        class: "Reserved".into(),
        description: format!("reserved byte (PLACE={}, COLUMN=101)", place(b)),
        fields: vec![],
    }
}

fn inspect_svara(b: u8, hex: String, binary: String) -> ByteInfo {
    let q = svara_q(b);
    let a = svara_a(b);
    let s = svara_s(b);
    let g = svara_g(b);

    let q_str = match q {
        0b01 => "hrasva",
        0b10 => "dīrgha",
        0b11 => "pluta",
        _ => "?",
    };
    let a_str = match a {
        0b00 => "neutral",
        0b01 => "udātta",
        0b10 => "anudātta",
        0b11 => "svarita",
        _ => "?",
    };
    let s_str = match s {
        0b00 => "A",
        0b01 => "I",
        0b10 => "U",
        0b11 => "Ṛ",
        _ => "?",
    };
    let g_str = match g {
        0b00 => "śuddha",
        0b01 => "guṇa",
        0b10 => "vṛddhi",
        0b11 => "special",
        _ => "?",
    };

    let iast = decoder::byte_to_iast(b);

    ByteInfo {
        byte: b,
        hex,
        binary,
        class: "Svara".into(),
        description: format!("svara '{}' ({}, {}, {}-series, {})", iast, q_str, a_str, s_str, g_str),
        fields: vec![
            ("Q (quantity)".into(), format!("{:02b} = {}", q, q_str)),
            ("A (accent)".into(), format!("{:02b} = {}", a, a_str)),
            ("S (series)".into(), format!("{:02b} = {}", s, s_str)),
            ("G (grade)".into(), format!("{:02b} = {}", g, g_str)),
            ("IAST".into(), iast.to_string()),
        ],
    }
}

fn inspect_vyanjana(b: u8, hex: String, binary: String) -> ByteInfo {
    let p = place(b);
    let c = column(b);

    let place_str = match p {
        0 => "kaṇṭhya (velar)",
        1 => "tālavya (palatal)",
        2 => "mūrdhanya (retroflex)",
        3 => "dantya (dental)",
        4 => "oṣṭhya (labial)",
        5 => "ūṣman (sibilant)",
        6 => "antastha (sonorant)",
        7 => "kaṇṭhya/Vedic (glottal)",
        _ => "?",
    };

    let manner_str = if p <= 4 {
        match c {
            0 => "aghoṣa alpaprāṇa (voiceless unaspirated)",
            1 => "aghoṣa mahāprāṇa (voiceless aspirated)",
            2 => "saghoṣa alpaprāṇa (voiced unaspirated)",
            3 => "saghoṣa mahāprāṇa (voiced aspirated)",
            4 => "anunāsika (nasal)",
            _ => "?",
        }
    } else {
        "ordinal (non-varga)"
    };

    let iast = decoder::byte_to_iast(b);
    let is_varga_str = if p <= 4 { "yes" } else { "no" };

    ByteInfo {
        byte: b,
        hex,
        binary,
        class: "Vyañjana".into(),
        description: format!("vyañjana '{}' ({}, {})", iast, place_str, manner_str),
        fields: vec![
            ("PLACE".into(), format!("{:03b} = {}", p, place_str)),
            ("COLUMN".into(), format!("{:03b} = {}", c, manner_str)),
            ("Varga".into(), is_varga_str.into()),
            ("IAST".into(), iast.to_string()),
        ],
    }
}

fn inspect_bhasha_control(b: u8, hex: String, binary: String) -> ByteInfo {
    let name = match b {
        0x06 => "META_START",
        0x0E => "META_END",
        0x16 => "PHON_START",
        0x1E => "PHON_END",
        0x26 => "PADA_START",
        0x2E => "PADA_END",
        0x36 => "reserved",
        0x3E => "SAṄKHYĀ_START",
        _ => "unknown",
    };

    ByteInfo {
        byte: b,
        hex,
        binary,
        class: "Bhāṣā Control".into(),
        description: format!("{} — bhāṣā lane (COLUMN=110)", name),
        fields: vec![
            ("PLACE".into(), format!("{:03b}", place(b))),
            ("Name".into(), name.into()),
        ],
    }
}

fn inspect_lipi_control(b: u8, hex: String, binary: String) -> ByteInfo {
    let name = match b {
        0x07 => "reserved",
        0x0F => "DANDA (।)",
        0x17 => "DOUBLE_DANDA (॥)",
        0x1F => "SPACE",
        0x27 => "AVAGRAHA (ऽ)",
        0x2F => "NUM",
        0x37 => "META_EXT",
        0x3F => "reserved",
        _ => "unknown",
    };

    ByteInfo {
        byte: b,
        hex,
        binary,
        class: "Lipi Control".into(),
        description: format!("{} — lipi lane (COLUMN=111)", name),
        fields: vec![
            ("PLACE".into(), format!("{:03b}", place(b))),
            ("Name".into(), name.into()),
        ],
    }
}

/// Inspect a hex stream (e.g. "1B 40 33 24 40") and return info for each byte.
pub fn inspect_hex_stream(hex_str: &str) -> Result<Vec<ByteInfo>, String> {
    let mut bytes = Vec::new();
    for token in hex_str.split_whitespace() {
        let token = token.trim_start_matches("0x").trim_start_matches("0X");
        let b = u8::from_str_radix(token, 16)
            .map_err(|_| format!("invalid hex byte: '{}'", token))?;
        bytes.push(b);
    }
    Ok(bytes.iter().map(|&b| inspect_byte(b)).collect())
}

/// Format a ByteInfo for display.
pub fn format_byte_info(info: &ByteInfo) -> String {
    let mut out = format!(
        "  {} ({}) [{}]\n  Class: {}\n  {}",
        info.hex, info.binary, info.class, info.class, info.description
    );
    if !info.fields.is_empty() {
        out.push('\n');
        for (name, value) in &info.fields {
            out.push_str(&format!("    {}: {}\n", name, value));
        }
    }
    out
}
