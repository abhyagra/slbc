# Preamble

## On the Occasion of Mahāśivarātri 

This draft of **Sanskrit Linguistic Binary Codec (SLBC)** is released for preview and comments on 15 February 2026.

> **Kṛṣṇa-pakṣe Caturdaśyāṁ, Uttarāṣāḍhā-nakṣatre, Vyatīpāta-yoge, Viṣṭi-karaṇe, Vikrama-saṁvate 2082, Parābhava-saṁvatsare.**

Traditionally aligned with *Mahāśivarātri*, also remembered as *udgama* of the **Māheśvara-sūtrāṇi** - symbolically reflects the phonological foundation upon which the Aṣṭādhyāyī — backbone of Pāṇinian grammar — is constructed.


In keeping with the spirit of that tradition, this work begins with salutations to:

> **Ācārya Pāṇini**,
> whose *Aṣṭādhyāyī* distilled the Sanskrit language into an algorithmic system of extraordinary compression, precision, and generative power;
> and to the lineage of grammarians preceding him, whom he himself acknowledged while constructing his system upon earlier foundations.

---

## The Udgama of Varṇa-Architecture

The *Māheśvara-sūtrāṇi* are not a mere list of sounds.
They are a computational compression scheme.

By arranging *varṇas* in a specific sequence and marking boundaries with *anubandhas*, Pāṇini achieved:

* Dynamic set construction (*pratyāhāra*)
* Group-level transformations
* Sound-class targeting through minimal symbols
* Algorithmic *sandhi* resolution

The elegance lies not in enumeration, but in **structure**.

The Sanskrit *varṇa*-system represents different 'varṇa' fundamentally in grid-like structure similar to:

| Place (*sthāna*) | Voiceless | Aspirated | Voiced | Voiced Aspirated | Nasal |
| ---------------- | --------- | --------- | ------ | ---------------- | ----- |
| Velar            | k         | kh        | g      | gh               | ṅ     |
| Palatal          | c         | ch        | j      | jh               | ñ     |
| Retroflex        | ṭ         | ṭh        | ḍ      | ḍh               | ṇ     |
| Dental           | t         | th        | d      | dh               | n     |
| Labial           | p         | ph        | b      | bh               | m     |

This two-dimensional grid reflects:

* *sthāna* (place of articulation)
* *prayatna* (manner of articulation)

Operations such as **jaśtva** (voicing transformation) become mechanical:

> Replace COLUMN bits corresponding to voiceless with voiced.

No lexical lookup is required.

This is not orthography — this is algebra over the human vocal apparatus.

---

## On Śabda

In the grammatical tradition, *śabda* is not merely written form.

It is structured articulation.

Each varṇa corresponds to a defined configuration of the vocal apparatus.
Each transformation preserves articulatory law.
Each rule operates within a constrained generative system.

Where orthographic systems preserve appearance,
SLBC preserves relation.

Where character encodings store symbols,
SLBC stores generative structure.

---

## Sequence, Combination, and Sandhi

In the *Aṣṭādhyāyī*, rules such as:

* 8.2.39 *jhalāṁ jaśo 'nte*
* 8.4.62 *raṣābhyāṁ no ṇaḥ samānapade*
* 6.1.77 *iko yaṇ aci*

are not arbitrary transformations.

They operate on:

* Sound classes
* Position in sequence
* Articulatory compatibility
* Constraints of the vocal system

The human vocal tract is the substrate.

The grammar is an abstract machine describing lawful transitions of that substrate.

---

## Why Existing Encoding Systems Are Insufficient

Modern encodings such as ASCII and Unicode:

* Represent characters as visual symbols
* Encode script identity
* Do not encode articulatory structure
* Do not encode morphophonemic algebra
* Do not preserve generative class relationships

Unicode distinguishes:

* *ka* (क)
* *ga* (ग)

but does not encode the fact that:

* They share identical *sthāna*
* They differ only in voicing
* *jaśtva* is structurally a minimal transformation

Similarly, Unicode encodes *anusvāra* as a symbol, but not as a vowel nasalization feature whose realization depends on the following *varga*.

ASCII and Unicode are orthographic encodings.

SLBC is a **linguistic encoder-decoder**.

It encodes:

* Articulatory geometry
* Morphophonemic algebra
* Pāṇinian derivational context
* Explicit separation between *bhāṣā* (sound) and *lipi* (script)

It is not a script replacement.
It is a structural representation layer.

---

## Scope of This Draft (v0.9)

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
| **1** | **Specification** — Define the SLBC binary layout, byte-space classification, svara/vyañjana encoding, control bytes, container format, vyākaraṇa layer, and registries. | 📄 **Full specification:** [SLBC_spec.md](SLBC_spec.md) | 📝 Draft |
| **2** | **MVP Codec in Rust (CLI)** — Implement the core encoder-decoder in Rust with a CLI interface supporting `encode`, `decode`, `inspect`, `transform`, and `roundtrip` commands as defined in the spec §10. | `slbc` crate + binary | ⏳ Planned |
| **3** | **Test Automation & Validation** — Build a comprehensive test suite: round-trip correctness, algebraic operation verification (guṇa, vṛddhi, jaśtva, saṃprasāraṇa), container format parsing, edge cases (pluta svaras, Vedic accents, jihvāmūlīya/upadhmānīya). | CI pipeline + test corpus | ⏳ Planned |
| **4** | **Stream Encoding/Decoding (gRPC, REST)** — Expose the codec as a network service supporting streaming encode/decode over gRPC and REST, enabling integration with external NLP pipelines and annotation services. | `slbc-server` | ⏳ Planned |
| **5** | **WASM-WASI Support** — Compile the codec to WebAssembly (WASM + WASI) for browser-based and sandboxed environments, enabling client-side encoding/decoding without native installation. | `slbc.wasm` | ⏳ Planned |
| **6** | **Community & Extensions** — Incorporate feature requests, expand registry coverage (dhātu, prātipadika, sandhi-rule registries), explore ML-assisted vyākaraṇa annotation, and broaden transliteration input support. | Ongoing | ⏳ Planned |

> **Current milestone:** Phase 1 — Specification (Draft, v0.8)

---

## Intent

SLBC does not attempt to modernize Sanskrit.

It attempts to:

* Restore the algebraic elegance of Pāṇini
* Encode sound-class mechanics natively
* Enable direct rule application on byte structures
* Provide a research-grade representation for computational Sanskrit


This draft is offered in the spirit of inquiry, formal discipline, collaboration and refinement.

---
