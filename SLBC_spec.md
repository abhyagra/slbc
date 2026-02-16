# Sanskrit Linguistic Binary Codec (SLBC)
## Version: 0.9-draft

---


# 1. Architecture Overview

## 1.1 Encoding Pipeline

```
Any Script → [aksharamukha / sanscript] → IAST → [SLBC encoder] → .slbc
.slbc → [SLBC decoder] → IAST → [aksharamukha / sanscript] → Any Script
.slbc → [SLBC decoder] → Devanāgarī (convenience output, built-in)
```

SLBC's encoder accepts **IAST only**. IAST is the phonemically unambiguous interchange format — every sound has exactly one representation, no inherent vowel ambiguity, no conjunct guessing.

Script conversion (Devanāgarī, Grantha, etc.) is the responsibility of mature external tools **before** encoding and **after** decoding. SLBC does not reimplement script transposition.

## 1.2 Two Layers, Two Modes

```
┌─────────────────────────────────────────────────────┐
│           Vyākaraṇa (vya)                           │
│  morphology · samāsa · kāraka · sandhi · anvaya     │
│  [all sub-components travel together — indivisible] │
├─────────────────────────────────────────────────────┤
│           Bhāṣā + Lipi (bha + lipi)                 │
│  svaras · vyañjanas · controls · spaces · daṇḍas    │
│  [always present — the minimum viable stream]       │
└─────────────────────────────────────────────────────┘
```

**Mode 1 — Pāṭha** (bha + lipi): The readable text. Phonemic stream plus rendering metadata. Sufficient for transliteration, metre analysis, TTS, script conversion. All META blocks stripped.

**Mode 2 — Vyākhyā** (bha + lipi + vya): The analyzed text. Everything in pāṭha plus the complete vyākaraṇa envelope. Morphological tags, samāsa decomposition, kāraka roles, sandhi history, anvaya dependencies — all or nothing.

There is no partial vyākaraṇa mode. You either get the full grammatical analysis or none. This reflects the reality that these components are interdependent limbs of one discipline.

## 1.3 Inference Pipeline

```
Stage 1 (deterministic):  IAST → slbc encode → file.slbc [pāṭha]
Stage 2 (LLM/ML):         slbc extract --mode patha | morphology model
                           → slbc annotate --add vya → file.slbc [vyākhyā]
Stage 3 (LLM/ML):         Progressively enrich vya layer:
                           + kāraka sub-tags (0xFD)
                           + sandhi history (0xFE)
                           + anvaya chunk (ANVY)
```

Stage 1 is fully deterministic — no ML needed. Stages 2+ add vyākaraṇa via inference. The container accumulates annotations without modifying earlier data.

## 1.4 Binary Format Advisory

**`.slbc` is an opaque binary format.** All consumers — file utilities, transport layers, APIs — MUST treat `.slbc` payloads as raw binary, never as text.

Rationale: The vyañjana `ka` encodes as byte `0x00` (the null byte). This is a valid, frequent phoneme — not a sentinel or terminator. C-string functions, JSON text fields, and text-based framing protocols will silently corrupt `.slbc` data if they interpret `0x00` as end-of-string.

Implementation requirements:
- File I/O: open in binary mode (`"rb"` / `"wb"`), never text mode.
- Transport (gRPC, REST): use `bytes` / `application/octet-stream`, not string fields.
- Serialization: if embedding in JSON, Base64-encode first.
- WASM interop: pass as `&[u8]` / `Uint8Array`, not as strings.

---

# 2. Byte-Space Classification

Unchanged from v0.5.

| Class | Condition | Meaning |
|---|---|---|
| **Svara** | bits[7:6] ≠ 00 | Vowel byte |
| **Vyañjana** | bits[7:6] = 00, COLUMN ∈ {000–100} | Consonant byte |
| **Bhāṣā Control** | bits[7:6] = 00, COLUMN = 110 | Linguistic structure |
| **Lipi Control** | bits[7:6] = 00, COLUMN = 111 | Rendering metadata |
| **Reserved** | bits[7:6] = 00, COLUMN = 101 | Future expansion |

---

# 3. Vyañjana Encoding

Structure: `00 PLACE[3] COLUMN[3]`

### 3.1 PLACE Values

| PLACE | Binary | Articulation | Bytes |
|---|---|---|---|
| 0 | 000 | Kaṇṭhya (velar) | ka-varga |
| 1 | 001 | Tālavya (palatal) | ca-varga |
| 2 | 010 | Mūrdhanya (retroflex) | ṭa-varga |
| 3 | 011 | Dantya (dental) | ta-varga |
| 4 | 100 | Oṣṭhya (labial) | pa-varga |
| 5 | 101 | Ūṣman (sibilant) | śa, ṣa, sa |
| 6 | 110 | Antastha (sonorant) | ya, va, ra, la |
| 7 | 111 | Kaṇṭhya/Vedic (glottal) | ha, ḥ, ṃ, etc. |

### 3.2 COLUMN Values (Varga)

| COLUMN | Binary | Manner | Voicing | Aspiration |
|---|---|---|---|---|
| 0 | 000 | Sparśa | Aghoṣa | Alpaprāṇa |
| 1 | 001 | Sparśa | Aghoṣa | Mahāprāṇa |
| 2 | 010 | Sparśa | Saghoṣa | Alpaprāṇa |
| 3 | 011 | Sparśa | Saghoṣa | Mahāprāṇa |
| 4 | 100 | Anunāsika | — | — |
| 5 | 101 | (reserved) | — | — |
| 6 | 110 | Bhāṣā control | — | — |
| 7 | 111 | Lipi control | — | — |

**Domain of COLUMN semantics:** The manner/voicing/aspiration interpretation of COLUMN (rows 0–4 above) applies **only** to varga consonants, i.e. PLACE ∈ {0–4}. For non-varga groups (PLACE ∈ {5–7}), COLUMN values are positional indices within the group and do NOT carry manner-of-articulation semantics. See §3.5.

### 3.3 Complete Varga Table

| | COL 0 | COL 1 | COL 2 | COL 3 | COL 4 |
|---|---|---|---|---|---|
| **Velar (000)** | ka 0x00 | kha 0x01 | ga 0x02 | gha 0x03 | ṅa 0x04 |
| **Palatal (001)** | ca 0x08 | cha 0x09 | ja 0x0A | jha 0x0B | ña 0x0C |
| **Retroflex (010)** | ṭa 0x10 | ṭha 0x11 | ḍa 0x12 | ḍha 0x13 | ṇa 0x14 |
| **Dental (011)** | ta 0x18 | tha 0x19 | da 0x1A | dha 0x1B | na 0x1C |
| **Labial (100)** | pa 0x20 | pha 0x21 | ba 0x22 | bha 0x23 | ma 0x24 |

### 3.4 Non-Varga Consonants

| Byte | Binary | Phoneme | Group |
|---|---|---|---|
| 0x29 | 00 101 001 | śa | Sibilant (palatal) |
| 0x2A | 00 101 010 | ṣa | Sibilant (retroflex) |
| 0x2B | 00 101 011 | sa | Sibilant (dental) |
| 0x31 | 00 110 001 | ya | Sonorant ↔ i-series |
| 0x32 | 00 110 010 | va | Sonorant ↔ u-series |
| 0x33 | 00 110 011 | ra | Sonorant ↔ ṛ-series |
| 0x34 | 00 110 100 | la | Sonorant ↔ ḷ-special |
| 0x38 | 00 111 000 | ha | Glottal |
| 0x39 | 00 111 001 | ḥ | Visarga |
| 0x3A | 00 111 010 | ṃ | Anusvāra |
| 0x3B | 00 111 011 | ẖ | Jihvāmūlīya |
| 0x3C | 00 111 100 | ḫ | Upadhmānīya |

### 3.5 Non-Varga COLUMN Semantics (Advisory)

For PLACE ∈ {5, 6, 7}, COLUMN values are **ordinal indices** — they identify *which* consonant within the group, not manner of articulation. The voicing/aspiration bit-semantics of COLUMN 0–4 (§3.2) do **not** transfer.

**Consequence for algebraic operations:** Varga algebra (§5.2) — jaśtva, voice toggle, aspiration toggle, nasal assignment — is defined **only** for bytes where `PLACE ∈ {0–4}`. Applying these operations to sibilants, sonorants, or glottals produces undefined/meaningless results. Implementations MUST guard:

```
fn is_varga(byte: u8) -> bool {
    byte >> 6 == 0b00 && (byte >> 3) & 0b111 <= 4
}
```

---

# 4. Svara Encoding

Structure: `Q[2] A[2] S[2] G[2]`

| Field | Bits | Values |
|---|---|---|
| Q (quantity) | 7:6 | 01=hrasva, 10=dīrgha, 11=pluta |
| A (accent) | 5:4 | **00=neutral**, 01=udātta, 10=anudātta, 11=svarita |
| S (series) | 3:2 | 00=A, 01=I, 10=U, 11=Ṛ |
| G (grade) | 1:0 | 00=śuddha, 01=guṇa, 10=vṛddhi, 11=special |

**Design note:** `A=00` is the neutral (unaccented) default. This ensures that non-Vedic text — the common case — has all-zero accent bits, simplifying encoding and comparison. The canonical unaccented value is `00`; tools that strip accent information MUST write `00`, not any other value.

### 4.1 Vowel Table (neutral accent, A=00)

| Vowel | Hex | Q | S | G | Binary |
|---|---|---|---|---|---|
| a | 0x40 | hrasva | A | śuddha | 01 00 00 00 |
| ā | 0x80 | dīrgha | A | śuddha | 10 00 00 00 |
| i | 0x44 | hrasva | I | śuddha | 01 00 01 00 |
| ī | 0x84 | dīrgha | I | śuddha | 10 00 01 00 |
| u | 0x48 | hrasva | U | śuddha | 01 00 10 00 |
| ū | 0x88 | dīrgha | U | śuddha | 10 00 10 00 |
| ṛ | 0x4C | hrasva | Ṛ | śuddha | 01 00 11 00 |
| ṝ | 0x8C | dīrgha | Ṛ | śuddha | 10 00 11 00 |
| ḷ | 0x4F | hrasva | Ṛ | special | 01 00 11 11 |
| ḹ | 0x8F | dīrgha | Ṛ | special | 10 00 11 11 |
| e | 0x85 | dīrgha | I | guṇa | 10 00 01 01 |
| ai | 0x86 | dīrgha | I | vṛddhi | 10 00 01 10 |
| o | 0x89 | dīrgha | U | guṇa | 10 00 10 01 |
| au | 0x8A | dīrgha | U | vṛddhi | 10 00 10 10 |

### 4.2 Explicit Vowel Convention

SLBC always encodes vowels **explicitly**. There is no inherent vowel. Since input is IAST — where every vowel is written — the encoder emits a svara byte for every vowel phoneme, including the `a` that Devanāgarī treats as inherent.

A consonant cluster is simply consecutive vyañjana bytes with no intervening svara.

Decoders targeting scripts with an inherent vowel (Devanāgarī, etc.) MUST implement the mapping:
- vyañjana + svara `a` → bare consonant glyph (no vowel mātrā)
- vyañjana + other svara → consonant + corresponding mātrā
- vyañjana + vyañjana → insert virāma/halant between them
- vyañjana at pada-end (before PADA_END or SPACE) → consonant + virāma

There is no explicit virāma byte in SLBC. Virāma is a script-rendering artifact, not a phonemic entity. Its insertion is the decoder's responsibility.

---

# 5. Algebraic Operations

### 5.1 Svara Algebra

| Operation | Bit manipulation | Example |
|---|---|---|
| Guṇa | G := 01, Q := 10 | i(0x44) → e(0x85) |
| Vṛddhi | G := 10, Q := 10 | i(0x44) → ai(0x86) |
| Dīrgha | Q := 10 | a(0x40) → ā(0x80) |
| Hrasva | Q := 01 | ā(0x80) → a(0x40) |
| Savarṇa-dīrgha | if S₁=S₂, Q := 10 | a+a → ā, i+i → ī |

**Note:** Svara algebra operates on Q, S, and G fields only. The accent field (A) is preserved unchanged through all transformations. This is phonologically correct — sandhi alters quality and quantity, not accent.

### 5.2 Vyañjana Algebra

**Domain of validity: PLACE ∈ {0–4} (varga consonants only).** See §3.5 for rationale.

| Operation | Bit manipulation | Example |
|---|---|---|
| Jaśtva | COL := 010 | ka(0x00) → ga(0x02) |
| Toggle voice | COL ^= 010 | ka(0x00) ↔ ga(0x02) |
| Toggle aspiration | COL ^= 001 | ka(0x00) ↔ kha(0x01) |
| Make nasal | COL := 100 | ka(0x00) → ṅa(0x04) |
| Homorganic nasal | copy PLACE, COL := 100 | ca → ña |

### 5.3 Saṃprasāraṇa

| Sonorant | COLUMN[1:0] | ↔ | Svara | SERIES |
|---|---|---|---|---|
| ya (0x31) | 01 | ↔ | i (0x44) | 01 |
| va (0x32) | 10 | ↔ | u (0x48) | 10 |
| ra (0x33) | 11 | ↔ | ṛ (0x4C) | 11 |
| la (0x34) | 00→special | ↔ | ḷ (0x4F) | 11+special |

**Special case — la ↔ ḷ:** The first three sonorants (ya, va, ra) have a clean bit-correspondence: COLUMN[1:0] of the sonorant equals the SERIES field of the corresponding svara. `la` breaks this pattern — its COLUMN[1:0] is `00` (which would be A-series), but it corresponds to `ḷ` which uses SERIES=`11` with G=`11` (special).

This is a **known structural irregularity** reflecting the marginal status of ḷ in the varṇa-system. Implementations MUST handle la ↔ ḷ as an explicit special case, not via the bit-copy path used for the other three:

```
fn samprasarana_to_svara(consonant: u8) -> u8 {
    match consonant {
        0x31 => 0x44,  // ya → i
        0x32 => 0x48,  // va → u
        0x33 => 0x4C,  // ra → ṛ
        0x34 => 0x4F,  // la → ḷ  (special case)
        _ => panic!("not a sonorant"),
    }
}
```

---

# 6. Control Bytes

### 6.1 Bhāṣā Lane (COLUMN = 110)

| Byte | Hex | Name | Function |
|---|---|---|---|
| 00 000 110 | 0x06 | META_START | Begin grammar/meta block |
| 00 001 110 | 0x0E | META_END | End grammar/meta block |
| 00 010 110 | 0x16 | PHON_START | Phonological boundary start |
| 00 011 110 | 0x1E | PHON_END | Phonological boundary end |
| 00 100 110 | 0x26 | PADA_START | Word/pada boundary start |
| 00 101 110 | 0x2E | PADA_END | Word/pada boundary end |
| 00 110 110 | 0x36 | ANU | Anunāsika modifier |
| 00 111 110 | 0x3E | SAṄKHYĀ_START | Opens numeral digit-word span (see §6.3) |

**Note:** All bhāṣā control slots are now allocated. Future bhāṣā-layer control needs must use META extension mechanisms or the reserved column (COLUMN=101).

### 6.2 Lipi Lane (COLUMN = 111)

| Byte | Hex | Name | Function |
|---|---|---|---|
| 00 000 111 | 0x07 | — | Reserved |
| 00 001 111 | 0x0F | DANDA | Single daṇḍa (।) |
| 00 010 111 | 0x17 | DOUBLE_DANDA | Double daṇḍa (॥) |
| 00 011 111 | 0x1F | SPACE | Visual word separator |
| 00 100 111 | 0x27 | AVAGRAHA | Avagraha (ऽ) |
| 00 101 111 | 0x2F | NUM | Numeral digit-glyph span (see §6.3) |
| 00 110 111 | 0x37 | META_EXT | Lipi extensions |
| 00 111 111 | 0x3F | — | Reserved |

### 6.3 Numeral Encoding

#### 6.3.1 Design Principle

Numeric glyphs (१, २, ३, etc.) are a **lipi-layer convention** — visual shorthand that diverges from the spoken form for readability. They are not phonemic entities.

In the bhāṣā layer, every entity must have articulatory form. The bhāṣā layer is primary; lipi is its projection. A numeral that exists only in lipi would be **irrecoverably lost** during bhāṣā-only processing — unlike virāma or daṇḍa, which are reconstructible from pada boundaries.

Therefore, numerals are encoded in **both layers**:

| Layer | Representation | Purpose |
|---|---|---|
| **Bhāṣā** | Digit-words in prātipadika form, R→L order | Phonemic preservation, grammatical processability |
| **Lipi** | Digit glyphs in visual order | Rendering fidelity |

The bhāṣā encoding follows the classical mathematical recitation convention:

> **aṅkānāṃ vāmato gatiḥ** — "digits proceed leftward"

Digits are encoded starting from the units place (rightmost) moving left, each as a **pure prātipadika** (stem form) — no vibhakti, no sandhi.

#### 6.3.2 Bhāṣā Layer: SAṄKHYĀ Span

**Span structure:**

```
[SAṄKHYĀ_START (0x3E)]
[digit-count (ULEB128)]            ← number of digit-padas that follow
[PADA_START] digit-word₁ [PADA_END]
[PADA_START] digit-word₂ [PADA_END]
...
[PADA_START] digit-wordₙ [PADA_END]
```

**Termination:** The SAṄKHYĀ span contains exactly `digit-count` digit-padas. No SAṄKHYĀ_END byte is needed. After consuming the declared number of digit-padas, the parser resumes normal mode.

**Rationale for explicit count over vocabulary-exhaustion termination:** Four digit-words — pañca (5), sapta (7), aṣṭa (8), nava (9) — are an-stem numerals whose prathamā/dvitīyā napuṃsakaliṅga forms are identical to their prātipadika. These words can appear as regular grammatical padas immediately after a SAṄKHYĀ span. Without an explicit count, the decoder would incorrectly consume them as digits. Additionally, `nava` ("nine") is homophonous with `nava` ("new"), creating a lexical ambiguity that further necessitates explicit span boundaries.

ULEB128 is chosen for the count to maintain consistency with all other variable-length integers in the format (§7.4, §8.4, §9.1).

**Digit-word vocabulary (closed set):**

| Digit | Prātipadika | SLBC bhāṣā bytes |
|---|---|---|
| 0 | śūnya | 0x29 0x88 0x1C 0x31 0x40 |
| 1 | eka | 0x85 0x00 0x40 |
| 2 | dvi | 0x1A 0x32 0x44 |
| 3 | tri | 0x18 0x33 0x44 |
| 4 | catur | 0x08 0x40 0x18 0x48 0x33 |
| 5 | pañca | 0x20 0x40 0x0C 0x08 0x40 |
| 6 | ṣaṣ | 0x2A 0x40 0x2A |
| 7 | sapta | 0x2B 0x40 0x20 0x18 0x40 |
| 8 | aṣṭa | 0x40 0x2A 0x10 0x40 |
| 9 | nava | 0x1C 0x40 0x32 0x40 |

These are the **only** valid pada contents within a SAṄKHYĀ span. The set is fixed and closed — implementations MUST validate pada contents against this table.

**Ordering: R→L (units-first).**

Digits are encoded starting from the units place (rightmost digit) and proceeding leftward:

```
Number:  108
Digits:  1   0   8
Order:   3rd 2nd 1st  (R→L: units first)

Bhāṣā:  [SAṄKHYĀ_START] [0x03]
         [PADA_START] aṣṭa  [PADA_END]    ← 8 (units)
         [PADA_START] śūnya  [PADA_END]    ← 0 (tens)
         [PADA_START] eka    [PADA_END]    ← 1 (hundreds)
```

The decoder reverses the sequence to reconstruct the positional value.

#### 6.3.3 Lipi Layer: NUM Digit-Glyph Span

NUM (0x2F) opens a **digit-glyph span** in the lipi lane. Within the span, bytes 0x00–0x0F are interpreted as glyph tokens:

| Byte | Glyph | Function |
|---|---|---|
| 0x00–0x09 | Digits 0–9 | Digit glyphs |
| 0x0A | — | Digit-group separator (comma, etc.) |
| 0x0B | — | Fractional mark (apūrṇāṅka / decimal point) |
| 0x0C | — | Positive sign |
| 0x0D | — | Negative sign |
| 0x0E–0x0F | — | Reserved |

**Termination:** The first byte outside 0x00–0x0F exits the digit-glyph span. The parser resumes normal lipi-mode parsing.

**Digit order: L→R (visual).** Lipi digit-glyphs follow **visual order** (left-to-right), matching manuscript fidelity:

```
Number 108:
  NUM (0x2F) 0x01 0x00 0x08
              1    0    8     ← visual L→R order
```

Leading zeros are preserved (glyph-faithful encoding). `0x2F 0x00` is valid (zero).

#### 6.3.4 Interleaved PHON Chunk: Both Layers Together

In a PHON chunk (interleaved bhāṣā + lipi), both representations appear for the same numeral:

```
Full stream for "adhyāyaḥ 108 dharma":

[PADA_START]
  a(0x40) dh(0x1B) y(0x31) ā(0x80) y(0x31) a(0x40) ḥ(0x39)
[PADA_END]

[SPACE (0x1F)]

[SAṄKHYĀ_START (0x3E)]
[0x03]                                            ← 3 digit-padas

  [PADA_START]
    a(0x40) ṣ(0x2A) ṭ(0x10) a(0x40)              ← "aṣṭa" = 8
  [PADA_END]

  [PADA_START]
    ś(0x29) ū(0x88) n(0x1C) y(0x31) a(0x40)      ← "śūnya" = 0
  [PADA_END]

  [PADA_START]
    e(0x85) k(0x00) a(0x40)                        ← "eka" = 1
  [PADA_END]

[NUM (0x2F)] [0x01] [0x00] [0x08]                 ← lipi digit glyphs

[SPACE (0x1F)]

[PADA_START]
  dh(0x1B) a(0x40) r(0x33) m(0x24) a(0x40)       ← "dharma"
[PADA_END]
```

#### 6.3.5 Layer Extraction Behavior

| Extraction mode | SAṄKHYĀ span (bhāṣā) | NUM span (lipi) |
|---|---|---|
| **Pāṭha** (bha + lipi) | Emitted | Emitted |
| **Bhāṣā-only** | Emitted (phonemic content preserved) | Stripped |
| **Vyākhyā** | Emitted; META may annotate with prātipadika class=14 | Emitted |

Bhāṣā-only extraction retains full numeral context. The digit-words are phonemically real, grammatically inert (prātipadika form), and unambiguously bounded (SAṄKHYĀ_START + count).

#### 6.3.6 Decoder Reference

```rust
/// Decode a SAṄKHYĀ span from the bhāṣā stream.
/// Returns the reconstructed integer value as a digit vector.
fn decode_sankhya(stream: &mut ByteStream) -> Vec<u8> {
    assert_eq!(stream.read_byte(), 0x3E); // SAṄKHYĀ_START

    let digit_count = stream.read_uleb128();
    let mut digits = Vec::with_capacity(digit_count);

    for _ in 0..digit_count {
        assert_eq!(stream.read_byte(), 0x26); // PADA_START
        let pada_bytes = stream.read_until(0x2E); // read to PADA_END
        let digit = match digit_vocabulary_lookup(&pada_bytes) {
            Some(d) => d,
            None => panic!("invalid digit-word inside SAṄKHYĀ span"),
        };
        digits.push(digit);
    }

    digits.reverse(); // vāmato gatiḥ: R→L input → L→R value
    digits
}

/// Encode an integer as a SAṄKHYĀ span in the bhāṣā stream.
fn encode_sankhya(value: &str, stream: &mut ByteStream) {
    let digit_chars: Vec<char> = value.chars().collect();

    stream.write_byte(0x3E); // SAṄKHYĀ_START
    stream.write_uleb128(digit_chars.len());

    // Emit digits R→L (units first)
    for ch in digit_chars.iter().rev() {
        let digit = ch.to_digit(10).expect("non-digit character");
        stream.write_byte(0x26); // PADA_START
        stream.write_bytes(digit_to_pratipadika(digit));
        stream.write_byte(0x2E); // PADA_END
    }
}
```

#### 6.3.7 Open Considerations

**Fractional / decimal numbers:** The lipi layer supports fractional marks (0x0B) and signs (0x0C, 0x0D) within a NUM span. The bhāṣā-layer representation of fractions and negative numbers is **not yet specified**. This may require extending the SAṄKHYĀ span grammar or introducing additional marker bytes. Deferred to future revision.

**Ordinal and compound numerals:** When a number appears as part of a literary samāsa (e.g., "aṣṭottaraśatam" written by the author in IAST), it enters the bhāṣā layer as normal phonemes with no SAṄKHYĀ markers. The SAṄKHYĀ mechanism applies **only** when the encoder translates digit glyphs from the input.

---

# 7. Container Format (.slbc)

## 7.1 Header

```
Bytes 0–3:    "SLBC" (magic, ASCII)
Bytes 4–7:    Version: 0.0.0.8 (one byte per component)
Bytes 8–11:   Flags (see §7.2)
Bytes 12–13:  Extended header length (uint16 LE)
```

**Byte order:** All multi-byte integer fields in the container format are **little-endian** unless explicitly stated otherwise.

**Extended header length (bytes 12–13):** This is the length in bytes of any additional header data **beyond** the fixed 14-byte header. A value of `0x0000` means no extended header — the first chunk begins immediately at byte offset 14. If non-zero, the extended header occupies bytes 14 through (14 + length − 1), and the first chunk begins at byte offset (14 + length).

The extended header is reserved for future use (e.g., registry version pinning, source metadata). v0.8 encoders MUST write `0x0000`. Decoders MUST skip the extended header by its declared length even if they do not understand its contents.

## 7.2 Flags Bytes (v0.8)

### **Bytes 8 to 10 are reserved and MUST be 0x00.**

### Byte 11:
```
  7     6     5     4     3     2     1     0
┌─────┬─────┬─────┬─────┬─────┬─────┬─────┬─────┐
│ HAS │ HAS │INTER│ VED │ VYA │rsvd │rsvd │rsvd │
│LIPI │META │LEAV │ IC  │     │     │     │     │
└─────┴─────┴─────┴─────┴─────┴─────┴─────┴─────┘
```

| Bit | Name | Meaning |
|---|---|---|
| 7 | HAS_LIPI | Lipi-layer bytes present |
| 6 | HAS_META | META blocks present (PADA/PHON markers) |
| 5 | INTERLEAVED | Bhāṣā and lipi interleaved |
| 4 | VEDIC | Vedic accents meaningful |
| 3 | VYA | Vyākaraṇa layer present (morphology + kāraka + sandhi + anvaya) |
| 2:0 | — | Reserved (must be 0) |

**Mode derivation from flags:**
- `HAS_LIPI=1, VYA=0` → file contains pāṭha only
- `HAS_LIPI=1, VYA=1` → file contains vyākhyā (full analysis)
- `HAS_LIPI=0` → bhāṣā-canonical form (internal use only; not a consumer-facing mode)

## 7.3 Chunk Types

| Type | Hex | Name | Description |
|---|---|---|---|
| 0x01 | PHON | Phonemic | Interleaved bha+lipi stream |
| 0x02 | BHA | Bhāṣā | Pure bhāṣā bytes |
| 0x03 | LIPI | Lipi | Pure lipi bytes |
| 0x04 | META | Grammar | Standalone vyākaraṇa block |
| 0x05 | DICT | Dictionary | Registry references |
| 0x06 | IDX | Index | Pada offset index |
| 0x07 | ANVY | Anvaya | Dependency structure |
| 0x10 | EXT | Extension | Application-specific |
| 0xFF | EOF | End of File | Terminal (payload length = 0) |

**Namespace note:** Chunk type values (0x01–0xFF) occupy the same numerical range as some bhāṣā/lipi control bytes (e.g., 0x06 = both IDX chunk type and META_START control byte, 0x07 = both ANVY chunk type and a reserved lipi byte). These are **not** ambiguous — chunk type bytes appear only in chunk headers (§7.4), while control bytes appear only within chunk payloads. Parsers always know which namespace they are in based on parsing state.

## 7.4 Chunk Framing (Bhāṣā + Lipi)

Each chunk in the container has the following structure:

```
┌──────────┬────────────────────┬───────────────────┐
│ Type     │ Payload Length     │ Payload           │
│ (1 byte) │ (ULEB128-32)       │ (variable)        │
└──────────┴────────────────────┴───────────────────┘
```

| Field | Size | Description |
|---|---|---|
| Type | 1 byte | Chunk type from §7.3 |
| Payload length | 1–5 bytes (ULEB128-32) | Length of payload in bytes. 0 is valid (e.g., EOF chunk). See encoding below. |
| Payload | variable | Chunk-specific data |

**ULEB128-32 encoding:** Standard ULEB128 — each byte carries 7 data bits (bits[6:0]) and a continuation flag (bit[7]; 1 = more bytes follow, 0 = final byte). The decoded value MUST fit in 32 bits (max `0xFFFFFFFF`). A decoder MUST reject any encoding that requires more than 5 bytes or produces a value exceeding 2³²−1.

| Payload size range | Length field bytes |
|---|---|
| 0 – 127 | 1 |
| 128 – 16,383 | 2 |
| 16,384 – 2,097,151 | 3 |
| 2,097,152 – 268,435,455 | 4 |
| 268,435,456 – 4,294,967,295 | 5 |

This aligns with the ULEB128 encoding already used in the vyākaraṇa layer (§8.4) and registries (§9.1), giving the format a single variable-length integer encoding throughout.

**EOF chunk:** Type `0xFF`, payload length `0x00` (single byte, ULEB128 for 0), no payload bytes. Marks end of file. A valid `.slbc` file MUST end with an EOF chunk.

**Chunk sequence in pāṭha mode (v0.8 minimum viable):**

```
[Header 14 bytes]
[PHON chunk: type=0x01, length, interleaved bha+lipi bytes]
  ...additional PHON chunks if needed...
[EOF chunk: type=0xFF, length=0]
```

**Vyākaraṇa chunk framing** (META, DICT, IDX, ANVY) is structurally identical (same type + length + payload header), but the internal payload layout of these chunks is **deferred to a future revision**. v0.8 fully defines the framing for PHON, BHA, and LIPI chunks, and the DICT chunk payload format (§9.6). The vyākaraṇa payload schemas are specified at the envelope level (§8) but detailed wire formats for sub-fields (kāraka byte layout, sandhi history byte layout) remain under design.

## 7.5 Extraction Logic

```
slbc extract --mode patha:
  For each byte:
    if is_svara(b) or is_vyanjana(b) → emit
    if is_bhasha_control(b):
      if b == META_START → skip until matching META_END
      if b == SANKHYA_START → emit (enter SAṄKHYĀ span; emit count + digit-padas)
      else → emit (PADA_START, PADA_END, PHON markers)
    if is_lipi_control(b):
      if b == NUM → emit (enter digit-glyph span; emit glyph bytes)
      else → emit
  For chunks:
    PHON, BHA, LIPI → include
    META, DICT, ANVY → exclude

slbc extract --mode bhasha-only:
  For each byte:
    if is_svara(b) or is_vyanjana(b) → emit
    if is_bhasha_control(b):
      if b == META_START → skip until matching META_END
      if b == SANKHYA_START → emit (SAṄKHYĀ span carries phonemic content)
      else → emit (PADA_START, PADA_END, PHON markers)
    if is_lipi_control(b) → strip (including NUM spans)
  For chunks:
    BHA → include
    LIPI → exclude
    PHON → emit bhāṣā bytes only

slbc extract --mode vyakhya:
  Emit everything.
```

---

# 8. Vyākaraṇa Layer (Complete)

The vyākaraṇa layer is a **single indivisible** annotation envelope. It contains all grammatical analysis: morphological tags, samāsa decomposition, kāraka roles, sandhi history, and anvaya dependencies. These are interdependent limbs of one discipline — they travel together.

## 8.1 META Envelope Structure

```
PADA_START (0x26)
├── META_START (0x06)
│   ├── Grammar Tag (1+ bytes)
│   │   [subanta / tiṅanta / avyaya / samāsa header]
│   │   [morphological fields + registry ID]
│   ├── [0xFD] Kāraka sub-tag (optional)
│   │   [role, governor ref, sub-type]
│   ├── [0xFE] Sandhi history sub-tag (optional)
│   │   [type, pre-sandhi form, junction, rule ref]
│   └── META_END (0x0E)
├── Phonemic bytes (bhāṣā stream)
└── PADA_END (0x2E)
```

### 8.2 Tag Header Byte

```
  7   │  6  │ 5  4 │ 3  2  1  0
LAYER │ VER │ TYPE │  SUBTYPE
```

| TYPE | Binary | Category |
|---|---|---|
| 00 | Subanta | Nominal (vibhakti, vacana, liṅga, stem class) |
| 01 | Tiṅanta | Verbal (lakāra, puruṣa, vacana, pada, prayoga, gaṇa) |
| 10 | Avyaya | Indeclinable (class + registry ID) |
| 11 | Samāsa | Compound (type, member count, nested children) |

(See vyākaraṇa detail spec for full field layouts.)

### 8.3 Internal Sub-Tags

| Marker | Hex | Sub-component | Contains |
|---|---|---|---|
| 0xFD | Kāraka | Role (kartā/karma/karaṇa/etc.), governor reference, sub-type |
| 0xFE | Sandhi history | Sandhi type, pre-sandhi SLBC bytes, junction type, rule reference |

### 8.4 Anvaya Chunk (ANVY 0x07)

Sentence-level dependency trees. Stored as a separate chunk (not inline) because it's a whole-sentence property:

```
ANVY chunk:
├── Sentence count (ULEB128)
└── Per sentence:
    ├── Pada count (ULEB128)
    ├── Root verb pada index (ULEB128)
    └── Dependency edges (dependent→governor + relation type)
```

**Parser statefulness note:** ULEB128 values within the ANVY chunk payload (and within META envelopes) are positionally determined. The parser knows at each byte offset whether it is reading a ULEB128 count, an index, a tag header, or a control byte — because the schema dictates the sequence. There is no byte-level ambiguity even though a ULEB128-encoded value could numerically equal a control byte (e.g., a sentence count of 6 would encode as `0x06`, the same value as META_START). The parser is **stateful**, not a flat byte-scanner. It never interprets a ULEB128 field as a control byte or vice versa.

### 8.5 All-or-Nothing Principle

A consumer either reads the vyākaraṇa layer completely or ignores it entirely. There is no mode that extracts morphology without kāraka, or kāraka without anvaya. The reasons:

1. **Kāraka without morphology is meaningless** — you can't assign "kartā" without knowing vibhakti
2. **Anvaya without kāraka is unanchored** — dependency edges need semantic roles
3. **Sandhi history without morphology is just raw bytes** — pre-sandhi forms need stem identification
4. **Pāṇini didn't write four separate treatises** — these are one system

---

# 9. Registries

Registries bridge the gap between **phonemes** (what SLBC encodes at Stage 1) and **meaning** (what the vyākaraṇa layer annotates at Stage 2+). When a META block tags a pada as a tiṅanta, it includes a dhātu ID. When it tags a subanta, it includes a prātipadika ID. When sandhi history records which rule applied, it includes a sandhi rule ID. These IDs must resolve to actual entries — that resolution is the registry's job.

```
META tag inside PHON chunk
  └── ULEB128 ID (e.g. 0x0A = 10)
        └── DICT chunk in container
              └── resolves via registry
                    └── ID 10 → kṛ (gaṇa 8, ubhayapada, sakarmaka)
```

## 9.1 Three Registries

| Registry | File ext. | Magic | Covers | Bootstrap size |
|---|---|---|---|---|
| **Dhātu** | .sldr | `SPDR` | Verbal roots from Dhātupāṭha | 292 entries, all 10 gaṇas |
| **Prātipadika** | .slpr | `SPPR` | Nominal stems (nouns, adjectives, pronouns, numerals) | 185 entries, 15 stem classes |
| **Sandhi Rule** | .slsr | `SPSR` | Aṣṭādhyāyī sūtra references | 36 core rules |

IDs use ULEB128 encoding throughout. Registries are **append-only** — IDs never reassigned. Referenced by version in container DICT chunk.

## 9.2 Standard vs. Extension Registries

The codec **internalizes** the standard registries. These ship as built-in data compiled into the `slbc` binary:

```
slbc (binary)
├── [built-in] slbc-dhatu-v1      292 dhātus, all 10 gaṇas
├── [built-in] slbc-prati-v1      185 prātipadikas, 15 stem classes
└── [built-in] slbc-sandhi-v1      36 sandhi rules
```

The standard registries cover the core classical vocabulary: Pāṇinīya Dhātupāṭha bootstrap, Gītā/epic proper names, and the most frequent nominal stems and sandhi rules.

**Extension registries** allow users to supply additional entries beyond the standard set — corpus-specific dhātus, domain-specific prātipadikas, or specialized sandhi rules. Extensions are loaded at runtime via CLI switches and are **additive**: they augment the built-in data without replacing it. If an extension ID collides with a built-in ID, the extension is rejected with an error.

```bash
# Use only built-in registries (default)
slbc annotate --add vya -i base.slbc --from analysis.json -o out.slbc

# Extend with additional dhātu entries
slbc annotate --add vya -i base.slbc --from analysis.json \
    --sldr custom-dhatus.sldr -o out.slbc

# Extend with additional prātipadika entries
slbc annotate --add vya -i base.slbc --from analysis.json \
    --slpr vedic-stems.slpr -o out.slbc

# Extend with additional sandhi rules
slbc annotate --add vya -i base.slbc --from analysis.json \
    --slsr special-rules.slsr -o out.slbc

# Combine multiple extensions
slbc annotate --add vya -i base.slbc --from analysis.json \
    --sldr custom-dhatus.sldr \
    --slpr vedic-stems.slpr \
    --slsr special-rules.slsr \
    -o out.slbc
```

When writing the output container, the encoder records extension provenance in the DICT chunk (§9.6), so downstream decoders know which registries are needed.

## 9.3 Three Formats

### 9.3.1 Human-Readable TSV (authoring & version control)

Tab-separated, one entry per line, `#` comments. This is the source of truth for registry data — what you version-control, review, and extend. Binary formats are derived from TSV.

**Dhātu TSV:**
```
# ID	MŪLA	GAṆA	PADA	KARMA	SET	IT	ARTHA	DP_INDEX
1	bhū	1	P	ak	s		to be, become	01.0001
10	kṛ	1	U	sak	a	ṇ	to do, make	01.0010
300	ad	2	P	sak	a		to eat	02.0001
```

**Prātipadika TSV:**
```
# ID	STEM	CLASS	LIṄGA	FLAGS	ARTHA
5	dharma	0	puṃ	-	dharma, duty
44	kṣetra	1	napuṃ	-	field
128	sarva	13	tri	S	all, every
```

**Sandhi Rule TSV:**
```
# ID	TYPE	SUTRA	DESCRIPTION	EXAMPLE
1	svara	6.1.77	iko yaṇ aci	iti api → ityapi
71	hal	8.4.53	jhalāṃ jaśo'nte	vāk + dānam → vāgdānam
```

### 9.3.2 Binary (.sldr / .slpr / .slsr) — compact runtime format

12-byte header + packed entries. Each entry: ULEB128 ID + length-prefixed IAST string (ULEB128 length + UTF-8 bytes) + metadata bytes.

**Binary file header (12 bytes):**
```
Bytes 0–3:   Magic ("SPDR" / "SPPR" / "SPSR")
Bytes 4–5:   Version (uint16 LE)
Bytes 6–9:   Entry count (uint32 LE)
Bytes 10–11: Reserved (0x0000)
```

**Dhātu metadata (3 bytes per entry):**
```
Byte 0: GAṆA(4 bits) | PADA(2 bits) | KARMA(2 bits)
Byte 1: IT_MARKERS (anubandha flags: ṇit, ñit, ṅit, etc.)
Byte 2: SET flags (seṭ / aniṭ / veṭ)
```

**Prātipadika metadata (2 bytes per entry):**
```
Byte 0: STEM_CLASS(4 bits) | LIṄGA(3 bits) | reserved(1 bit)
Byte 1: FLAGS (sarvanāma, saṃkhyā, avyaya)
```

**Sandhi Rule metadata (variable per entry):**
```
Byte 0: TYPE(4 bits) | reserved(4 bits)
Bytes 1+: Sūtra reference as length-prefixed UTF-8 string
```

### 9.3.3 TSV → Binary compilation

```bash
# Compile TSV source to binary registry
slbc registry compile --type dhatu -i dhatus.tsv -o custom-dhatus.sldr
slbc registry compile --type prati -i stems.tsv -o vedic-stems.slpr
slbc registry compile --type sandhi -i rules.tsv -o special-rules.slsr

# Inspect a binary registry
slbc registry inspect -i custom-dhatus.sldr
slbc registry lookup -i custom-dhatus.sldr --id 10    # → kṛ
slbc registry lookup -i custom-dhatus.sldr --mula "kṛ" # → ID=10
```

## 9.4 Dhātu ID Allocation by Gaṇa

IDs are pre-allocated by gaṇa. The 127 most frequent Bhvādi roots fit in a single ULEB128 byte. The entire classical Dhātupāṭha fits in ≤ 2 bytes.

| Range | ULEB128 bytes | Gaṇa | Bootstrap count | Vikaraṇa |
|---|---|---|---|---|
| 1–127 | 1 | 1. Bhvādi | 127 | śap (a) |
| 128–299 | 2 | 1. Bhvādi (cont.) | — | śap (a) |
| 300–399 | 2 | 2. Adādi | 30 | ∅ (root class) |
| 400–449 | 2 | 3. Juhotyādi | 11 | reduplication |
| 450–599 | 2 | 4. Divādi | 31 | ya |
| 600–649 | 2 | 5. Svādi | 11 | nu/nv |
| 650–849 | 2 | 6. Tudādi | 26 | a (accented) |
| 850–899 | 2 | 7. Rudhādi | 14 | na/n (infix) |
| 900–929 | 2 | 8. Tanādi | 5 | u |
| 930–999 | 2 | 9. Kryādi | 16 | nā/nī |
| 1000–1499 | 2 | 10. Curādi | 21 | ṇic (aya) |
| 1500–1999 | 2 | Vedic/Kaṇḍvādi | — | — |
| 2000–16383 | 2 | Extended/variants | — | — |
| 16384+ | 3 | Future expansion | — | — |

**Total bootstrapped: 292 dhātus across all 10 gaṇas.**

Extension registries (loaded via `--sldr`) MUST allocate IDs in the **Extended/variants** range (2000+) or the **Future expansion** range (16384+). IDs below 2000 are reserved for the standard registry.

## 9.5 Prātipadika Stem Classes

| CLASS | Code | Paradigm | Examples in registry |
|---|---|---|---|
| 0 | a-kārānta puṃ | rāma-like | deva, dharma, kṛṣṇa, arjuna |
| 1 | a-kārānta napuṃ | vana-like | vana, phala, kṣetra, sukha |
| 2 | ā-kārānta strī | ramā-like | vidyā, gītā, māyā, śraddhā |
| 3 | i-kārānta | agni/mati-like | agni, muni, bhūmi, buddhi |
| 4 | ī-kārānta | nadī-like | nadī, devī, strī, lakṣmī |
| 5 | u-kārānta | guru-like | guru, madhu, śatru, kuru |
| 6 | ū-kārānta | bhū-like | (reserved) |
| 7 | ṛ-kārānta | pitṛ-like | pitṛ, mātṛ, kartṛ, bhrātṛ |
| 8 | consonant | generic | (covered by subtypes below) |
| 9 | s-stem | manas-like | manas, tejas, tapas, havis |
| 10 | an-stem | rājan-like | rājan, ātman, brahman |
| 11 | in-stem | yogin-like | yogin, dhanin, balin |
| 12 | at/mat/vat-stem | bhagavat-like | mahat, bhagavat, dhīmat |
| 13 | pronoun | sarvanāma | sarva, idam, tat, kim, yad |
| 14 | numeral | saṃkhyā | eka, dvi, tri, catur, śata |

**Total bootstrapped: 185 prātipadikas** including Gītā/epic proper names.

## 9.6 DICT Chunk Format (Container Embedding)

The DICT chunk (type `0x05`) connects META tag IDs to registry data. A container MAY have **multiple DICT chunks** — typically one per registry type.

### DICT chunk payload structure:

```
┌───────────────┬──────────┬────────────────────────────────────┐
│ Registry Type │ Mode     │ Mode-specific payload              │
│ (1 byte)      │ (1 byte) │ (variable)                         │
└───────────────┴──────────┴────────────────────────────────────┘
```

**Registry type byte:**

| Value | Registry |
|---|---|
| 0x01 | Dhātu |
| 0x02 | Prātipadika |
| 0x03 | Sandhi Rule |

**Mode byte:**

| Mode | Byte | Behavior | Use case |
|---|---|---|---|
| **Embedded** | 0x00 | Full registry data inline in payload | Archival, offline, self-contained files |
| **External** | 0x01 | Filename reference only | Working corpora with shared registry |
| **Hybrid** | 0x02 | External base + inline overrides | Production (recommended) |

### Mode-specific payloads:

**Embedded (0x00):**
```
Entry count (ULEB128)
Per entry:
  ID (ULEB128)
  IAST string (ULEB128 length + UTF-8 bytes)
  Metadata bytes (registry-type dependent; see §9.3.2)
```

**External (0x01):**
```
Version (uint16 LE)
Filename (ULEB128 length + UTF-8 bytes)
```

**Hybrid (0x02):**
```
[External block — as above]
Override count (ULEB128)
Per override:
  [Entry — as in Embedded]
```

### Resolution order:

1. Built-in standard registry (always available)
2. External registry file (if referenced)
3. Embedded/hybrid overrides (highest priority)

A decoder MUST be able to resolve all IDs referenced by META tags. If a DICT chunk references an external file that is unavailable, the decoder MUST report an error rather than silently dropping annotations.

## 9.7 Design Principles

**Append-only IDs.** Once assigned, an ID is never reused. Registry version increments; old files remain decodable.

**ULEB128 everywhere.** Variable-length encoding means no fixed-width waste. Common roots (bhū, kṛ, gam) cost 1 byte. The entire Dhātupāṭha fits in ≤ 2 bytes. The registry can grow to millions without format changes.

**TSV is source of truth.** Binary formats are derived. Human-readable TSV files are what you version-control, review, and extend. Binary compilation is a build step.

**Built-in for standard, extension for the rest.** The standard registries cover the core classical vocabulary and ship with the codec. Corpus-specific additions (Vedic roots, technical śāstra terms, regional stem variants) are loaded as extensions at runtime — never by modifying the standard data.

**Hybrid mode for production containers.** Reference the canonical registry for standard entries; embed only corpus-specific additions. A Gītā file references the standard dhātu/prātipadika registries and only embeds custom entries like "yuyutsu" (desiderative adjective not in the standard stem list).

**Registry is not the grammar.** The registry maps IDs to lemmas. The grammar (vibhakti, lakāra, kāraka, sandhi) lives in the META tag bytes. The registry just tells you *which* root or stem is being inflected.

## 9.8 Data Files

```
slbc/data/
├── slbc-dhatu-v1.dhatu.tsv      292 entries, all 10 gaṇas
├── slbc-prati-v1.prati.tsv      185 entries, 15 stem classes
└── slbc-sandhi-v1.sandhi.tsv     36 rules (svara/visarga/hal/compound)
```

Bootstrap coverage is sufficient for Gītā, Upaniṣad, and epic-level texts. Extending to full Dhātupāṭha coverage (~2000 roots) and Amarakośa-level prātipadika coverage (~5000 stems) is a data entry task, not a format change. The architecture supports it without modification — append to the TSV files, recompile, bump the version.

---

# 10. CLI Interface

```bash
# ── Encode (IAST only) ──
slbc encode "dharmakṣetre kurukṣetre" --hex
slbc encode -i input_iast.txt -o output.slbc

# ── Decode ──
slbc decode -i input.slbc --to iast
slbc decode -i input.slbc --to devanagari

# ── Extract modes ──
slbc extract --mode patha -i annotated.slbc -o readable.slbc
slbc extract --mode vyakhya -i annotated.slbc  # identity (emit all)

# ── Inspect ──
slbc inspect --byte 0x1B
slbc inspect --from-hex "1B 40 33 24 40"

# ── Transform ──
slbc transform --guna 0x44      # i → e
slbc transform --jastva 0x00    # ka → ga

# ── Round-trip test ──
slbc roundtrip "kṛṣṇa"

# ── Annotate (uses built-in registries by default) ──
slbc annotate --add vya -i base.slbc --from analysis.json -o annotated.slbc

# ── Annotate with extension registries ──
slbc annotate --add vya -i base.slbc --from analysis.json \
    --sldr custom-dhatus.sldr \
    --slpr vedic-stems.slpr \
    --slsr special-rules.slsr \
    -o annotated.slbc

# ── Registry management ──
# Compile TSV → binary
slbc registry compile --type dhatu -i dhatus.tsv -o custom-dhatus.sldr
slbc registry compile --type prati -i stems.tsv -o vedic-stems.slpr
slbc registry compile --type sandhi -i rules.tsv -o special-rules.slsr

# Inspect binary registry
slbc registry inspect -i custom-dhatus.sldr

# Lookup by ID or mūla
slbc registry lookup -i custom-dhatus.sldr --id 10
slbc registry lookup -i custom-dhatus.sldr --mula "kṛ"

# List built-in registry stats
slbc registry stats
```

### 10.1 Registry Extension Switches

| Switch | Accepts | Effect |
|---|---|---|
| `--sldr <path>` | `.sldr` binary file | Load additional dhātu entries for annotation |
| `--slpr <path>` | `.slpr` binary file | Load additional prātipadika entries for annotation |
| `--slsr <path>` | `.slsr` binary file | Load additional sandhi rule entries for annotation |

Each switch may be specified multiple times to load several extension files of the same type. Extensions are merged with the built-in registry at runtime. ID collisions between extensions and built-ins, or between multiple extensions, are **fatal errors** — the codec rejects ambiguous data rather than silently resolving it.

When extensions are used, the output `.slbc` container records the extension provenance in its DICT chunk(s) using hybrid mode (§9.6), so downstream decoders can locate the required registries.

---

# 11. Summary

SLBC v0.8 is a binary encoding of Sanskrit that:

1. **Encodes from IAST** — phonemically unambiguous input, no script ambiguity
2. **Decodes to IAST or Devanāgarī** — with script conversion via external tools for other scripts
3. **Preserves Pāṇinian structure** — bit positions map to phonological features
4. **Enables algebraic transformations** — sandhi, guṇa, vṛddhi, saṃprasāraṇa as bit ops
5. **Supports progressive annotation** — deterministic encoding → ML-added vyākaraṇa
6. **Offers two clean modes** — pāṭha (readable text) and vyākhyā (analyzed text)
7. **Treats vyākaraṇa as indivisible** — morphology, kāraka, sandhi, anvaya travel together

---

# 12. Open Items (TBD)

Items acknowledged as under-specified in v0.8. Struck-through items have been resolved in subsequent revisions.

| ID | Section | Item | Notes |
|---|---|---|---|
| ~~TBD-1~~ | ~~§6.2~~ | ~~**Numeral encoding format**~~ | **Resolved in v0.9** — see §6.3. Dual-layer design: bhāṣā layer uses SAṄKHYĀ_START (0x3E) + ULEB128 digit count + R→L prātipadika digit-words (*aṅkānāṃ vāmato gatiḥ*); lipi layer uses NUM (0x2F) + L→R digit-glyph span. |
| TBD-2 | §8 | **Vyākaraṇa sub-field wire formats** — Detailed byte layouts for subanta fields (vibhakti/vacana/liṅga packing), tiṅanta fields (lakāra/puruṣa/vacana/pada/prayoga/gaṇa packing), kāraka sub-tag internals, and sandhi history sub-tag internals. Envelope structure is defined; field-level encoding is not. | Deferred to v0.9. |
| ~~TBD-3~~ | ~~§9.2~~ | ~~**DICT chunk internal format**~~ | **Resolved in v0.8** — see §9.6. |
| TBD-4 | §6.1 | **ANU (anunāsika modifier) interaction with anusvāra** — `ANU` (0x36, bhāṣā control) is the chandrabindu/anunāsika modifier; `ṃ` (0x3A, glottal group) is the anusvāra. Their distinct roles and sequencing rules (e.g., does ANU precede or follow the svara it modifies?) need formal specification. | Deferred to v0.9. |
| TBD-5 | — | **OṂkāra (ॐ) encoding** — Whether ॐ receives a dedicated byte or is encoded analytically as `o` + `ṃ` (0x89 0x3A). Liturgical and typographic considerations apply. | Deferred to v0.9. |
| TBD-6 | §9.3.2 | **Sandhi rule binary metadata** — The sandhi rule binary entry format (§9.3.2) specifies a type nibble and sūtra reference string, but does not define structured fields for rule applicability (e.g., left-context class, right-context class, transformation). This is needed for automated sandhi application. | Deferred to v0.9. |

---

# Appendix A. Changelog

## A.1 v0.9 Changes (from v0.8-draft)

| # | Change | Sections affected |
|---|---|---|
| 1 | **TBD-1 resolved: Numeral encoding** — Dual-layer design. Bhāṣā layer uses SAṄKHYĀ_START (0x3E) + ULEB128 digit count + R→L prātipadika digit-words (*aṅkānāṃ vāmato gatiḥ* convention). Lipi layer uses NUM (0x2F) + L→R digit-glyph span with auxiliary symbols (separator, fractional mark, signs). Explicit count prevents ambiguity with an-stem numerals (pañca, sapta, aṣṭa, nava) whose inflected forms are identical to their prātipadika. | §6.1, §6.3 (new), §7.5 |
| 2 | **SAṄKHYĀ_START (0x3E) assigned** — Previously reserved bhāṣā control slot allocated for numeral digit-word spans. All 8 bhāṣā control slots now occupied. | §6.1 |
| 3 | **NUM (0x2F) digit-glyph span specified** — Lipi-layer numeral rendering: 10 digit glyphs (0x00–0x09) plus auxiliary symbols (separator, fractional mark, positive/negative signs). Implicit termination on first byte ≥ 0x10. | §6.3.3 |
| 4 | **Extraction logic expanded** — Added bhāṣā-only extraction mode. SAṄKHYĀ spans preserved in bhāṣā extraction (phonemic content). NUM spans stripped in bhāṣā extraction (lipi-only). | §7.5 |

## A.2 v0.8 Changes (from v0.7-draft)

| # | Change | Sections affected |
|---|---|---|
| 1 | **Registry system fully specified** — Three registries (dhātu, prātipadika, sandhi rule) with TSV authoring format, binary runtime format, file headers, and per-entry metadata byte layouts. | §9.1–§9.8 (rewritten) |
| 2 | **Standard vs. extension registry architecture** — Standard registries are built into the codec binary. Extension registries are loaded at runtime via `--sldr`, `--slpr`, `--slsr` CLI switches. Extensions are additive; ID collisions are fatal. | §9.2 (new) |
| 3 | **Dhātu ID allocation table** — Pre-allocated ID ranges by gaṇa with ULEB128 byte costs. Extension IDs must use range 2000+. | §9.4 (new) |
| 4 | **Prātipadika stem class table** — 15 stem classes (0–14) covering all major declension paradigms. | §9.5 (new) |
| 5 | **DICT chunk wire format defined** — Registry type byte, mode byte (embedded/external/hybrid), mode-specific payloads, and resolution order. Resolves TBD-3. | §9.6 (new) |
| 6 | **Registry CLI commands** — `slbc registry compile`, `slbc registry inspect`, `slbc registry lookup`, `slbc registry stats`. | §10 (expanded) |
| 7 | **Registry extension switch table** — `--sldr`, `--slpr`, `--slsr` with merge semantics and provenance recording. | §10.1 (new) |
| 8 | **TBD-6 added** — Sandhi rule binary metadata needs structured applicability fields for automated sandhi. | §12 |

## A.3 v0.7 Changes (from v0.6-draft)

| # | Change | Sections affected |
|---|---|---|
| 1 | **Accent field re-ordered:** `A=00` is now neutral (was `01`). Udātta, anudātta, svarita shifted accordingly. | §4, §4.1, §5.1, §5.3, §10 |
| 2 | **All svara hex values recalculated** to reflect new accent field mapping. | §4.1, §5.1, §5.3, §10 |
| 3 | **Binary format advisory added** (§1.4): `.slbc` is opaque binary; `0x00` (`ka`) is a valid byte. Transport and I/O requirements specified. | §1.4 (new) |
| 4 | **Non-varga COLUMN semantics documented** (§3.5): COLUMN values for PLACE ∈ {5–7} are ordinal, not articulatory. Domain guard for varga algebra specified. | §3.2, §3.5 (new), §5.2 |
| 5 | **Saṃprasāraṇa la ↔ ḷ** formally documented as a structural irregularity requiring special-case handling. | §5.3 |
| 6 | **Explicit vowel convention** added (§4.2): no inherent vowel, no virāma byte; decoder responsibilities defined. | §4.2 (new) |
| 7 | **Chunk framing defined** (§7.4): type (1 byte) + payload length (ULEB128-32, 1–5 bytes) + payload. Unified variable-length integer encoding with §8.4 and §9.1. Defined for bhāṣā+lipi chunks. Vyākaraṇa chunk payload internals deferred. | §7.4 (new) |
| 8 | **Extended header length** clarified: length of additional header beyond the fixed 14-byte header; `0x0000` means no extension. | §7.1 |
| 9 | **Chunk type namespace note** added to prevent confusion with control bytes sharing the same numerical values. | §7.3 |
| 10 | **ULEB128 parser statefulness** documented: positional parsing eliminates byte-value ambiguity within structured payloads. | §8.4 |
| 11 | **Byte order** standardized as little-endian for all multi-byte container fields. | §7.1 |
| 12 | **TBD section** added (§12) for formally tracked open items: numeral format, vyākaraṇa sub-field wire formats, DICT internals, ANU/anusvāra interaction, OṂkāra encoding. | §12 (new) |
