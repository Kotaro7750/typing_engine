pub use crate::display_info::DisplayInfo;
pub use crate::key_stroke::{KeyStrokeChar, KeyStrokeCharError};
pub use crate::query::{QueryRequest, VocabularyOrder, VocabularyQuantifier, VocabularySeparator};
pub use crate::spell::{SpellString, SpellStringError};
pub use crate::typing_engine::*;
pub use crate::vocabulary::VocabularyEntry;

mod chunk;
mod chunk_key_stroke_dictionary;
pub mod display_info;
mod key_stroke;
mod query;
mod spell;
mod statistics;
mod typing_engine;
mod utility;
mod vocabulary;

#[cfg(test)]
mod test_utility;
