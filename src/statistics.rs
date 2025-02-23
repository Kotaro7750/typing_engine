use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

pub(crate) mod lap_statistics;
mod multi_target_position_convert;
pub(crate) mod result;
pub(crate) mod statistical_event;
pub(crate) mod statistics_counter;

use lap_statistics::PrimitiveLapStatisticsBuilder;
use statistics_counter::PrimitiveStatisticsCounter;
use statistics_counter::StatisticsCounter;

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
    confirmed_only_statistics_counter: StatisticsCounter,
}

impl StatisticsManager {
    pub(crate) fn new() -> Self {
        Self {
            confirmed_only_statistics_counter: StatisticsCounter::new(),
        }
    }

    pub(crate) fn confirmed_only_statistics_counter(&self) -> &StatisticsCounter {
        &self.confirmed_only_statistics_counter
    }

    /// Consume event and update statistics.
    pub(crate) fn consume_event(&mut self, event: statistical_event::StatisticalEvent) {
        match event {
            statistical_event::StatisticalEvent::ChunkConfirmed(chunk_confirmation_info) => {
                self.confirmed_only_statistics_counter.on_add_chunk(
                    chunk_confirmation_info.key_stroke_element_count,
                    chunk_confirmation_info.ideal_key_stroke_element_count,
                    chunk_confirmation_info.spell_count,
                );

                self.confirmed_only_statistics_counter.on_start_chunk(
                    chunk_confirmation_info.candidate_key_stroke_count,
                    chunk_confirmation_info.ideal_candidate_key_stroke_count,
                );

                chunk_confirmation_info
                    .actual_key_stroke_info
                    .iter()
                    .for_each(|(is_correct, spell_end)| {
                        self.confirmed_only_statistics_counter.on_stroke_key(
                            *is_correct,
                            chunk_confirmation_info.effective_spell_count,
                        );
                        if *is_correct {
                            if let Some(delta) = spell_end {
                                self.confirmed_only_statistics_counter
                                    .on_finish_spell(*delta);
                            }
                        }
                    });

                self.confirmed_only_statistics_counter.on_finish_chunk();
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::statistics::statistical_event::ChunkConfirmationInfo;
    use crate::statistics::statistical_event::StatisticalEvent;
    use crate::statistics::PrimitiveStatisticsCounter;
    use crate::statistics::StatisticsCounter;
    use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;

    use super::StatisticsManager;

    #[test]
    fn consume_event_1() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_1
        // In short, stroke u -> w -> w -> u for 「うっう」

        let events = vec![
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))],
            )),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            *statistics_manager.confirmed_only_statistics_counter(),
            StatisticsCounter::new_with_values(
                PrimitiveStatisticsCounter::new(4, 4, 4, 0),
                PrimitiveStatisticsCounter::new(4, 4, 4, 0),
                PrimitiveStatisticsCounter::new(3, 3, 3, 0),
                PrimitiveStatisticsCounter::new(3, 3, 3, 0),
                false,
                false,
                false,
                false,
                None,
                None,
                0,
            )
        );
    }

    #[test]
    fn consume_event_2() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_2
        // In short, stroke k-> a -> n -> j -> k -> i for 「かんき」

        let events = vec![
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(false, None), (true, None), (true, Some(1))],
            )),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            *statistics_manager.confirmed_only_statistics_counter(),
            StatisticsCounter::new_with_values(
                PrimitiveStatisticsCounter::new(5, 5, 4, 1),
                PrimitiveStatisticsCounter::new(5, 5, 4, 1),
                PrimitiveStatisticsCounter::new(3, 3, 2, 1),
                PrimitiveStatisticsCounter::new(3, 3, 2, 1),
                false,
                false,
                false,
                false,
                None,
                None,
                0,
            )
        );
    }

    #[test]
    fn consume_event_3() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_3
        // In short, stroke k-> a -> n -> j -> n -> k -> i for 「かんき」

        let events = vec![
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(1),
                1,
                2,
                1,
                1,
                vec![(true, None), (false, None), (true, Some(1))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))],
            )),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            *statistics_manager.confirmed_only_statistics_counter(),
            StatisticsCounter::new_with_values(
                PrimitiveStatisticsCounter::new(6, 6, 5, 1),
                PrimitiveStatisticsCounter::new(5, 5, 4, 1),
                PrimitiveStatisticsCounter::new(3, 3, 2, 1),
                PrimitiveStatisticsCounter::new(3, 3, 2, 1),
                false,
                false,
                false,
                false,
                None,
                None,
                0,
            )
        );
    }

    #[test]
    fn consume_event_4() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::stroke_key_4
        // In short, stroke n -> p for reduced 「んぴ」

        let events = vec![
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))],
            )),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            *statistics_manager.confirmed_only_statistics_counter(),
            StatisticsCounter::new_with_values(
                PrimitiveStatisticsCounter::new(2, 2, 2, 0),
                PrimitiveStatisticsCounter::new(2, 2, 2, 0),
                PrimitiveStatisticsCounter::new(2, 2, 2, 0),
                PrimitiveStatisticsCounter::new(2, 2, 2, 0),
                false,
                false,
                false,
                false,
                None,
                None,
                0
            )
        );
    }

    #[test]
    fn consume_event_5() {
        let mut statistics_manager = StatisticsManager::new();

        // These events are same with typing_engin::processed_chunk_info::test::construst_display_info_1
        // In short, stroke k -> u -> y -> o -> k -> i -> j -> x -> y -> o -> c -> k for reduced 「きょきょきょ」

        let events = vec![
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(3),
                KeyStrokeElementCount::Sigle(3),
                2,
                3,
                3,
                2,
                vec![(true, None), (false, None), (true, None), (true, Some(2))],
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Double((2, 3)),
                KeyStrokeElementCount::Sigle(3),
                2,
                5,
                3,
                1,
                vec![
                    (true, None),
                    (true, Some(1)),
                    (false, None),
                    (true, None),
                    (true, None),
                    (true, Some(1)),
                ],
            )),
        ];

        events.iter().for_each(|statistical_event| {
            statistics_manager.consume_event(statistical_event.clone());
        });

        assert_eq!(
            *statistics_manager.confirmed_only_statistics_counter(),
            StatisticsCounter::new_with_values(
                PrimitiveStatisticsCounter::new(8, 8, 6, 2),
                PrimitiveStatisticsCounter::new(6, 6, 4, 2),
                PrimitiveStatisticsCounter::new(4, 4, 1, 3),
                PrimitiveStatisticsCounter::new(2, 2, 0, 2),
                false,
                false,
                false,
                false,
                None,
                None,
                0
            )
        );
    }
}
