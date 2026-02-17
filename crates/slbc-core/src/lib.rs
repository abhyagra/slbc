//! Sanskrit Linguistic Binary Codec (SLBC)
//!
//! A binary encoding of Sanskrit that preserves Pāṇinian phonological structure.
//! Encodes from IAST, decodes to IAST or Devanāgarī.

pub mod container;
pub mod decoder;
pub mod encoder;
pub mod inspect;
pub mod numeral;
pub mod transform;
pub mod types;
