use std::{num::NonZeroUsize, time::Duration};

use result::TypingResultSummary;
use serde::{Deserialize, Serialize};

pub(crate) mod lap_statistics;
pub(crate) mod multi_target_position_convert;
pub(crate) mod result;
pub(crate) mod statistical_event;
pub(crate) mod statistics_counter;

use crate::{
    statistics::statistical_event::StatisticalEvent,
    typing_primitive_types::chunk::{inflight::ChunkSpellCursorPosition, ChunkSpell},
    KeyStrokeChar,
};
use lap_statistics::PrimitiveLapStatisticsBuilder;
use statistics_counter::PrimitiveStatisticsCounter;

use self::multi_target_position_convert::BaseTarget;

#[deprecated(note = "Use `LapInfo` and `EntitySummaryStatistics` via `DisplayInfo` instead")]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OnTypingStatisticsTarget {
    // 対象を何個打ち終えたか
    finished_count: usize,
    // クエリに対象は何個あるか
    whole_count: usize,
    // 1回もミスタイプしないで打ち終えた対象は何個あるか
    completely_correct_count: usize,
    // ミスタイプした対象は重複込みで何個あるか
    // 重複というのは1つの対象に対して複数回ミスタイプした場合にもカウントされるため
    wrong_count: usize,
    // ラップ当たりの対象数
    targets_per_lap: Option<NonZeroUsize>,
    // 各ラップ末の経過時間
    lap_end_time: Option<Vec<Duration>>,
    // 各ラップ末の位置
    lap_end_position: Vec<usize>,
}

impl OnTypingStatisticsTarget {
    pub(crate) fn new(
        finished_count: usize,
        whole_count: usize,
        completely_correct_count: usize,
        wrong_count: usize,
        targets_per_lap: Option<NonZeroUsize>,
        lap_end_time: Option<Vec<Duration>>,
        lap_end_position: Vec<usize>,
    ) -> Self {
        assert_eq!(targets_per_lap.is_some(), lap_end_time.is_some());

        Self {
            finished_count,
            whole_count,
            completely_correct_count,
            wrong_count,
            targets_per_lap,
            lap_end_time,
            lap_end_position,
        }
    }

    /// Get count of finished targets.
    #[deprecated(
        note = "Use `EntitySummaryStatistics::finished_count()` via `KeyStrokeDisplayInfo::summary_statistics()` instead"
    )]
    pub fn finished_count(&self) -> usize {
        self.finished_count
    }

    /// Get count of whole targets.
    #[deprecated(
        note = "Use `EntitySummaryStatistics::whole_count()` via `KeyStrokeDisplayInfo::summary_statistics()` instead"
    )]
    pub fn whole_count(&self) -> usize {
        self.whole_count
    }

    /// Get count of targets that are finished without miss.
    #[deprecated(
        note = "Use `EntitySummaryStatistics::completely_correct_count()` via `KeyStrokeDisplayInfo::summary_statistics()` instead"
    )]
    pub fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }

    /// Get count of wrong typed targets.
    /// Multiple miss types in same targets are counted separately.
    #[deprecated(
        note = "Use `EntitySummaryStatistics::wrong_count()` via `KeyStrokeDisplayInfo::summary_statistics()` instead"
    )]
    pub fn wrong_count(&self) -> usize {
        self.wrong_count
    }

    /// Get lap end time of target.
    /// This returns [`None`](std::option::Option::None) when target is not a target for take laps.
    #[deprecated(note = "Use `DisplayInfo::lap_info()` instead")]
    pub fn lap_end_time(&self) -> Option<&Vec<Duration>> {
        self.lap_end_time.as_ref()
    }

    /// Get lap end positions of target.
    /// Each positions is converted from requested target.
    #[deprecated(note = "Use `DisplayInfo::lap_info()` instead")]
    pub fn lap_end_positions(&self) -> &Vec<usize> {
        &self.lap_end_position
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LapRequest {
    KeyStroke(NonZeroUsize),
    IdealKeyStroke(NonZeroUsize),
    Spell(NonZeroUsize),
    Chunk(NonZeroUsize),
}

impl LapRequest {
    fn construct_base_target(&self) -> BaseTarget {
        match self {
            Self::KeyStroke(_) => BaseTarget::KeyStroke,
            Self::IdealKeyStroke(_) => BaseTarget::IdealKeyStroke,
            Self::Spell(_) => BaseTarget::Spell,
            Self::Chunk(_) => BaseTarget::Chunk,
        }
    }
}

/// Helper function to construct OnTypingStatisticsTarget from PrimitiveStatisticsCounter and
/// PrimitiveLapStatisticsBuilder.
pub(crate) fn construct_on_typing_statistics_target(
    psc: &PrimitiveStatisticsCounter,
    plb: &PrimitiveLapStatisticsBuilder,
) -> OnTypingStatisticsTarget {
    OnTypingStatisticsTarget::new(
        psc.finished_count(),
        psc.whole_count(),
        psc.completely_correct_count(),
        psc.wrong_count(),
        plb.targets_per_lap(),
        plb.lap_end_time().cloned(),
        plb.lap_end_positions().clone(),
    )
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing display string
pub(crate) struct DisplayStringBuilder {
    spell: SpellDisplayStringBuilder,
    key_stroke: KeyStrokeDisplayStringBuilder,
}

impl DisplayStringBuilder {
    pub(crate) fn new() -> Self {
        Self {
            spell: SpellDisplayStringBuilder::new(),
            key_stroke: KeyStrokeDisplayStringBuilder::new(),
        }
    }

    /// Consume event and update display string.
    pub(crate) fn consume_event(&mut self, event: StatisticalEvent) {
        match event {
            StatisticalEvent::KeyStrokeCorrect(key_stroke_correct_context) => {
                if !key_stroke_correct_context.wrong_key_strokes().is_empty() {
                    self.key_stroke.add_to_wrong_positions();
                }

                self.key_stroke
                    .append(key_stroke_correct_context.key_stroke());
            }
            StatisticalEvent::SpellFinished(spell_finished_context)
            | StatisticalEvent::SpellDeemedFinished(spell_finished_context) => {
                if spell_finished_context.wrong_key_stroke_count() != 0 {
                    self.spell
                        .add_to_wrong_positions(spell_finished_context.spell());
                }
                self.spell
                    .advance_head_position(spell_finished_context.spell().as_ref());
            }
            StatisticalEvent::ChunkAdded(chunk_added_context) => {
                self.spell.append(chunk_added_context.spell().as_ref());
            }
            StatisticalEvent::KeyStrokeSnapshotted(key_stroke_snapshotted_context) => {
                self.key_stroke
                    .append_without_advancing_cursor(key_stroke_snapshotted_context.key_stroke());

                if let Some(wrong_key_strokes) = key_stroke_snapshotted_context.wrong_key_strokes()
                {
                    if !wrong_key_strokes.is_empty() {
                        self.key_stroke.add_to_wrong_positions();
                    }
                }
            }
            StatisticalEvent::InflightSpellSnapshotted(inflight_spell_snapshotted_context) => {
                if !inflight_spell_snapshotted_context
                    .wrong_key_strokes()
                    .is_empty()
                {
                    self.spell
                        .add_to_wrong_positions(inflight_spell_snapshotted_context.spell());
                }

                self.spell.set_cursor_position(
                    inflight_spell_snapshotted_context.chunk_spell_cursor_position(),
                );
            }
            _ => {}
        }
    }

    pub(crate) fn spell(&self) -> &SpellDisplayStringBuilder {
        &self.spell
    }

    pub(crate) fn key_stroke(&self) -> &KeyStrokeDisplayStringBuilder {
        &self.key_stroke
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// An enum representing cursor position of spell.
pub(crate) enum SpellCursorPosition {
    Single(usize),
    Double(usize, usize),
}

impl SpellCursorPosition {
    /// Convert self into vec form
    pub(crate) fn construct_vec(&self) -> Vec<usize> {
        match self {
            Self::Single(i) => vec![*i],
            Self::Double(i1, i2) => vec![*i1, *i2],
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing display string of spell.
pub(crate) struct SpellDisplayStringBuilder {
    /// Display string of spell.
    spell: String,
    /// Cursor position of spell.
    /// This is None when spell is not set.
    cursor_positions: Option<SpellCursorPosition>,
    /// Wrong positions of spell.
    wrong_positions: Vec<usize>,
    /// Head position of unfinished spell.
    current_head_position: usize,
}

impl SpellDisplayStringBuilder {
    fn new() -> Self {
        Self {
            spell: String::new(),
            cursor_positions: None,
            wrong_positions: vec![],
            current_head_position: 0,
        }
    }

    pub(crate) fn spell(&self) -> &str {
        &self.spell
    }

    pub(crate) fn cursor_position(&self) -> SpellCursorPosition {
        self.cursor_positions
            .as_ref()
            .map_or(SpellCursorPosition::Single(self.last_position() + 1), |p| {
                p.clone()
            })
    }

    pub(crate) fn wrong_positions(&self) -> &[usize] {
        &self.wrong_positions
    }

    /// Returns the index of last spell to be typed.
    /// This method returns 0 when spell is empty.
    pub(crate) fn last_position(&self) -> usize {
        if self.spell().chars().count() == 0 {
            0
        } else {
            self.spell.chars().count() - 1
        }
    }

    /// Append spell to display string.
    fn append(&mut self, spell: &str) {
        self.spell.push_str(spell);
    }

    /// Advance head position which is used like cursor.
    fn advance_head_position(&mut self, spell: &str) {
        self.current_head_position += spell.chars().count();
    }

    /// Set cursor position.
    fn set_cursor_position(&mut self, chunk_spell_cursor_position: &ChunkSpellCursorPosition) {
        if chunk_spell_cursor_position.is_cursor_count_double() {
            self.cursor_positions = Some(SpellCursorPosition::Double(
                self.current_head_position,
                self.current_head_position + 1,
            ));
        } else {
            self.cursor_positions = Some(SpellCursorPosition::Single(self.current_head_position));
        }
    }

    /// Add current head position to wrong positions.
    fn add_to_wrong_positions(&mut self, chunk_spell: &ChunkSpell) {
        for i in 0..chunk_spell.count() {
            self.wrong_positions.push(self.current_head_position + i);
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing display string of key stroke.
pub(crate) struct KeyStrokeDisplayStringBuilder {
    key_stroke: String,
    cursor_position: usize,
    wrong_positions: Vec<usize>,
}

impl KeyStrokeDisplayStringBuilder {
    fn new() -> Self {
        Self {
            key_stroke: String::new(),
            cursor_position: 0,
            wrong_positions: vec![],
        }
    }

    pub(crate) fn key_stroke(&self) -> &str {
        &self.key_stroke
    }

    pub(crate) fn cursor_position(&self) -> usize {
        self.cursor_position
    }

    pub(crate) fn wrong_positions(&self) -> &[usize] {
        &self.wrong_positions
    }

    /// Append key stroke to display string and advance cursor.
    fn append(&mut self, key_stroke: &KeyStrokeChar) {
        self.append_without_advancing_cursor(key_stroke);
        self.cursor_position += 1;
    }

    /// Append key stroke to display string without advancing cursor.
    fn append_without_advancing_cursor(&mut self, key_stroke: &KeyStrokeChar) {
        self.key_stroke.push(key_stroke.clone().into());
    }

    /// Add current cursor position to wrong positions.
    fn add_to_wrong_positions(&mut self) {
        self.wrong_positions.push(self.cursor_position);
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Holding and updating whole realtime statistics.
pub(crate) struct StatisticsManager {
    key_stroke: PrimitiveStatisticsCounter,
    ideal_key_stroke: PrimitiveStatisticsCounter,
    spell: PrimitiveStatisticsCounter,
    chunk: PrimitiveStatisticsCounter,
}

impl StatisticsManager {
    pub(crate) fn new() -> Self {
        Self {
            key_stroke: PrimitiveStatisticsCounter::empty_counter(),
            ideal_key_stroke: PrimitiveStatisticsCounter::empty_counter(),
            spell: PrimitiveStatisticsCounter::empty_counter(),
            chunk: PrimitiveStatisticsCounter::empty_counter(),
        }
    }

    /// Returns PrimitiveStatisticsCounter for key stroke.
    pub(crate) fn key_stroke_statistics_counter(&self) -> &PrimitiveStatisticsCounter {
        &self.key_stroke
    }

    /// Returns PrimitiveStatisticsCounter for ideal key stroke.
    pub(crate) fn ideal_key_stroke_statistics_counter(&self) -> &PrimitiveStatisticsCounter {
        &self.ideal_key_stroke
    }

    /// Returns PrimitiveStatisticsCounter for spell.
    pub(crate) fn spell_statistics_counter(&self) -> &PrimitiveStatisticsCounter {
        &self.spell
    }

    #[cfg(test)]
    /// Returns PrimitiveStatisticsCounter for chunk.
    pub(crate) fn chunk_statistics_counter(&self) -> &PrimitiveStatisticsCounter {
        &self.chunk
    }

    /// Consume event and update statistics.
    pub(crate) fn consume_event(&mut self, event: statistical_event::StatisticalEvent) {
        match event {
            StatisticalEvent::KeyStrokeCorrect(key_stroke_correct_context) => {
                let wrong_key_strokes_count = key_stroke_correct_context.wrong_key_strokes().len();

                self.key_stroke.on_target_add(1);
                self.key_stroke.on_finished(1, wrong_key_strokes_count == 0);
                self.key_stroke.on_wrong(wrong_key_strokes_count);
                self.ideal_key_stroke.on_wrong(wrong_key_strokes_count);

                self.chunk.on_wrong(wrong_key_strokes_count);
            }
            StatisticalEvent::SpellFinished(spell_finished_context)
            | StatisticalEvent::SpellDeemedFinished(spell_finished_context) => {
                let spell_count = spell_finished_context.spell().count();
                let wrong_key_stroke_count = spell_finished_context.wrong_key_stroke_count();

                self.spell.on_wrong(spell_count * wrong_key_stroke_count);
                self.spell
                    .on_finished(spell_count, wrong_key_stroke_count == 0);
            }
            StatisticalEvent::ChunkConfirmed(chunk_confirmed_context) => {
                self.chunk
                    .on_finished(1, chunk_confirmed_context.completely_correct());

                chunk_confirmed_context
                    .wrong_key_stroke_count_of_ideal_key_stroke_index()
                    .iter()
                    .for_each(|count| {
                        self.ideal_key_stroke.on_finished(1, *count == 0);
                    });
            }
            StatisticalEvent::ChunkAdded(chunk_added_context) => {
                self.chunk.on_target_add(1);
                self.spell.on_target_add(chunk_added_context.spell_count());
                self.ideal_key_stroke.on_target_add(
                    chunk_added_context
                        .ideal_key_stroke_element_count()
                        .whole_count(),
                );
            }
            StatisticalEvent::KeyStrokeSnapshotted(key_stroke_snapshotted_context) => {
                if let Some(wrong_key_strokes) = key_stroke_snapshotted_context.wrong_key_strokes()
                {
                    let wrong_key_strokes_count = wrong_key_strokes.len();
                    self.key_stroke.on_wrong(wrong_key_strokes_count);
                    self.ideal_key_stroke.on_wrong(wrong_key_strokes_count);
                }

                self.key_stroke.on_target_add(1);
            }
            StatisticalEvent::InflightSpellSnapshotted(inflight_spell_snapshotted_context) => {
                let spell_count = inflight_spell_snapshotted_context.spell().count();
                let wrong_key_stroke_count =
                    inflight_spell_snapshotted_context.wrong_key_strokes().len();

                self.spell.on_wrong(spell_count * wrong_key_stroke_count);
            }
            StatisticalEvent::IdealKeyStrokeDeemedFinished(
                ideal_key_stroke_deemed_finished_context,
            ) => {
                self.ideal_key_stroke.on_finished(
                    1,
                    ideal_key_stroke_deemed_finished_context.wrong_key_stroke_count() == 0,
                );
            }
        }
    }

    /// Construct [`TypingResultSummary`](TypingResultSummary) from holding information.
    /// This method assume that typing is done and updating statistics is all done
    pub(crate) fn construct_typing_result_summary(&self) -> TypingResultSummary {
        TypingResultSummary::new(
            (&self.key_stroke).into(),
            (&self.ideal_key_stroke).into(),
            (&self.spell).into(),
            (&self.chunk).into(),
        )
    }
}

#[cfg(test)]
mod test {
    use statistical_event::IdealKeyStrokeDeemedFinishedContext;
    use statistical_event::InflightSpellSnapshottedContext;
    use statistical_event::KeyStrokeSnapshottedContext;

    use crate::statistics::statistical_event::ChunkConfirmedContext;
    use crate::statistics::statistical_event::SpellFinishedContext;
    use crate::statistics::statistical_event::{
        ChunkAddedContext, KeyStrokeCorrectContext, StatisticalEvent,
    };
    use crate::statistics::PrimitiveStatisticsCounter;
    use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;
    use crate::typing_primitive_types::chunk::ChunkSpell;

    use super::*;

    #[test]
    fn spell_cursor_position_of_display_string_builder_without_inflight_spell_snapshotted_event_is_last_position_plus_one(
    ) {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
            ChunkSpell::new("きょ".to_string().try_into().unwrap()),
            KeyStrokeElementCount::Sigle(3),
        ));
        display_string_builder.consume_event(event);

        assert_eq!(
            display_string_builder.spell().cursor_position(),
            SpellCursorPosition::Single(2)
        );
    }

    #[test]
    fn consume_chunk_added_event_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
            ChunkSpell::new("きょ".to_string().try_into().unwrap()),
            KeyStrokeElementCount::Sigle(3),
        ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 1, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 2, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 3, 0, 0)
        );
    }

    #[test]
    fn consume_chunk_added_event_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
            ChunkSpell::new("きょ".to_string().try_into().unwrap()),
            KeyStrokeElementCount::Sigle(3),
        ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().spell(), "きょ");
        assert_eq!(display_string_builder.spell().last_position(), 1);
    }

    #[test]
    fn consume_key_stroke_correct_event_without_wrong_stroke_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
            'u'.try_into().unwrap(),
            vec![],
        ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 1, 1, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_key_stroke_correct_event_with_wrong_stroke_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
            'u'.try_into().unwrap(),
            vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
        ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 2)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 1, 0, 2)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 2)
        );
    }

    #[test]
    fn consume_key_stroke_correct_event_without_wrong_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
            'u'.try_into().unwrap(),
            vec![],
        ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.key_stroke().key_stroke(), "u");
        assert_eq!(display_string_builder.key_stroke().cursor_position(), 1);
    }

    #[test]
    fn consume_key_stroke_correct_event_with_wrong_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
            'u'.try_into().unwrap(),
            vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
        ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.key_stroke().key_stroke(), "u");
        assert_eq!(display_string_builder.key_stroke().cursor_position(), 1);
        assert_eq!(display_string_builder.key_stroke().wrong_positions(), &[0]);
    }

    #[test]
    fn consume_spell_finished_event_without_wrong_key_stroke_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::SpellFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            0,
        ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 1, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_spell_finished_event_with_wrong_key_stroke_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::SpellFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            2,
        ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 0, 2)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_spell_finished_event_without_wrong_key_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::SpellFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            0,
        ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[]);
    }

    #[test]
    fn consume_spell_finished_event_with_wrong_key_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::SpellFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            0,
        ));
        display_string_builder.consume_event(event);
        let event = StatisticalEvent::SpellFinished(SpellFinishedContext::new(
            ChunkSpell::new("きょ".to_string().try_into().unwrap()),
            2,
        ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[1, 2]);
    }

    #[test]
    fn consume_spell_deemed_finished_event_without_wrong_key_stroke_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::SpellDeemedFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            0,
        ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 1, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_spell_deemed_finished_event_with_wrong_key_stroke_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::SpellDeemedFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            2,
        ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 0, 2)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_spell_deemed_finished_event_without_wrong_key_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::SpellDeemedFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            0,
        ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[]);
    }

    #[test]
    fn consume_spell_deemed_finished_event_with_wrong_key_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::SpellDeemedFinished(SpellFinishedContext::new(
            ChunkSpell::new("う".to_string().try_into().unwrap()),
            0,
        ));
        display_string_builder.consume_event(event);
        let event = StatisticalEvent::SpellFinished(SpellFinishedContext::new(
            ChunkSpell::new("きょ".to_string().try_into().unwrap()),
            2,
        ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[1, 2]);
    }

    #[test]
    fn consume_chunk_confirmed_event_without_wrong_key_stroke() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0]));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 1, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 1, 0)
        );
    }

    #[test]
    fn consume_chunk_confirmed_event_with_wrong_key_stroke() {
        let mut statistics_manager = StatisticsManager::new();
        let event =
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(3, vec![1, 0, 0, 0, 2]));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(3, 0, 1, 0)
        );
    }

    #[test]
    fn consume_key_stroke_snapshotted_event_unstarted_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::KeyStrokeSnapshotted(
            KeyStrokeSnapshottedContext::new_unstarted(&'u'.try_into().unwrap()),
        );

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.key_stroke().key_stroke(), "u");
        assert_eq!(display_string_builder.key_stroke().cursor_position(), 0);
    }

    #[test]
    fn consume_key_stroke_snapshotted_event_started_without_wrong_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::KeyStrokeSnapshotted(
            KeyStrokeSnapshottedContext::new_started(&'u'.try_into().unwrap(), vec![]),
        );

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.key_stroke().key_stroke(), "u");
        assert_eq!(display_string_builder.key_stroke().cursor_position(), 0);
        assert_eq!(display_string_builder.key_stroke().wrong_positions(), &[]);
    }

    #[test]
    fn consume_key_stroke_snapshotted_event_started_with_wrong_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event =
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_started(
                &'u'.try_into().unwrap(),
                vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
            ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.key_stroke().key_stroke(), "u");
        assert_eq!(display_string_builder.key_stroke().cursor_position(), 0);
        assert_eq!(display_string_builder.key_stroke().wrong_positions(), &[0]);
    }

    #[test]
    fn consume_key_stroke_snapshotted_event_unstarted_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::KeyStrokeSnapshotted(
            KeyStrokeSnapshottedContext::new_unstarted(&'u'.try_into().unwrap()),
        );

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 1, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_key_stroke_snapshotted_event_started_without_wrong_stroke_update_statistics_manager()
    {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::KeyStrokeSnapshotted(
            KeyStrokeSnapshottedContext::new_started(&'u'.try_into().unwrap(), vec![]),
        );

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 1, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_key_stroke_snapshotted_event_started_with_wrong_stroke_update_statistics_manager() {
        let mut statistics_manager = StatisticsManager::new();
        let event =
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_started(
                &'u'.try_into().unwrap(),
                vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
            ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 1, 0, 2)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 2)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_without_wrong_stroke_with_single_spell_update_display_string(
    ) {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("ょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleSecond,
                vec![],
            ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[]);
        assert_eq!(
            display_string_builder.spell().cursor_position(),
            SpellCursorPosition::Single(0)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_without_wrong_stroke_with_double_spell_update_display_string(
    ) {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleCombined,
                vec![],
            ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[]);
        assert_eq!(
            display_string_builder.spell().cursor_position(),
            SpellCursorPosition::Double(0, 1)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_with_wrong_stroke_with_single_spell_update_display_string(
    ) {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("あ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::Single,
                vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
            ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[0]);
        assert_eq!(
            display_string_builder.spell().cursor_position(),
            SpellCursorPosition::Single(0)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_with_wrong_stroke_with_double_spell_update_display_string(
    ) {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleCombined,
                vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
            ));

        display_string_builder.consume_event(event);

        assert_eq!(display_string_builder.spell().wrong_positions(), &[0, 1]);
        assert_eq!(
            display_string_builder.spell().cursor_position(),
            SpellCursorPosition::Double(0, 1)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_without_wrong_stroke_with_single_spell_update_statistics_manager(
    ) {
        let mut statistics_manager = StatisticsManager::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("ょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleSecond,
                vec![],
            ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_without_wrong_stroke_with_double_spell_update_statistics_manager(
    ) {
        let mut statistics_manager = StatisticsManager::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleCombined,
                vec![],
            ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_with_wrong_stroke_with_single_spell_update_statistics_manager(
    ) {
        let mut statistics_manager = StatisticsManager::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("あ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::Single,
                vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
            ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 2)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_inflight_spell_snapshotted_event_with_wrong_stroke_with_double_spell_update_statistics_manager(
    ) {
        let mut statistics_manager = StatisticsManager::new();
        let event =
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleCombined,
                vec!['y'.try_into().unwrap(), 'i'.try_into().unwrap()],
            ));

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 4)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
    }

    #[test]
    fn consume_spell_deemed_finished_event_without_wrong_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::SpellDeemedFinished(SpellFinishedContext::new(
            ChunkSpell::new("きょ".to_string().try_into().unwrap()),
            0,
        ));

        display_string_builder.consume_event(event);

        // SpellDeemedFinished event solely does not update public method result.
        display_string_builder.consume_event(StatisticalEvent::InflightSpellSnapshotted(
            InflightSpellSnapshottedContext::new(
                ChunkSpell::new("あ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::Single,
                vec![],
            ),
        ));
        assert_eq!(
            display_string_builder.spell().cursor_position(),
            SpellCursorPosition::Single(2)
        );
    }

    #[test]
    fn consume_spell_deemed_finished_event_with_wrong_stroke_update_display_string() {
        let mut display_string_builder = DisplayStringBuilder::new();
        let event = StatisticalEvent::SpellDeemedFinished(SpellFinishedContext::new(
            ChunkSpell::new("きょ".to_string().try_into().unwrap()),
            2,
        ));

        display_string_builder.consume_event(event);

        // SpellDeemedFinished event solely does not update public method result.
        display_string_builder.consume_event(StatisticalEvent::InflightSpellSnapshotted(
            InflightSpellSnapshottedContext::new(
                ChunkSpell::new("あ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::Single,
                vec![],
            ),
        ));
        assert_eq!(
            display_string_builder.spell().cursor_position(),
            SpellCursorPosition::Single(2)
        );
        assert_eq!(display_string_builder.spell().wrong_positions(), &[0, 1]);
    }

    #[test]
    fn consume_ideal_key_stroke_deemed_finished_event_without_wrong_stroke_update_statistics_manager(
    ) {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::IdealKeyStrokeDeemedFinished(
            IdealKeyStrokeDeemedFinishedContext::new(0),
        );

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 1, 0)
        );
    }

    #[test]
    fn consume_ideal_key_stroke_deemed_finished_event_with_wrong_stroke_update_statistics_manager()
    {
        let mut statistics_manager = StatisticsManager::new();
        let event = StatisticalEvent::IdealKeyStrokeDeemedFinished(
            IdealKeyStrokeDeemedFinishedContext::new(2),
        );

        statistics_manager.consume_event(event);

        assert_eq!(
            statistics_manager.chunk_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.spell_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(0, 0, 0, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke_statistics_counter(),
            &PrimitiveStatisticsCounter::new(1, 0, 0, 0)
        );
    }
}
