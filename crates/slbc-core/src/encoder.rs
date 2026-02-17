//! IAST → SLBC encoder.
//!
//! Parses IAST text into phoneme tokens, then emits SLBC byte stream
//! wrapped in PADA/SPACE/DANDA boundaries.

use crate::types::*;
use crate::numeral;

/// A token produced by the IAST tokenizer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Svara(u8),
    Vyanjana(u8),
    Space,
    Danda,
    DoubleDanda,
    Avagraha,
    Numeral(String), // string of digit chars, e.g. "108"
}

/// Tokenize an IAST string into a sequence of tokens.
pub fn tokenize_iast(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        let next = if i + 1 < len { Some(chars[i + 1]) } else { None };

        // Skip carriage returns
        if ch == '\r' {
            i += 1;
            continue;
        }

        // Whitespace → SPACE token
        if ch == ' ' || ch == '\t' || ch == '\n' {
            // Collapse consecutive whitespace
            if tokens.last() != Some(&Token::Space) {
                tokens.push(Token::Space);
            }
            i += 1;
            continue;
        }

        // Double daṇḍa: ||
        if ch == '|' && next == Some('|') {
            tokens.push(Token::DoubleDanda);
            i += 2;
            continue;
        }

        // Single daṇḍa: |
        if ch == '|' {
            tokens.push(Token::Danda);
            i += 1;
            continue;
        }

        // Avagraha
        if ch == '\'' || ch == 'ऽ' {
            tokens.push(Token::Avagraha);
            i += 1;
            continue;
        }

        // Numerals: consecutive ASCII digits
        if ch.is_ascii_digit() {
            let start = i;
            while i < len && chars[i].is_ascii_digit() {
                i += 1;
            }
            let num_str: String = chars[start..i].iter().collect();
            tokens.push(Token::Numeral(num_str));
            continue;
        }

        // ── Diphthongs (must check before single vowels) ──
        if ch == 'a' && next == Some('i') {
            tokens.push(Token::Svara(0x86)); // ai
            i += 2;
            continue;
        }
        if ch == 'a' && next == Some('u') {
            tokens.push(Token::Svara(0x8A)); // au
            i += 2;
            continue;
        }

        // ── Aspirated consonants (base + h) ──
        if let Some('h') = next {
            let aspirated = match ch {
                'k' => Some(0x01u8), // kha
                'g' => Some(0x03),   // gha
                'c' => Some(0x09),   // cha
                'j' => Some(0x0B),   // jha
                'ṭ' => Some(0x11),   // ṭha
                'ḍ' => Some(0x13),   // ḍha
                't' => Some(0x19),   // tha
                'd' => Some(0x1B),   // dha
                'p' => Some(0x21),   // pha
                'b' => Some(0x23),   // bha
                _ => None,
            };
            if let Some(byte) = aspirated {
                tokens.push(Token::Vyanjana(byte));
                i += 2;
                continue;
            }
        }

        // ── Single-character mappings ──
        match match_single(ch) {
            Some(tok) => {
                tokens.push(tok);
                i += 1;
            }
            None => {
                return Err(format!(
                    "unrecognized IAST character '{}' (U+{:04X}) at position {}",
                    ch, ch as u32, i
                ));
            }
        }
    }

    Ok(tokens)
}

/// Match a single IAST character to a token.
fn match_single(ch: char) -> Option<Token> {
    let tok = match ch {
        // ── Svaras ──
        'a' => Token::Svara(0x40),
        'ā' => Token::Svara(0x80),
        'i' => Token::Svara(0x44),
        'ī' => Token::Svara(0x84),
        'u' => Token::Svara(0x48),
        'ū' => Token::Svara(0x88),
        'ṛ' => Token::Svara(0x4C),
        'ṝ' => Token::Svara(0x8C),
        'ḷ' => Token::Svara(0x4F),
        'ḹ' => Token::Svara(0x8F),
        'e' => Token::Svara(0x85),
        'o' => Token::Svara(0x89),

        // ── Varga vyañjanas (unaspirated only — aspirated handled above) ──
        'k' => Token::Vyanjana(0x00),
        'g' => Token::Vyanjana(0x02),
        'ṅ' => Token::Vyanjana(0x04),
        'c' => Token::Vyanjana(0x08),
        'j' => Token::Vyanjana(0x0A),
        'ñ' => Token::Vyanjana(0x0C),
        'ṭ' => Token::Vyanjana(0x10),
        'ḍ' => Token::Vyanjana(0x12),
        'ṇ' => Token::Vyanjana(0x14),
        't' => Token::Vyanjana(0x18),
        'd' => Token::Vyanjana(0x1A),
        'n' => Token::Vyanjana(0x1C),
        'p' => Token::Vyanjana(0x20),
        'b' => Token::Vyanjana(0x22),
        'm' => Token::Vyanjana(0x24),

        // ── Sibilants ──
        'ś' => Token::Vyanjana(0x29),
        'ṣ' => Token::Vyanjana(0x2A),
        's' => Token::Vyanjana(0x2B),

        // ── Sonorants ──
        'y' => Token::Vyanjana(0x31),
        'v' => Token::Vyanjana(0x32),
        'r' => Token::Vyanjana(0x33),
        'l' => Token::Vyanjana(0x34),

        // ── Glottal / special ──
        'h' => Token::Vyanjana(0x38),
        'ḥ' => Token::Vyanjana(0x39), // visarga
        'ṃ' => Token::Vyanjana(0x3A), // anusvāra
        'ẖ' => Token::Vyanjana(0x3B), // jihvāmūlīya
        'ḫ' => Token::Vyanjana(0x3C), // upadhmānīya

        _ => return None,
    };
    Some(tok)
}

/// Encode a token stream into an SLBC byte stream (PHON chunk payload).
///
/// Inserts PADA_START/PADA_END around word segments.
/// Handles SPACE, DANDA, DOUBLE_DANDA, AVAGRAHA, and numeral spans.
pub fn tokens_to_bytes(tokens: &[Token]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut in_pada = false;

    for token in tokens {
        match token {
            Token::Svara(b) | Token::Vyanjana(b) => {
                if !in_pada {
                    out.push(PADA_START);
                    in_pada = true;
                }
                out.push(*b);
            }
            Token::Space => {
                if in_pada {
                    out.push(PADA_END);
                    in_pada = false;
                }
                out.push(SPACE);
            }
            Token::Danda => {
                if in_pada {
                    out.push(PADA_END);
                    in_pada = false;
                }
                out.push(DANDA);
            }
            Token::DoubleDanda => {
                if in_pada {
                    out.push(PADA_END);
                    in_pada = false;
                }
                out.push(DOUBLE_DANDA);
            }
            Token::Avagraha => {
                // Avagraha is lipi-layer, but appears inline
                if !in_pada {
                    out.push(PADA_START);
                    in_pada = true;
                }
                out.push(AVAGRAHA);
            }
            Token::Numeral(digits) => {
                if in_pada {
                    out.push(PADA_END);
                    in_pada = false;
                }
                numeral::encode_numeral(digits, &mut out);
            }
        }
    }

    // Close any open pada
    if in_pada {
        out.push(PADA_END);
    }

    out
}

/// Top-level encode: IAST string → SLBC byte stream (PHON payload).
pub fn encode_iast(input: &str) -> Result<Vec<u8>, String> {
    let tokens = tokenize_iast(input)?;
    Ok(tokens_to_bytes(&tokens))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let tokens = tokenize_iast("ka").unwrap();
        assert_eq!(tokens, vec![Token::Vyanjana(0x00), Token::Svara(0x40)]);
    }

    #[test]
    fn test_tokenize_aspirate() {
        let tokens = tokenize_iast("kha").unwrap();
        assert_eq!(tokens, vec![Token::Vyanjana(0x01), Token::Svara(0x40)]);
    }

    #[test]
    fn test_tokenize_diphthong() {
        let tokens = tokenize_iast("ai").unwrap();
        assert_eq!(tokens, vec![Token::Svara(0x86)]);
    }

    #[test]
    fn test_tokenize_au() {
        let tokens = tokenize_iast("au").unwrap();
        assert_eq!(tokens, vec![Token::Svara(0x8A)]);
    }

    #[test]
    fn test_encode_dharma() {
        let bytes = encode_iast("dharma").unwrap();
        // PADA_START dh a r m a PADA_END
        assert_eq!(
            bytes,
            vec![0x26, 0x1B, 0x40, 0x33, 0x24, 0x40, 0x2E]
        );
    }

    #[test]
    fn test_encode_two_words() {
        let bytes = encode_iast("na ca").unwrap();
        // PADA_START n a PADA_END SPACE PADA_START c a PADA_END
        assert_eq!(
            bytes,
            vec![0x26, 0x1C, 0x40, 0x2E, 0x1F, 0x26, 0x08, 0x40, 0x2E]
        );
    }

    #[test]
    fn test_ka_is_null_byte() {
        let tokens = tokenize_iast("ka").unwrap();
        assert_eq!(tokens[0], Token::Vyanjana(0x00));
    }
}
