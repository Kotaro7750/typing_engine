//! Internal helpers for managing multiple typing targets.
//!
//! A typing target bundles per-query state (`ProcessedChunkInfo`, vocabulary info, display builder,
//! and statistics). `TargetsSwitcher` owns the list of targets and tracks which one is active.

use std::cmp::max;
use std::time::Duration;

use super::processed_chunk_info::ProcessedChunkInfo;
use crate::DisplayInfo;
use crate::KeyStrokeChar;
use crate::LapRequest;
use crate::display_info::IdealKeyStrokeDisplayInfo;
use crate::display_info::KeyStrokeDisplayInfo;
use crate::display_info::SpellDisplayInfo;
use crate::display_info::ViewDisplayInfo;
use crate::statistics::DisplayStringBuilder;
use crate::statistics::StatisticsManager;
use crate::statistics::statistical_event::StatisticalEvent;
use crate::typing_primitive_types::vocabulary::VocabularyInfo;
use crate::typing_primitive_types::vocabulary::view_position_of_spell_for_vocabulary_infos;

/// Lifecycle status for a single typing target.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum TypingTargetStatus {
    Initialized,
    Started,
    Finished,
}

/// A per-query bundle of typing state, statistics, and display-building context.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(super) struct TypingTarget {
    status: TypingTargetStatus,
    last_activated_global_elapsed_time: Option<Duration>,
    last_key_stroke_global_elapsed_time: Option<Duration>,
    processed_chunk_info: ProcessedChunkInfo,
    vocabulary_infos: Vec<VocabularyInfo>,
    display_string_builder: DisplayStringBuilder,
    statistics_manager: StatisticsManager,
}

impl TypingTarget {
    /// Create a target with the provided chunk and vocabulary state.
    ///
    /// The target starts in the initialized state and must be explicitly started
    /// before it can accept key strokes.
    pub(super) fn new(
        processed_chunk_info: ProcessedChunkInfo,
        vocabulary_infos: Vec<VocabularyInfo>,
    ) -> Self {
        Self {
            status: TypingTargetStatus::Initialized,
            last_activated_global_elapsed_time: None,
            last_key_stroke_global_elapsed_time: None,
            processed_chunk_info: processed_chunk_info,
            vocabulary_infos: vocabulary_infos,
            display_string_builder: DisplayStringBuilder::new(),
            statistics_manager: StatisticsManager::new(),
        }
    }

    pub(super) fn processed_chunk_info(&self) -> &ProcessedChunkInfo {
        &self.processed_chunk_info
    }

    /// Mutable access to the target's processed chunk information.
    pub(super) fn processed_chunk_info_mut(&mut self) -> &mut ProcessedChunkInfo {
        &mut self.processed_chunk_info
    }

    /// Vocabulary info used for building view strings and lap stats.
    pub(super) fn vocabulary_infos(&self) -> &Vec<VocabularyInfo> {
        &self.vocabulary_infos
    }

    /// Mutable access to vocabulary info to allow appending new query data.
    pub(super) fn vocabulary_infos_mut(&mut self) -> &mut Vec<VocabularyInfo> {
        &mut self.vocabulary_infos
    }

    /// Snapshot of the display string builder for reconstructing UI state.
    pub(super) fn display_string_builder(&self) -> &DisplayStringBuilder {
        &self.display_string_builder
    }

    /// Statistics manager that tracks per-target summaries.
    pub(super) fn statistics_manager(&self) -> &StatisticsManager {
        &self.statistics_manager
    }

    /// Last global elapsed time at which this target received a key stroke.
    pub(super) fn last_key_stroke_global_elapsed_time(&self) -> Option<Duration> {
        self.last_key_stroke_global_elapsed_time
    }

    /// Consume a statistical event and update display and statistics state.
    pub(super) fn consume_event(&mut self, event: StatisticalEvent) {
        self.statistics_manager.consume_event(event.clone());
        self.display_string_builder.consume_event(event);
    }

    /// Returns whether the target has started (including already finished).
    pub(super) fn is_started(&self) -> bool {
        self.status == TypingTargetStatus::Started || self.status == TypingTargetStatus::Finished
    }

    /// Record activation timing so global elapsed time can be mapped to local time.
    pub(super) fn on_activate(&mut self, elapsed_time: Duration) {
        self.last_activated_global_elapsed_time = Some(elapsed_time);
    }

    /// Start the target; must only be called once.
    pub(super) fn start(&mut self) {
        assert!(self.status == TypingTargetStatus::Initialized);
        self.status = TypingTargetStatus::Started;
        self.processed_chunk_info_mut().move_next_chunk();
    }

    /// Apply a key stroke in global time and return generated events plus completion state.
    ///
    /// Preconditions: the target must be started and the current chunk must not be finished.
    /// The returned boolean indicates whether the target became finished after applying the stroke.
    pub(super) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        global_elapsed_time: Duration,
    ) -> (Vec<StatisticalEvent>, bool) {
        assert!(self.is_started());
        assert!(!self.processed_chunk_info().is_finished());

        let target_local_elapsed_time = self.convert_to_local_time(global_elapsed_time);
        let pci = self.processed_chunk_info_mut();

        let (_, statistical_events) = pci.stroke_key(key_stroke, target_local_elapsed_time);
        statistical_events.iter().for_each(|statistical_event| {
            self.consume_event(statistical_event.clone());
        });

        self.last_key_stroke_global_elapsed_time = Some(global_elapsed_time);

        (
            statistical_events,
            self.processed_chunk_info().is_finished(),
        )
    }

    /// Build display information for the current target state.
    ///
    /// This method assumes the target has started.
    pub(super) fn construct_display_info(&self, lap_request: LapRequest) -> DisplayInfo {
        assert!(self.is_started());

        let processed_chunk_info = self.processed_chunk_info();

        let mut display_string_builder = self.display_string_builder().clone();
        let mut statistics_manager = self.statistics_manager().clone();

        processed_chunk_info
            .snapshot()
            .into_iter()
            .for_each(|event| {
                display_string_builder.consume_event(event.clone());
                statistics_manager.consume_event(event);
            });

        let view_position_of_spell =
            view_position_of_spell_for_vocabulary_infos(self.vocabulary_infos());

        let lap_info = processed_chunk_info
            .construct_lap_statistics(lap_request.clone(), &view_position_of_spell);

        let key_stroke_display_info = KeyStrokeDisplayInfo::new_with(
            display_string_builder.key_stroke(),
            statistics_manager.key_stroke_statistics_counter().into(),
        );
        let ideal_key_stroke_display_info = IdealKeyStrokeDisplayInfo::new_with(
            statistics_manager
                .ideal_key_stroke_statistics_counter()
                .into(),
        );
        let spell_display_info = SpellDisplayInfo::new_with(
            display_string_builder.spell(),
            statistics_manager.spell_statistics_counter().into(),
        );

        let view = self
            .vocabulary_infos()
            .iter()
            .map(|vocabulary_info| vocabulary_info.view().to_string())
            .reduce(|accum, item| accum + &item)
            .unwrap();

        let view_display_info =
            ViewDisplayInfo::new(&spell_display_info, view, view_position_of_spell);

        DisplayInfo::new(
            view_display_info,
            spell_display_info,
            key_stroke_display_info,
            ideal_key_stroke_display_info,
            lap_info,
        )
    }

    /// Convert the given global elapsed time to the local elapsed time of this target.
    /// This is assumed to be used for a key stroke time conversion.
    ///
    /// Global elapsed time: time since the typing session started. This progresses even though
    /// this target is inactive.
    /// Local elapsed time: accumulated time since activation of this target. This ignores the time
    /// between last key stroke and re-activation.
    fn convert_to_local_time(&self, global_elapsed_time: Duration) -> Duration {
        // Conversion concept:
        // tX: global time
        // TX: local time
        // X is numerical for key stroke and 's' for activation of this target
        //
        //  case1: first key stroke after activation
        //  t0                        ts <-> t1
        // -+--+ ~target is inactive~ +------+----
        //  T0                               T1 = T0 + (t1 - ts)
        //
        //  case2: subsequent key stroke after activation
        //  ts   t0 <-> t1
        // -+----+------+----
        //       T0     T1 = T0 + (t1 - t0)

        // T0
        let last_key_stroke_local_elapsed_time = self
            .processed_chunk_info()
            .last_key_stroke_elapsed_time()
            .unwrap_or(Duration::ZERO);

        // ts
        let last_activated_global_elapsed_time = self
            .last_activated_global_elapsed_time
            .unwrap_or(Duration::ZERO);
        // t0
        let last_key_stroke_global_elapsed_time = self
            .last_key_stroke_global_elapsed_time
            .unwrap_or(Duration::ZERO);

        // We can simplify the calculation: T1 = T0 + (t1 - max(ts, t0))
        last_key_stroke_local_elapsed_time
            + (global_elapsed_time
                - max(
                    last_activated_global_elapsed_time,
                    last_key_stroke_global_elapsed_time,
                ))
    }
}

/// Tracks multiple typing targets and which one is currently active.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(super) struct TargetsSwitcher {
    targets: Vec<TypingTarget>,
    active_target_index: Option<usize>,
}

impl TargetsSwitcher {
    /// Create an empty target switcher with no active target.
    pub(super) fn new() -> Self {
        Self {
            targets: vec![],
            active_target_index: None,
        }
    }

    /// Return the index of the active target, if any.
    pub(super) fn active_target_index(&self) -> Option<usize> {
        self.active_target_index
    }

    /// Add a new target. This does not activate it.
    pub(super) fn add_target(&mut self, target: TypingTarget) {
        self.targets.push(target);
    }

    /// Activate the target at the given index.
    /// Returns `true` if the target was successfully activated, `false` if the specified target is
    /// not found.
    pub(super) fn activate_target(&mut self, target_index: usize, elapsed_time: Duration) -> bool {
        if let Some(target) = self.targets.get_mut(target_index) {
            // Re-activate results in no-op
            if self.active_target_index == Some(target_index) {
                return true;
            }
            self.active_target_index = Some(target_index);
            target.on_activate(elapsed_time);
            true
        } else {
            false
        }
    }

    pub(super) fn active_target(&self) -> Option<&TypingTarget> {
        match self.active_target_index {
            Some(index) => {
                assert!(self.targets.get(index).is_some());
                self.targets.get(index)
            }
            None => None,
        }
    }

    /// Mutable access to the active target, if any.
    pub(super) fn active_target_mut(&mut self) -> Option<&mut TypingTarget> {
        match self.active_target_index {
            Some(index) => {
                assert!(self.targets.get(index).is_some());
                self.targets.get_mut(index)
            }
            None => None,
        }
    }

    /// Access a target by index.
    pub(super) fn target(&self, index: usize) -> Option<&TypingTarget> {
        self.targets.get(index)
    }

    /// Mutable access to a target by index.
    pub(super) fn target_mut(&mut self, index: usize) -> Option<&mut TypingTarget> {
        self.targets.get_mut(index)
    }

    /// Snapshot of all targets.
    pub(super) fn targets(&self) -> &[TypingTarget] {
        self.targets.as_slice()
    }
}

#[cfg(test)]
mod test {
    use std::time::Duration;

    use crate::typing_engine::processed_chunk_info::ProcessedChunkInfo;
    use crate::{gen_candidate, gen_candidate_key_stroke, gen_chunk_unprocessed};

    use super::{TargetsSwitcher, TypingTarget};

    fn dummy_target() -> TypingTarget {
        let (pci, _) = ProcessedChunkInfo::new(vec![]);
        TypingTarget::new(pci, vec![])
    }

    fn started_target_double_stroke() -> TypingTarget {
        let (pci, _) = ProcessedChunkInfo::new(vec![
            gen_chunk_unprocessed!(
                "a",
                vec![gen_candidate!(gen_candidate_key_stroke!("a"))],
                gen_candidate!(gen_candidate_key_stroke!("a"))
            ),
            gen_chunk_unprocessed!(
                "b",
                vec![gen_candidate!(gen_candidate_key_stroke!("b"))],
                gen_candidate!(gen_candidate_key_stroke!("b"))
            ),
        ]);
        let mut target = TypingTarget::new(pci, vec![]);
        target.start();
        target
    }

    #[test]
    fn empty_targets_switcher_has_no_active_target() {
        let mut switcher = TargetsSwitcher::new();

        assert!(switcher.active_target().is_none());
        assert!(switcher.active_target_mut().is_none());
    }

    #[test]
    fn add_target_not_activate_target() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(dummy_target());

        assert!(switcher.active_target().is_none());
        assert!(switcher.active_target_mut().is_none());
    }

    #[test]
    fn activate_target_first_time_switches_correctly() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(dummy_target());

        assert!(switcher.activate_target(0, Duration::from_secs(0)));
        assert!(switcher.active_target_index().is_some_and(|i| i == 0));
        assert!(switcher.active_target().is_some());
        assert!(switcher.active_target_mut().is_some());
    }

    #[test]
    fn activate_target_second_time_switches_correctly() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(dummy_target());
        switcher.add_target(dummy_target());
        switcher.activate_target(0, Duration::from_secs(0));

        assert!(switcher.activate_target(1, Duration::from_secs(0)));
        assert!(switcher.active_target_index().is_some_and(|i| i == 1));
        assert!(switcher.active_target().is_some());
        assert!(switcher.active_target_mut().is_some());
    }

    #[test]
    fn activate_target_to_non_exist_fails() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(dummy_target());
        switcher.activate_target(0, Duration::from_secs(0));

        assert!(!switcher.activate_target(1, Duration::from_secs(0)));
        assert!(switcher.active_target_index().is_some_and(|i| i == 0));
        assert!(switcher.active_target().is_some());
        assert!(switcher.active_target_mut().is_some());
    }

    #[test]
    fn local_time_first_key_after_activation_uses_activation_time() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(started_target_double_stroke());

        switcher.activate_target(0, Duration::from_secs(2));
        let target = switcher.active_target_mut().unwrap();
        target.stroke_key('a'.try_into().unwrap(), Duration::from_secs(5));

        assert_eq!(
            target
                .processed_chunk_info()
                .last_key_stroke_elapsed_time()
                .unwrap(),
            Duration::from_secs(3)
        );
    }

    #[test]
    fn local_time_subsequent_key_uses_last_key_time() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(started_target_double_stroke());

        switcher.activate_target(0, Duration::from_secs(2));
        let target = switcher.active_target_mut().unwrap();
        target.stroke_key('a'.try_into().unwrap(), Duration::from_secs(5));

        target.stroke_key('b'.try_into().unwrap(), Duration::from_secs(7));
        assert_eq!(
            target
                .processed_chunk_info()
                .last_key_stroke_elapsed_time()
                .unwrap(),
            Duration::from_secs(5)
        );
    }

    #[test]
    fn local_time_ignores_inactive_time_across_targets() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(started_target_double_stroke());
        switcher.add_target(started_target_double_stroke());

        switcher.activate_target(0, Duration::from_secs(0));
        switcher
            .active_target_mut()
            .unwrap()
            .stroke_key('a'.try_into().unwrap(), Duration::from_secs(1));

        switcher.activate_target(1, Duration::from_secs(2));
        switcher
            .active_target_mut()
            .unwrap()
            .stroke_key('a'.try_into().unwrap(), Duration::from_secs(3));

        switcher.activate_target(0, Duration::from_secs(10));
        switcher
            .active_target_mut()
            .unwrap()
            .stroke_key('b'.try_into().unwrap(), Duration::from_secs(12));

        assert_eq!(
            switcher
                .target(0)
                .unwrap()
                .processed_chunk_info()
                .last_key_stroke_elapsed_time()
                .unwrap(),
            Duration::from_secs(3)
        );
    }

    #[test]
    fn local_time_same_target_reactivate_uses_last_key_time() {
        let mut switcher = TargetsSwitcher::new();

        switcher.add_target(started_target_double_stroke());

        switcher.activate_target(0, Duration::from_secs(0));
        switcher
            .active_target_mut()
            .unwrap()
            .stroke_key('a'.try_into().unwrap(), Duration::from_secs(1));

        switcher.activate_target(0, Duration::from_secs(5));
        switcher
            .active_target_mut()
            .unwrap()
            .stroke_key('b'.try_into().unwrap(), Duration::from_secs(7));

        assert_eq!(
            switcher
                .target(0)
                .unwrap()
                .processed_chunk_info()
                .last_key_stroke_elapsed_time()
                .unwrap(),
            Duration::from_secs(7)
        );
    }
}
