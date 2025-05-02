pub use crate::display_info::DisplayInfo;
pub use crate::query::{QueryRequest, VocabularyOrder, VocabularyQuantifier, VocabularySeparator};
pub use crate::statistics::lap_statistics::LapInfo;
pub use crate::statistics::result::{TypingResult, TypingResultSummary};
pub use crate::statistics::result::{TypingResultStatistics, TypingResultStatisticsTarget};
pub use crate::statistics::statistics_counter::EntitySummaryStatistics;
pub use crate::statistics::{LapRequest, OnTypingStatisticsTarget};
pub use crate::typing_engine::*;
pub use crate::typing_primitive_types::key_stroke::{KeyStrokeChar, KeyStrokeCharError};
pub use crate::typing_primitive_types::spell::{SpellString, SpellStringError};
pub use crate::typing_primitive_types::vocabulary::{VocabularyEntry, VocabularySpellElement};

pub mod display_info;
mod query;
mod statistics;
mod typing_engine;
pub mod typing_primitive_types;
mod utility;

#[cfg(test)]
mod test_utility;
