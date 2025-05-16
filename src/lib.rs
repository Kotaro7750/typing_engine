//! Library for creating a typing game engine.
//! 
//! # Example
//! ```rust
//! use std::num::NonZeroUsize;
//! use std::time::Duration;
//! use typing_engine::{
//!     LapRequest, QueryRequest, TypingEngine, VocabularyEntry, VocabularyOrder,
//!     VocabularyQuantifier, VocabularySeparator, VocabularySpellElement,
//! };
//!
//! // Create a new typing engine
//! let mut engine = TypingEngine::new();
//!
//! // Create vocabulary entries
//! let vocabularies = vec![VocabularyEntry::new(
//!     "Hello".to_string(),
//!     vec![
//!         VocabularySpellElement::Normal("H".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Normal("e".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Normal("l".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Normal("l".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Normal("o".to_string().try_into().unwrap()),
//!     ],
//! )
//! .unwrap()];
//!
//! // Prepare vocabulary entries for the engine
//! let vocabularies_slice: Vec<&VocabularyEntry> = vocabularies.iter().collect();
//!
//! // Initialize the engine with query request
//! let query_request = QueryRequest::new(
//!     &vocabularies_slice,
//!     VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
//!     VocabularySeparator::None,
//!     VocabularyOrder::InOrder,
//! );
//! engine.init(query_request);
//!
//! // Start the typing game
//! engine.start().expect("Failed to start typing");
//!
//! // Process key strokes
//! engine
//!     .stroke_key_with_elapsed_time('H'.try_into().unwrap(), Duration::from_millis(100))
//!     .unwrap();
//! engine
//!     .stroke_key_with_elapsed_time('e'.try_into().unwrap(), Duration::from_millis(200))
//!     .unwrap();
//! engine
//!     .stroke_key_with_elapsed_time('l'.try_into().unwrap(), Duration::from_millis(300))
//!     .unwrap();
//! engine
//!     .stroke_key_with_elapsed_time('l'.try_into().unwrap(), Duration::from_millis(400))
//!     .unwrap();
//! engine
//!     .stroke_key_with_elapsed_time('o'.try_into().unwrap(), Duration::from_millis(500))
//!     .unwrap();
//!
//! // Get display information
//! let display_info = engine
//!     .construct_display_info(LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
//!     .unwrap();
//!
//! // Get the result after finishing
//! let result = engine
//!     .construct_result(LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
//!     .expect("Failed to get result");
//! ```
//!
//! # Japanese Example
//! ```rust
//! use std::num::NonZeroUsize;
//! use typing_engine::{
//!     TypingEngine, VocabularyEntry, VocabularySpellElement,
//!     QueryRequest, VocabularyQuantifier, VocabularySeparator, VocabularyOrder
//! };
//!
//! // Create a new typing engine
//! let mut engine = TypingEngine::new();
//!
//! // Create Japanese vocabulary with compounds
//! let vocabulary = VocabularyEntry::new(
//!     "昨日の敵は今日の友".to_string(),
//!     vec![
//!         VocabularySpellElement::Compound((
//!             "きのう".to_string().try_into().unwrap(),
//!             NonZeroUsize::new(2).unwrap()
//!         )),
//!         VocabularySpellElement::Normal("の".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Normal("てき".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Normal("は".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Compound((
//!             "きょう".to_string().try_into().unwrap(),
//!             NonZeroUsize::new(2).unwrap()
//!         )),
//!         VocabularySpellElement::Normal("の".to_string().try_into().unwrap()),
//!         VocabularySpellElement::Normal("とも".to_string().try_into().unwrap()),
//!     ]
//! ).unwrap();
//!
//! // Initialize and start typing
//! let vocabularies = vec![&vocabulary];
//! let query_request = QueryRequest::new(
//!     &vocabularies,
//!     VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
//!     VocabularySeparator::None,
//!     VocabularyOrder::InOrder,
//! );
//! engine.init(query_request);
//! engine.start().expect("Failed to start typing");
//!
//! // Continue with key strokes...
//! ```
//! 
//! # Using Vocabulary Parser
//! 
//! You can also parse vocabulary entries from strings:
//! 
//! ```rust
//! use typing_engine::parse_vocabulary_entry;
//! 
//! // Parse a simple entry
//! let entry = parse_vocabulary_entry("Hello:H,e,l,l,o").unwrap();
//! 
//! // Parse Japanese entry with compounds
//! let japanese_entry = parse_vocabulary_entry("[昨日]の敵は[今日]の友:きのう,の,てき,は,きょう,の,とも").unwrap();
//! 
//! // Using escaped characters
//! let escaped_entry = parse_vocabulary_entry(r"a\:b:a,\:,b").unwrap();
//! assert_eq!(escaped_entry.view(), "a:b");
//! ```

pub use crate::display_info::DisplayInfo;
pub use crate::query::{QueryRequest, VocabularyOrder, VocabularyQuantifier, VocabularySeparator};
pub use crate::statistics::lap_statistics::LapInfo;
pub use crate::statistics::result::{TypingResult, TypingResultSummary};
pub use crate::statistics::skill_statistics::public::{EntitySkillStatistics, SkillStatistics};
pub use crate::statistics::statistics_counter::EntitySummaryStatistics;
pub use crate::statistics::LapRequest;
pub use crate::typing_engine::*;
pub use crate::typing_primitive_types::key_stroke::{KeyStrokeChar, KeyStrokeCharError};
pub use crate::typing_primitive_types::spell::{SpellString, SpellStringError};
pub use crate::typing_primitive_types::vocabulary::{VocabularyEntry, VocabularySpellElement};
pub use crate::vocabulary_parser::{parse_vocabulary_entry, VocabularyParseError};

pub mod display_info;
mod query;
mod statistics;
mod typing_engine;
pub mod typing_primitive_types;
mod utility;
mod vocabulary_parser;

#[cfg(test)]
mod test_utility;
