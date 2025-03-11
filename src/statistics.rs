use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

pub(crate) mod lap_statistics;
pub(crate) mod multi_target_position_convert;
pub(crate) mod result;
pub(crate) mod statistical_event;
pub(crate) mod statistics_counter;

use crate::statistics::statistical_event::StatisticalEvent;
use lap_statistics::PrimitiveLapStatisticsBuilder;
use statistics_counter::PrimitiveStatisticsCounter;

use self::multi_target_position_convert::BaseTarget;

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
    pub fn finished_count(&self) -> usize {
        self.finished_count
    }

    /// Get count of whole targets.
    pub fn whole_count(&self) -> usize {
        self.whole_count
    }

    /// Get count of targets that are finished without miss.
    pub fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }

    /// Get count of wrong typed targets.
    /// Multiple miss types in same targets are counted separately.
    pub fn wrong_count(&self) -> usize {
        self.wrong_count
    }

    /// Get lap end time of target.
    /// This returns [`None`](std::option::Option::None) when target is not a target for take laps.
    pub fn lap_end_time(&self) -> Option<&Vec<Duration>> {
        self.lap_end_time.as_ref()
    }

    /// Get lap end positions of target.
    /// Each positions is converted from requested target.
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
            StatisticalEvent::SpellFinished(spell_finished_context) => {
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
        }
    }
}

#[cfg(test)]
mod test {
    use crate::statistics::statistical_event::ChunkConfirmedContext;
    use crate::statistics::statistical_event::SpellFinishedContext;
    use crate::statistics::statistical_event::{
        ChunkAddedContext, KeyStrokeCorrectContext, StatisticalEvent,
    };
    use crate::statistics::PrimitiveStatisticsCounter;
    use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;
    use crate::typing_primitive_types::chunk::ChunkSpell;

    use super::StatisticsManager;

    #[test]
    fn consume_event_1() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_1
        // In short, stroke u -> w -> w -> u for 「うっう」 and append 「う」

        let events = vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2),
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'u'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("う".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'w'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("っ".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'w'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'u'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("う".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(2, vec![0, 0])),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1),
            )),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            statistics_manager.chunk,
            PrimitiveStatisticsCounter::new(3, 4, 3, 0)
        );
        assert_eq!(
            statistics_manager.spell,
            PrimitiveStatisticsCounter::new(3, 4, 3, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke,
            PrimitiveStatisticsCounter::new(4, 4, 4, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke,
            PrimitiveStatisticsCounter::new(4, 5, 4, 0)
        );
    }

    #[test]
    fn consume_event_2() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_2
        // In short, stroke k-> a -> n -> j -> k -> i for 「かんき」

        let events = vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2),
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'a'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("か".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(2, vec![0, 0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'n'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec!['j'.try_into().unwrap()],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'i'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("き".to_string().try_into().unwrap()),
                1,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(2, vec![1, 0])),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            statistics_manager.chunk,
            PrimitiveStatisticsCounter::new(3, 3, 2, 1)
        );
        assert_eq!(
            statistics_manager.spell,
            PrimitiveStatisticsCounter::new(3, 3, 2, 1)
        );
        assert_eq!(
            statistics_manager.key_stroke,
            PrimitiveStatisticsCounter::new(5, 5, 4, 1)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke,
            PrimitiveStatisticsCounter::new(5, 5, 4, 1)
        );
    }

    #[test]
    fn consume_event_3() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_3
        // In short, stroke k-> a -> n -> j -> n -> k -> i for 「かんき」

        let events = vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2),
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'a'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("か".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(2, vec![0, 0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'n'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'n'.try_into().unwrap(),
                vec!['j'.try_into().unwrap()],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                1,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0, 1])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'i'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("き".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(2, vec![0, 0])),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            statistics_manager.chunk,
            PrimitiveStatisticsCounter::new(3, 3, 2, 1)
        );
        assert_eq!(
            statistics_manager.spell,
            PrimitiveStatisticsCounter::new(3, 3, 2, 1)
        );
        assert_eq!(
            statistics_manager.key_stroke,
            PrimitiveStatisticsCounter::new(6, 6, 5, 1)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke,
            PrimitiveStatisticsCounter::new(5, 5, 4, 1)
        );
    }

    #[test]
    fn consume_event_4() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_4
        // In short, stroke n -> p for reduced 「んぴ」

        let events = vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1),
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'n'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'p'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ぴ".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            statistics_manager.chunk,
            PrimitiveStatisticsCounter::new(2, 2, 2, 0)
        );
        assert_eq!(
            statistics_manager.spell,
            PrimitiveStatisticsCounter::new(2, 2, 2, 0)
        );
        assert_eq!(
            statistics_manager.key_stroke,
            PrimitiveStatisticsCounter::new(2, 2, 2, 0)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke,
            PrimitiveStatisticsCounter::new(2, 2, 2, 0)
        );
    }

    #[test]
    fn consume_event_5() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::construst_display_info_1
        // In short, stroke k -> u -> y -> o -> k -> i -> j -> x -> y -> o -> c -> k for reduced 「きょきょきょきょ」

        let events = vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(3),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(3),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(3),
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(2),
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'y'.try_into().unwrap(),
                vec!['u'.try_into().unwrap()],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'o'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                1,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(3, vec![0, 1, 0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'i'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("き".to_string().try_into().unwrap()),
                0,
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'x'.try_into().unwrap(),
                vec!['j'.try_into().unwrap()],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'y'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'o'.try_into().unwrap(),
                vec![],
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ょ".to_string().try_into().unwrap()),
                1,
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(3, vec![0, 0, 1, 0, 0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec!['c'.try_into().unwrap()],
            )),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            statistics_manager.chunk,
            PrimitiveStatisticsCounter::new(2, 4, 0, 3)
        );
        assert_eq!(
            statistics_manager.spell,
            PrimitiveStatisticsCounter::new(4, 8, 1, 3)
        );
        assert_eq!(
            statistics_manager.key_stroke,
            PrimitiveStatisticsCounter::new(9, 9, 6, 3)
        );
        assert_eq!(
            statistics_manager.ideal_key_stroke,
            PrimitiveStatisticsCounter::new(6, 11, 4, 3)
        );
    }
}
