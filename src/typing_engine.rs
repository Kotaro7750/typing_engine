use std::error::Error;
use std::fmt::Display;
use std::time::Duration;

use crate::display_info::DisplayInfo;
use crate::query::QueryRequest;
use crate::statistics::LapRequest;
use crate::statistics::result::TypingResult;
use crate::statistics::skill_statistics::SkillStatisticsManager;
use crate::typing_engine::multi_target::{TargetsSwitcher, TypingTarget};
use crate::typing_engine::processed_chunk_info::ProcessedChunkInfo;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;

mod multi_target;
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
    AlreadyStarted,
    AlreadyFinished,
    NotFinished,
    TargetNotFound,
}

impl TypingEngineErrorKind {
    fn as_str(&self) -> &'static str {
        use TypingEngineErrorKind::*;

        match *self {
            MustBeInitialized => "not initialized",
            MustBeStarted => "not started",
            AlreadyStarted => "already started",
            AlreadyFinished => "already finished",
            NotFinished => "not finished",
            TargetNotFound => "target not found",
        }
    }
}

impl Display for TypingEngineErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The main typing game engine.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TypingEngine {
    targets_switcher: TargetsSwitcher,
    skill_statistics_manager: SkillStatisticsManager,
}

impl TypingEngine {
    /// Construct an empty engine in the uninitialized state.
    ///
    /// This only constructs the engine; call [`add_target`](Self::add_target()) to add typing
    /// targets and then call [`start_target`](Self::start_target()) to start typing.
    pub fn new() -> Self {
        Self {
            targets_switcher: TargetsSwitcher::new(),
            skill_statistics_manager: SkillStatisticsManager::new(),
        }
    }

    /// Return the index of the currently active target, if any.
    ///
    /// The active target is selected via [`activate_target`](Self::activate_target()).
    pub fn active_target_index(&self) -> Option<usize> {
        self.targets_switcher.active_target_index()
    }

    /// Return the number of targets currently registered in the engine.
    ///
    /// This includes targets that have not been started yet.
    pub fn target_count(&self) -> usize {
        self.targets_switcher.targets().len()
    }

    /// Make the target at `target_index` the active target.
    ///
    /// Returns `true` when the index exists and the active target is updated.
    /// This does not start typing for the target, so you must call
    /// [`start_target`](Self::start_target()) separately.
    pub fn activate_target(&mut self, target_index: usize, elapsed_time: Duration) -> bool {
        self.targets_switcher
            .activate_target(target_index, elapsed_time)
    }

    /// Add an additional typing target built from a [`QueryRequest`].
    ///
    /// This only appends a target to the internal target list, and you must call
    /// [`start_target`](Self::start_target()) for starting typing for the added target.
    pub fn add_target(&mut self, query_request: QueryRequest) {
        let query = query_request.construct_query();
        let (vocabulary_infos, chunks) = query.decompose();

        let (processed_chunk_info, statistical_events) = ProcessedChunkInfo::new(chunks);

        let mut typing_target = TypingTarget::new(processed_chunk_info, vocabulary_infos);

        statistical_events.iter().for_each(|statistical_event| {
            self.skill_statistics_manager
                .consume_event(statistical_event.clone());

            typing_target.consume_event(statistical_event.clone());
        });

        self.targets_switcher.add_target(typing_target);
    }

    #[deprecated(
        since = "0.5.0",
        note = "Use Self::add_target() and Self::activate_target instead."
    )]
    /// Initialize [`TypingEngine`] by constructing and resetting query using [`QueryRequest`].
    /// Any previous added targets and statistics will be cleared.
    ///
    /// This method is equivalent to calling [`add_target`](Self::add_target()) and
    /// [`activate_target`](Self::activate_target()) with index 0.
    pub fn init(&mut self, query_request: QueryRequest) {
        // For backward compatibility, we should reset the internal state, not just add a target.
        self.targets_switcher = TargetsSwitcher::new();
        self.skill_statistics_manager = SkillStatisticsManager::new();

        self.add_target(query_request);
        assert!(self.targets_switcher.activate_target(0, Duration::ZERO));
    }

    #[deprecated(since = "0.5.0", note = "Use Self::append_query_to_target() instead.")]
    /// Append query using [`QueryRequest`].
    ///
    /// If this method is called before initializing via calling [`init`](Self::init()) method, this
    /// method returns error.
    pub fn append_query(&mut self, query_request: QueryRequest) -> Result<(), TypingEngineError> {
        // For backward compatibility, we should always use target index 0 and treat error as not
        // initialized.
        if let Ok(_) = self.append_query_to_target(0, query_request) {
            Ok(())
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::MustBeInitialized,
            ))
        }
    }

    /// Append additional query data to an existing target.
    ///
    /// The new chunks are merged into the target and statistics are updated to include the
    /// appended query. Returns error if the target does not exist.
    pub fn append_query_to_target(
        &mut self,
        target_index: usize,
        query_request: QueryRequest,
    ) -> Result<(), TypingEngineError> {
        if let Some(target) = self.targets_switcher.target_mut(target_index) {
            let (mut vocabulary_infos, chunks) = query_request.construct_query().decompose();

            target.vocabulary_infos_mut().append(&mut vocabulary_infos);
            let statistical_events = target.processed_chunk_info_mut().append_chunks(chunks);

            statistical_events.iter().for_each(|statistical_event| {
                self.skill_statistics_manager
                    .consume_event(statistical_event.clone());

                target.consume_event(statistical_event.clone());
            });

            Ok(())
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::TargetNotFound,
            ))
        }
    }

    #[deprecated(since = "0.5.0", note = "Use Self::start_target() instead.")]
    /// Start typing.
    ///
    /// This method is equivalent to calling [`start_target`](Self::start_target()) with index 0.
    pub fn start(&mut self) -> Result<(), TypingEngineError> {
        match self.start_target(0) {
            Ok(()) => Ok(()),
            Err(TypingEngineError {
                kind: TypingEngineErrorKind::TargetNotFound,
            }) => Err(TypingEngineError::new(
                TypingEngineErrorKind::MustBeInitialized,
            )),
            Err(e) => Err(e),
        }
    }

    /// Start typing for the specified target.
    ///
    /// If this method is called when specified target does not exist
    /// or called twice for same target, this method returns error.
    pub fn start_target(&mut self, target_index: usize) -> Result<(), TypingEngineError> {
        if let Some(target) = self.targets_switcher.target_mut(target_index) {
            if target.is_started() {
                Err(TypingEngineError::new(
                    TypingEngineErrorKind::AlreadyStarted,
                ))
            } else {
                target.start();

                Ok(())
            }
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::TargetNotFound,
            ))
        }
    }

    /// Give a key stroke to [`TypingEngine`] with a specified elapsed time.
    ///
    /// Note that the elapsed time is the time elapsed since the whole typing session started,
    /// not the time since the last key stroke or since the target started.
    pub fn stroke_key_with_elapsed_time(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> Result<bool, TypingEngineError> {
        if let Some(target) = self.targets_switcher.active_target_mut() {
            if !target.is_started() {
                return Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted));
            }

            if target.processed_chunk_info().is_finished() {
                return Err(TypingEngineError::new(
                    TypingEngineErrorKind::AlreadyFinished,
                ));
            }

            let (statistical_events, is_finished) = target.stroke_key(key_stroke, elapsed_time);

            statistical_events.iter().for_each(|statistical_event| {
                self.skill_statistics_manager
                    .consume_event(statistical_event.clone());
            });

            Ok(is_finished)
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::MustBeInitialized,
            ))
        }
    }

    #[deprecated(
        since = "0.5.0",
        note = "Use Self::construct_display_info_of_target() instead."
    )]
    /// Construct [`DisplayInfo`] for composing UI.
    ///
    /// If this method is called before starting via calling [`start`](Self::start()) method,
    /// this method returns error.
    pub fn construct_display_info(
        &self,
        lap_request: LapRequest,
    ) -> Result<DisplayInfo, TypingEngineError> {
        // For backward compatibility, we should always use target index 0 and treat error as not
        // started.
        if let Ok(display_info) = self.construct_display_info_of_target(0, lap_request) {
            Ok(display_info)
        } else {
            Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted))
        }
    }

    /// Construct [`DisplayInfo`] for composing UI for a specified target.
    ///
    /// If this method is called before starting via calling [`start_target`](Self::start_target()) method
    /// or speacified target is not found, this method returns error.
    pub fn construct_display_info_of_target(
        &self,
        target_index: usize,
        lap_request: LapRequest,
    ) -> Result<DisplayInfo, TypingEngineError> {
        if let Some(target) = self.targets_switcher.target(target_index) {
            if !target.is_started() {
                return Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted));
            }

            Ok(target.construct_display_info(lap_request))
        } else {
            Err(TypingEngineError::new(
                TypingEngineErrorKind::TargetNotFound,
            ))
        }
    }

    #[deprecated(
        since = "0.5.0",
        note = "Use Self::construct_aggregated_result() instead."
    )]
    /// Construct [`TypingResult`](TypingResult).
    /// This method must be called when typing is finished.
    ///
    /// `lap_request` is currently unused but kept for API compatibility with display construction.
    pub fn construct_result(
        &self,
        #[allow(unused)] lap_request: LapRequest,
    ) -> Result<TypingResult, TypingEngineError> {
        if let Some(target) = self.targets_switcher.target(0)
            && target.is_started()
        {
            let processed_chunk_info = target.processed_chunk_info();

            if processed_chunk_info.is_finished() {
                Ok(TypingResult::new(
                    processed_chunk_info.last_key_stroke_elapsed_time().unwrap(),
                    target
                        .statistics_manager()
                        .construct_typing_result_summary(),
                    self.skill_statistics_manager.construct_skill_statistics(),
                ))
            } else {
                Err(TypingEngineError::new(TypingEngineErrorKind::NotFinished))
            }
        } else {
            Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted))
        }
    }

    /// Construct a [`TypingResult`](TypingResult) aggregated across started targets.
    ///
    /// This method ignores targets that have not been started. Started targets are included in the
    /// aggregation even if they are not finished yet, so the summary represents progress so far.
    /// The returned elapsed time is the latest global key-stroke time across started targets (or
    /// zero if no key stroke has occurred yet).
    pub fn construct_aggregated_result(&self) -> Result<TypingResult, TypingEngineError> {
        let mut aggregated_summary = None;
        let mut last_key_stroke_elapsed_time: Option<Duration> = None;

        for target in self.targets_switcher.targets() {
            if !target.is_started() {
                continue;
            }

            let summary = target
                .statistics_manager()
                .construct_typing_result_summary();

            aggregated_summary = Some(match aggregated_summary {
                Some(current) => current + summary,
                None => summary,
            });

            if let Some(global_elapsed_time) = target.last_key_stroke_global_elapsed_time() {
                last_key_stroke_elapsed_time = Some(
                    last_key_stroke_elapsed_time
                        .map(|current| current.max(global_elapsed_time))
                        .unwrap_or(global_elapsed_time),
                );
            }
        }

        let Some(summary) = aggregated_summary else {
            return Err(TypingEngineError::new(TypingEngineErrorKind::MustBeStarted));
        };

        let total_time = last_key_stroke_elapsed_time.unwrap_or(Duration::ZERO);

        Ok(TypingResult::new(
            total_time,
            summary,
            self.skill_statistics_manager.construct_skill_statistics(),
        ))
    }
}

impl Default for TypingEngine {
    fn default() -> Self {
        Self::new()
    }
}
