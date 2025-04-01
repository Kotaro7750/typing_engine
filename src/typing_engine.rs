use std::error::Error;
use std::fmt::Display;
use std::time::Instant;

use crate::display_info::{DisplayInfo, KeyStrokeDisplayInfo, SpellDisplayInfo, ViewDisplayInfo};
use crate::query::QueryRequest;
use crate::statistics::result::{construct_result, TypingResultStatistics};
use crate::statistics::{DisplayStringBuilder, LapRequest, StatisticsManager};
use crate::typing_engine::processed_chunk_info::ProcessedChunkInfo;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
use crate::typing_primitive_types::vocabulary::{
    view_position_of_spell_for_vocabulary_infos, VocabularyInfo,
};

mod processed_chunk_info;

/// Error type returned from [`TypingEngine`].
#[derive(Debug)]
pub struct TypingEngineError {
    kind: TypingEngineErrorKind,
}

impl TypingEngineError {
    fn new(kind: TypingEngineErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for TypingEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl Error for TypingEngineError {}

#[derive(Debug)]
enum TypingEngineErrorKind {
    MustBeInitialized,
    MustBeStarted,
    AlreadyFinished,
    NotFinished,
}

impl TypingEngineErrorKind {
    fn as_str(&self) -> &'static str {
        use TypingEngineErrorKind::*;

        match *self {
            MustBeInitialized => "not initialized",
            MustBeStarted => "not started",
            AlreadyFinished => "already finished",
            NotFinished => "not finished",
        }
    }
}

impl Display for TypingEngineErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum TypingEngineState {
    Uninitialized,
    Ready,
    Started,
}

/// The main engine of typing game.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypingEngine {
    state: TypingEngineState,
    start_time: Option<Instant>,
    processed_chunk_info: Option<ProcessedChunkInfo>,
    vocabulary_infos: Option<Vec<VocabularyInfo>>,
    statistics_manager: StatisticsManager,
    display_string_builder: DisplayStringBuilder,
}

impl TypingEngine {
    /// Construct an empty engine.
    ///
    /// This method only do construct typing engine, so you must call [`init`](Self::init()) method to construct
    /// query and [`start`](Self::start()) method to start typing.
    pub fn new() -> Self {
        Self {
            state: TypingEngineState::Uninitialized,
            start_time: None,
            processed_chunk_info: None,
            vocabulary_infos: None,
            statistics_manager: StatisticsManager::new(),
            display_string_builder: DisplayStringBuilder::new(),
        }
    }

    /// Initialize [`TypingEngine`](TypingEngine) by constructing and resetting query using [`QueryRequest`].
    pub fn init(&mut self, query_request: QueryRequest) {
        let query = query_request.construct_query();
        let (vocabulary_infos, chunks) = query.decompose();

        let (processed_chunk_info, statistical_events) = ProcessedChunkInfo::new(chunks);
        self.processed_chunk_info.replace(processed_chunk_info);
        self.vocabulary_infos.replace(vocabulary_infos);

        statistical_events.iter().for_each(|statistical_event| {
            self.statistics_manager
                .consume_event(statistical_event.clone());
            self.display_string_builder
                .consume_event(statistical_event.clone());
        });

        self.state = TypingEngineState::Ready;
    }

    /// Append query using [`QueryRequest`].
    ///
    /// If this method is called before initializing via calling [`init`](Self::init()) method, this
    /// method returns error.
    pub fn append_query(&mut self, query_request: QueryRequest) -> Result<(), TypingEngineError> {
        if self.is_initialized() {
            assert!(self.processed_chunk_info.is_some());
            assert!(self.vocabulary_infos.is_some());

            let (mut vocabulary_infos, chunks) = query_request.construct_query().decompose();

            self.vocabulary_infos
                .as_mut()
                .unwrap()
                .append(&mut vocabulary_infos);

            let statistical_events = self
                .processed_chunk_info
                .as_mut()
                .unwrap()
                .append_chunks(chunks);

            statistical_events.iter().for_each(|statistical_event| {
                self.statistics_manager
                    .consume_event(statistical_event.clone());
            });

            Ok(())
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::MustBeInitialized,
            ))
        }
    }

    /// Start typing.
    ///
    /// If this method is called before initializing via calling [`init`](Self::init()) method, this
    /// method returns error.
    pub fn start(&mut self) -> Result<(), TypingEngineError> {
        if self.is_initialized() {
            assert!(self.processed_chunk_info.is_some());
            assert!(self.vocabulary_infos.is_some());

            self.processed_chunk_info
                .as_mut()
                .unwrap()
                .move_next_chunk();

            self.state = TypingEngineState::Started;
            self.start_time.replace(Instant::now());
            Ok(())
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::MustBeInitialized,
            ))
        }
    }

    /// Give a key stroke to [`TypingEngine`].
    ///
    /// If this method is called before initializing via calling [`start`](Self::start()) method,
    /// this method returns error.
    pub fn stroke_key(&mut self, key_stroke: KeyStrokeChar) -> Result<bool, TypingEngineError> {
        if self.is_started() {
            let pci = self.processed_chunk_info.as_mut().unwrap();
            if pci.is_finished() {
                return Err(TypingEngineError::new(
                    TypingEngineErrorKind::AlreadyFinished,
                ));
            }

            let elapsed_time = self.start_time.as_ref().unwrap().elapsed();

            let (_, statistical_events) = pci.stroke_key(key_stroke, elapsed_time);
            statistical_events.iter().for_each(|statistical_event| {
                self.statistics_manager
                    .consume_event(statistical_event.clone());
                self.display_string_builder
                    .consume_event(statistical_event.clone());
            });

            Ok(pci.is_finished())
        } else {
            Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted))
        }
    }

    /// Construct [`DisplayInfo`] for composing UI.
    ///
    /// If this method is called before starting via calling [`start`](Self::start()) method,
    /// this method returns error.
    pub fn construct_display_info(
        &self,
        lap_request: LapRequest,
    ) -> Result<DisplayInfo, TypingEngineError> {
        if self.is_started() {
            let processed_chunk_info = self.processed_chunk_info.as_ref().unwrap();

            let (spell_display_info, key_stroke_display_info) = processed_chunk_info
                .construct_display_info(
                    lap_request,
                    self.statistics_manager.key_stroke_statistics_counter(),
                    self.statistics_manager
                        .ideal_key_stroke_statistics_counter(),
                    self.statistics_manager.spell_statistics_counter(),
                );

            let mut display_string_builder = self.display_string_builder.clone();
            processed_chunk_info
                .snapshot()
                .into_iter()
                .for_each(|event| {
                    display_string_builder.consume_event(event);
                });
            equality_check_for_construct_display_info_and_display_string_builder(
                &spell_display_info,
                &key_stroke_display_info,
                &display_string_builder,
            );

            let view_position_of_spell = view_position_of_spell_for_vocabulary_infos(
                self.vocabulary_infos.as_ref().unwrap(),
            );

            let view = self
                .vocabulary_infos
                .as_ref()
                .unwrap()
                .iter()
                .map(|vocabulary_info| vocabulary_info.view().to_string())
                .reduce(|accum, item| accum + &item)
                .unwrap();

            let view_display_info =
                ViewDisplayInfo::new(&spell_display_info, view, view_position_of_spell);

            Ok(DisplayInfo::new(
                view_display_info,
                spell_display_info,
                key_stroke_display_info,
            ))
        } else {
            Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted))
        }
    }

    pub fn construst_result_statistics(
        &self,
        lap_request: LapRequest,
    ) -> Result<TypingResultStatistics, TypingEngineError> {
        if self.is_started() {
            let confirmed_chunks = self
                .processed_chunk_info
                .as_ref()
                .unwrap()
                .confirmed_chunks();

            if self.processed_chunk_info.as_ref().unwrap().is_finished() {
                Ok(construct_result(confirmed_chunks, lap_request))
            } else {
                Err(TypingEngineError::new(TypingEngineErrorKind::NotFinished))
            }
        } else {
            Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted))
        }
    }

    fn is_initialized(&self) -> bool {
        !matches!(self.state, TypingEngineState::Uninitialized)
    }

    fn is_started(&self) -> bool {
        matches!(self.state, TypingEngineState::Started)
    }
}

impl Default for TypingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Output equality check of [`DisplayStringBuilder`](DisplayStringBuilder)
/// This function is temporal and for debug use
fn equality_check_for_construct_display_info_and_display_string_builder(
    sdi: &SpellDisplayInfo,
    kdi: &KeyStrokeDisplayInfo,
    dsb: &DisplayStringBuilder,
) {
    // Check equality between SpellDisplayStringBuilder and SpellDisplayInfo
    assert_eq!(dsb.spell().spell(), sdi.spell());
    assert_eq!(
        &dsb.spell().cursor_position().construct_vec(),
        sdi.current_cursor_positions()
    );
    assert_eq!(dsb.spell().wrong_positions(), sdi.missed_positions());
    assert_eq!(dsb.spell().last_position(), sdi.last_position());

    // Check equality between KeyStrokeDisplayStringBuilder and KeyStrokeDisplayInfo
    assert_eq!(dsb.key_stroke().key_stroke(), kdi.key_stroke());
    assert_eq!(
        dsb.key_stroke().cursor_position(),
        kdi.current_cursor_position()
    );
    assert_eq!(dsb.key_stroke().wrong_positions(), kdi.missed_positions());
}
