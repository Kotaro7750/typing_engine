pub use crate::query::{QueryRequest, VocabularyOrder, VocabularyQuantifier, VocabularySeparator};
pub use crate::spell::{SpellString, SpellStringError};
pub use crate::typing_engine::*;
pub use crate::vocabulary::VocabularyEntry;

mod chunk;
mod chunk_key_stroke_dictionary;
mod key_stroke;
mod query;
mod spell;
mod typing_engine;
mod utility;
mod vocabulary;

#[cfg(test)]
mod test_utility;
