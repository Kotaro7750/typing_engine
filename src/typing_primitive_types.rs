//! Primitive types which construct targets being typed.
//!
//! In this module, mainly 4 modules are defined:
//! - `key_stroke`: Module for each key stroke during typing.
//! - `vocabulary`: Module for each vocabulary. Vocabularies are comonents of query being typed.
//! - `spell`: Module for each spell. Spells are representations of vocabularies in a form of
//!     sequence of fundamental characters such as alphabets and hiraganas.
//! - `chunk`: Private module for each chunk, which is specific concept to Japanese. In Japanase, some
//!     sequences of key strokes can type multiple characters of spell.

pub(crate) mod chunk;
pub(crate) mod chunk_key_stroke_dictionary;
pub mod key_stroke;
pub mod spell;
pub mod vocabulary;
