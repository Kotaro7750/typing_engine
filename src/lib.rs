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
//! // Add a typing target and activate it
//! let query_request = QueryRequest::new(
//!     &vocabularies_slice,
//!     VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
//!     VocabularySeparator::None,
//!     VocabularyOrder::InOrder,
//! );
//! engine.add_target(query_request);
//! engine.activate_target(0, Duration::from_millis(0));
//!
//! // Start the typing game for the active target
//! engine.start_target(0).expect("Failed to start typing");
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
//!     .construct_display_info_of_target(0, LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
//!     .unwrap();
//!
//! // Get the result after finishing
//! let result = engine
//!     .construct_aggregated_result()
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
//! // Add and start typing
//! let vocabularies = vec![&vocabulary];
//! let query_request = QueryRequest::new(
//!     &vocabularies,
//!     VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
//!     VocabularySeparator::None,
//!     VocabularyOrder::InOrder,
//! );
//! engine.add_target(query_request);
//! engine.activate_target(0, std::time::Duration::from_millis(0));
//! engine.start_target(0).expect("Failed to start typing");
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
//!
//! # Switching Between Multiple Targets
//! ```rust
//! use std::num::NonZeroUsize;
//! use std::time::Duration;
//! use typing_engine::{
//!     QueryRequest, TypingEngine, VocabularyEntry, VocabularyOrder, VocabularyQuantifier,
//!     VocabularySeparator, VocabularySpellElement,
//! };
//!
//! let entry_a = VocabularyEntry::new(
//!     "阿".to_string(),
//!     vec![VocabularySpellElement::Normal(
//!         "あ".to_string().try_into().unwrap(),
//!     )],
//! )
//! .unwrap();
//! let entry_i = VocabularyEntry::new(
//!     "胃".to_string(),
//!     vec![VocabularySpellElement::Normal(
//!         "い".to_string().try_into().unwrap(),
//!     )],
//! )
//! .unwrap();
//!
//! let mut engine = TypingEngine::new();
//! let request_a = QueryRequest::new(
//!     vec![&entry_a].as_ref(),
//!     VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
//!     VocabularySeparator::None,
//!     VocabularyOrder::InOrder,
//! );
//! let request_i = QueryRequest::new(
//!     vec![&entry_i].as_ref(),
//!     VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
//!     VocabularySeparator::None,
//!     VocabularyOrder::InOrder,
//! );
//!
//! engine.add_target(request_a);
//! engine.add_target(request_i);
//!
//! engine.activate_target(0, Duration::from_millis(0));
//! engine.start_target(0).expect("Failed to start target 0");
//! engine
//!     .stroke_key_with_elapsed_time('a'.try_into().unwrap(), Duration::from_millis(100))
//!     .unwrap();
//!
//! engine.activate_target(1, Duration::from_millis(200));
//! engine.start_target(1).expect("Failed to start target 1");
//! engine
//!     .stroke_key_with_elapsed_time('i'.try_into().unwrap(), Duration::from_millis(300))
//!     .unwrap();
//!
//! // Aggregate results across started targets.
//! let result = engine.construct_aggregated_result().unwrap();
//! assert_eq!(result.summary().spell().finished_count(), 2);
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
