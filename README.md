# Preamble

[![CI](https://github.com/abhyagra/slbc/actions/workflows/ci.yml/badge.svg)](https://github.com/abhyagra/slbc/actions/workflows/ci.yml)

## On the Occasion of Mahāśivarātri 

This draft of **Sanskrit Linguistic Binary Codec (SLBC)** is released for preview and comments on 15 February 2026.

> **Kṛṣṇa-pakṣe Caturdaśyāṁ, Uttarāṣāḍhā-nakṣatre, Vyatīpāta-yoge, Viṣṭi-karaṇe, Vikrama-saṁvate 2082, Parābhava-saṁvatsare.**

Traditionally aligned with *Mahāśivarātri*, also remembered as *udgama* of the **Māheśvara-sūtrāṇi** — symbolically reflects the phonological foundation upon which the Aṣṭādhyāyī — backbone of Pāṇinian grammar — is constructed.

In keeping with the spirit of that tradition, this work begins with salutations to:

> **Ācārya Pāṇini**,
> whose *Aṣṭādhyāyī* distilled the Sanskrit language into an algorithmic system of extraordinary compression, precision, and generative power;
> and to the lineage of grammarians preceding him, whom he himself acknowledged while constructing his system upon earlier foundations.

---

## Why SLBC Exists

Sanskrit is not merely a script. It is a sound system, a generative grammar, and a tradition of precise analysis.

Most digital Sanskrit today is stored and processed as glyph strings. That works for display. But it breaks the moment you ask Sanskrit-native questions: What are the phonemes? Where are the pada boundaries? What changed due to sandhi? What is the stem or dhātu? What is the kāraka role? What is the anvaya?

SLBC exists to represent Sanskrit in a way that remains faithful to what Sanskrit *is*.

---

## The Problem Unicode Cannot Solve

Unicode is excellent at writing systems. It can store Devanāgarī glyphs, Roman transliterations, ligatures, marks, and punctuation. But Sanskrit computation needs something else.

If you store Sanskrit as Devanāgarī text: the inherent vowel is implicit, conjuncts require guessing, segmentation gets conflated with rendering, and sandhi becomes a regex game. If you store it as IAST: it is phonemically clearer, but still a string — and string-based systems struggle to express algebraic transformations (guṇa, vṛddhi, jaśtva), stable canonicalization, or layered grammatical annotation without corruption.

Unicode distinguishes *ka* (क) from *ga* (ग), but does not encode the fact that they share identical *sthāna*, differ only in voicing, and that *jaśtva* is structurally a minimal transformation. Similarly, Unicode encodes *anusvāra* as a symbol, but not as a nasal feature whose realization depends on the following *varga*.

Unicode is a writing substrate. Sanskrit needs a **language substrate**.

---

## Phoneme-First: The Udgama of Varṇa-Architecture

Sanskrit's stability comes from its sound-structure. Varṇas are not arbitrary letters — they are classified by articulation. Transformations are governed by systematic rules. A representation that starts with phonemes instead of glyphs gains unambiguous identity, stable canonicalization, and true transformability.

The *Māheśvara-sūtrāṇi* are not a mere list of sounds. They are a computational compression scheme. By arranging *varṇas* in a specific sequence and marking boundaries with *anubandhas*, Pāṇini achieved dynamic set construction (*pratyāhāra*), group-level transformations, sound-class targeting through minimal symbols, and algorithmic *sandhi* resolution.

The elegance lies not in enumeration, but in **structure**.

The Sanskrit *varṇa*-system represents varṇas fundamentally in a grid:

| Place (*sthāna*) | Voiceless | Aspirated | Voiced | Voiced Aspirated | Nasal |
| ---------------- | --------- | --------- | ------ | ---------------- | ----- |
| Velar            | k         | kh        | g      | gh               | ṅ     |
| Palatal          | c         | ch        | j      | jh               | ñ     |
| Retroflex        | ṭ         | ṭh        | ḍ      | ḍh               | ṇ     |
| Dental           | t         | th        | d      | dh               | n     |
| Labial           | p         | ph        | b      | bh               | m     |

This two-dimensional grid reflects *sthāna* (place of articulation) and *prayatna* (manner of articulation). Operations such as **jaśtva** become mechanical:

> Replace COLUMN bits corresponding to voiceless with voiced.

No lexical lookup is required. This is not orthography — this is algebra over the human vocal apparatus.

SLBC begins with the minimum truth that survives every script: the phonemic stream. Everything else — glyphs, ligatures, visual punctuation — becomes a projection.

---

## On Śabda and Sandhi

In the grammatical tradition, *śabda* is not merely written form. It is structured articulation. Each varṇa corresponds to a defined configuration of the vocal apparatus. Each transformation preserves articulatory law. Each rule operates within a constrained generative system.

In the *Aṣṭādhyāyī*, rules such as 8.2.39 *jhalāṁ jaśo 'nte*, 8.4.62 *raṣābhyāṁ no ṇaḥ samānapade*, and 6.1.77 *iko yaṇ aci* are not arbitrary transformations. They operate on sound classes, position in sequence, articulatory compatibility, and constraints of the vocal system. The human vocal tract is the substrate. The grammar is an abstract machine describing lawful transitions of that substrate.

Where orthographic systems preserve appearance, SLBC preserves relation. Where character encodings store symbols, SLBC stores generative structure.

---

## Why Grammar Is Indivisible

Sanskrit grammar is not "tags you sprinkle on text." It is a unified analytical system. You cannot meaningfully keep morphology without kāraka, kāraka without anvaya, or sandhi history without morphological grounding.

When you partially annotate Sanskrit, you create a false sense of certainty: a "kartā" label without a clear subanta analysis is fragile; a dependency edge without role grounding is arbitrary; sandhi history without pre-sandhi lexical identity becomes byte trivia.

SLBC treats vyākaraṇa as a complete envelope: either the text is just readable, or it is fully analyzed. This mirrors the reality of the tradition — Pāṇini did not design four separate optional modules. He designed one system.

---

## Sanskrit Deserves an Intermediate Representation

Modern computing matured by inventing intermediate representations. Compilers don't operate directly on raw source strings — they transform source into a structured IR; analysis and optimization happen at the IR layer; many outputs can be generated from the same IR.

Sanskrit needs the same leap. It has stable phonology, algebraic transformations, deep structure (samāsa, kāraka, anvaya), strong internal consistency across centuries, and massive corpora waiting to be cleaned, aligned, and analyzed.

Without an IR, Sanskrit tooling remains script-bound, fragile, heuristic-heavy, difficult to compose, and hard to keep correct over time. With an IR, you can build an ecosystem: deterministic encoding (truth layer), progressive annotation (analysis layer), multiple outputs (script conversion, TTS, metrical analysis, search), and reliable storage for critical editions.

SLBC exists to become that substrate.

---

## What SLBC Is

SLBC is a binary encoding that stores Sanskrit as a phonemic stream with optional rendering metadata, plus an all-or-nothing grammatical analysis layer.

It encodes articulatory geometry, morphophonemic algebra, Pāṇinian derivational context, and explicit separation between *bhāṣā* (sound) and *lipi* (script).

> Unicode is a photograph of the manuscript.
> SLBC is the musical score behind the chant.
> Vyākaraṇa is the full analysis of how that score was composed and how it functions inside a sentence.

## What SLBC Is Not

SLBC is not trying to replace Devanāgarī (for reading), Unicode (for writing systems), existing transliteration tools, or scholarly editions. It is also not claiming to "solve grammar." Its job is to store Sanskrit in a form that allows grammar to be added correctly, preserve correctness boundaries, and prevent loss of information across transformations.

---

## Quick Start

```bash
# Build
cargo build --workspace

# Round-trip test: IAST → SLBC → IAST + Devanāgarī
cargo run -p slbc-cli -- roundtrip "dharmakṣetre kurukṣetre"
# Output (IAST): dharmakṣetre kurukṣetre
# Output (Deva): धर्मक्षेत्रे कुरुक्षेत्रे
# ✓ Round-trip PASSED

# Encode to file
cargo run -p slbc-cli -- encode "oṃ namaḥ śivāya" -o test.slbc

# Decode
cargo run -p slbc-cli -- decode -i test.slbc --to iast
cargo run -p slbc-cli -- decode -i test.slbc --to devanagari

# Inspect a byte — see its phonological structure
cargo run -p slbc-cli -- inspect --byte 0x00
# Vyañjana 'ka' — kaṇṭhya (velar), aghoṣa alpaprāṇa

# Inspect a byte stream
cargo run -p slbc-cli -- inspect --from-hex "1B 40 33 24 40"

# Algebraic transforms — Pāṇinian operations as bit manipulation
cargo run -p slbc-cli -- transform --op guna 0x44       # i → e
cargo run -p slbc-cli -- transform --op jastva 0x00      # ka → ga
cargo run -p slbc-cli -- transform --op nasal 0x00       # ka → ṅa
```

---

## Scope of This Draft (v0.11)

**Bhāṣā layer status: FROZEN.** All phonemic encoding — svaras, vyañjanas, bhāṣā control bytes — is fully specified with no open items. The byte-space classification (§2), vyañjana grid (§3), svara encoding (§4), algebraic operations (§5), and control bytes (§6) are stable. Remaining open items (TBD-2, TBD-6) are scoped entirely within the META envelope and registry layer — they do not affect any byte in the bhāṣā or lipi lanes.

This initial public draft supports:

### Encoding Path

* IAST transliteration input

Other transliteration systems are intentionally excluded at this stage due to phonological ambiguity.

### Decoding Path

* IAST output
* Unicode text output (standardized rendering)

Unicode ↔ IAST normalization may rely on established transliteration tools (e.g., Sanscript, Indic Transliteration libraries, Aksharamukha), and will be out-of-scope in terms of integration with SLBC at this stage.

Future revisions may expand supported input formats.

---

## Project Roadmap

The development of SLBC is organized into the following phases. Each phase builds upon the prior, and transitions are gated by completion and review.

| Phase | Description | Scope | Status |
| ----- | ----------- | ----- | ------ |
| **1** | **Specification** — Define the SLBC binary layout, byte-space classification, svara/vyañjana encoding, control bytes, container format, vyākaraṇa layer, and registries. | 📄 **Full specification:** [SLBC_spec.md](SLBC_spec.md) | ✅ Bhāṣā + lipi frozen; vyākaraṇa wire formats (TBD-2, TBD-6) deferred |
| **2** | **MVP Codec in Rust (CLI)** — Implement the core encoder-decoder in Rust with a CLI interface supporting `encode`, `decode`, `inspect`, `transform`, and `roundtrip` commands. Pāṭha mode (bha + lipi). Vyākaraṇa annotation commands deferred pending TBD-2/6. | `slbc` crate + binary | 🔨 In progress |
| **3** | **Test Automation & Validation** — Build a comprehensive test suite: round-trip correctness, algebraic operation verification (guṇa, vṛddhi, jaśtva, saṃprasāraṇa), container format parsing, edge cases (pluta svaras, Vedic accents, jihvāmūlīya/upadhmānīya). | CI pipeline + test corpus | ⏳ Planned |
| **4** | **Stream Encoding/Decoding (gRPC, REST)** — Expose the codec as a network service supporting streaming encode/decode over gRPC and REST, enabling integration with external NLP pipelines and annotation services. | `slbc-server` | ⏳ Planned |
| **5** | **WASM-WASI Support** — Compile the codec to WebAssembly (WASM + WASI) for browser-based and sandboxed environments, enabling client-side encoding/decoding without native installation. | `slbc.wasm` | ⏳ Planned |
| **6** | **Community & Extensions** — Incorporate feature requests, expand registry coverage (dhātu, prātipadika, sandhi-rule registries), explore ML-assisted vyākaraṇa annotation, and broaden transliteration input support. | Ongoing | ⏳ Planned |

> **Current milestone:** Phase 2 — MVP Codec in Rust (pāṭha mode)

### Workspace Structure

```
├── crates/
│   ├── slbc-core/     # Core library — encoding, decoding, transforms, container format
│   ├── slbc-cli/      # CLI binary — encode, decode, inspect, transform, roundtrip
│   ├── slbc-grpc/     # gRPC service (Phase 4 — planned)
│   ├── slbc-rest/     # REST service (Phase 4 — planned)
│   └── slbc-wasm/     # WASM module (Phase 5 — planned)
├── SLBC_spec.md       # Full specification
└── LICENSE            # Apache-2.0
```

---

## Intent

SLBC does not attempt to modernize Sanskrit.

It attempts to restore the algebraic elegance of Pāṇini, encode sound-class mechanics natively, enable direct rule application on byte structures, and provide a research-grade representation for computational Sanskrit.

If SLBC succeeds, Sanskrit computation becomes less heuristic, more deterministic, more composable, more interoperable, and more faithful to the tradition. The long-term result is not just better tools — it is a digital foundation where Sanskrit texts can be preserved, compared, corrected, and analyzed without being destroyed by scripts, encodings, or ad-hoc pipelines.

Sanskrit deserves the score — not only the photograph.

This draft is offered in the spirit of inquiry, formal discipline, collaboration and refinement.

---
