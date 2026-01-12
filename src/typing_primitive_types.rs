//! Primitive types that represent what is being typed.
//!
//! This module groups the core primitives:
//! - `key_stroke`: Per-keystroke input during typing.
//! - `vocabulary`: Per-vocabulary units that compose a query.
//! - `spell`: Spellings for vocabularies, as sequences of fundamental characters such as ASCII
//!   or Hiragana.
//! - `chunk`: Internal typing units used mainly for Japanese, where a sequence of key strokes can
//!   produce multiple spell characters.

pub(crate) mod chunk;
pub(crate) mod chunk_key_stroke_dictionary;
pub mod key_stroke;
pub mod spell;
pub mod vocabulary;
