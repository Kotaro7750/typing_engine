pub use crate::display_info::DisplayInfo;
pub use crate::key_stroke::{KeyStrokeChar, KeyStrokeCharError};
pub use crate::query::{QueryRequest, VocabularyOrder, VocabularyQuantifier, VocabularySeparator};
pub use crate::statistics::result::{TypingResultStatistics, TypingResultStatisticsTarget};
pub use crate::statistics::{LapRequest, OnTypingStatisticsTarget};
pub use crate::typing_engine::*;
pub use crate::typing_primitive_types::spell::{SpellString, SpellStringError};
pub use crate::vocabulary::{VocabularyEntry, VocabularySpellElement};

mod chunk_key_stroke_dictionary;
pub mod display_info;
mod key_stroke;
mod query;
mod statistics;
mod typing_engine;
pub mod typing_primitive_types;
mod utility;
mod vocabulary;

#[cfg(test)]
mod test_utility;
