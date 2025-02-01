use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

pub(crate) mod lap_statistics;
mod multi_target_position_convert;
pub(crate) mod result;
pub(crate) mod statistics_counter;

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
