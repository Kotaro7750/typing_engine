use std::time::Duration;

use serde::{Deserialize, Serialize};

use super::statistics_counter::{PrimitiveStatisticsCounter, StatisticsCounter};
use crate::typing_primitive_types::chunk::confirmed::ChunkConfirmed;
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::typing_primitive_types::chunk::Chunk;
use crate::LapRequest;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing result of typing.
pub struct TypingResult {
    /// Duration for whole typing
    total_time: Duration,
    /// Aggregated result of typing
    summary: TypingResultSummary,
}

impl TypingResult {
    pub(crate) fn new(total_time: Duration, summary: TypingResultSummary) -> Self {
        Self {
            total_time,
            summary,
        }
    }

    /// Returns duration for whole typing
    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    /// Returns aggregated result of typing
    pub fn summary(&self) -> &TypingResultSummary {
        &self.summary
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing aggregated result of typing.
/// Aggregation is for 4 entities, key stroke, ideal key stroke, spell, and chunk.
pub struct TypingResultSummary {
    /// Aggregated result for key stroke
    key_stroke: EntitySummaryStatistics,
    /// Aggregated result for ideal key stroke
    ideal_key_stroke: EntitySummaryStatistics,
    /// Aggregated result for spell
    spell: EntitySummaryStatistics,
    /// Aggregated result for chunk
    chunk: EntitySummaryStatistics,
}

impl TypingResultSummary {
    pub(crate) fn new(
        key_stroke: EntitySummaryStatistics,
        ideal_key_stroke: EntitySummaryStatistics,
        spell: EntitySummaryStatistics,
        chunk: EntitySummaryStatistics,
    ) -> Self {
        Self {
            key_stroke,
            ideal_key_stroke,
            spell,
            chunk,
        }
    }

    /// Returns aggregated result for key stroke
    pub fn key_stroke(&self) -> &EntitySummaryStatistics {
        &self.key_stroke
    }

    /// Returns aggregated result for key stroke
    pub fn ideal_key_stroke(&self) -> &EntitySummaryStatistics {
        &self.ideal_key_stroke
    }

    /// Returns aggregated result for key stroke
    pub fn spell(&self) -> &EntitySummaryStatistics {
        &self.spell
    }

    /// Returns aggregated result for key stroke
    pub fn chunk(&self) -> &EntitySummaryStatistics {
        &self.chunk
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing aggregated result of typing for each entities.
pub struct EntitySummaryStatistics {
    /// Count how many entities are target of typing
    whole_count: usize,
    /// Count how many entities are finished typing without wrong types
    completely_correct_count: usize,
    /// Count how many wrong entity is observed
    /// This count includes duplication, so count may be above 1 when typed wrong multiple times
    wrong_count: usize,
}

impl EntitySummaryStatistics {
    /// Returns count how many entities are target of typing
    pub fn whole_count(&self) -> usize {
        self.whole_count
    }

    /// Returns count how many entities are finished typing without wrong types
    pub fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }

    /// Returns count how many wrong entity is observed
    /// This count includes duplication, so count may be above 1 when typed wrong multiple times
    pub fn wrong_count(&self) -> usize {
        self.wrong_count
    }
}

impl From<&PrimitiveStatisticsCounter> for EntitySummaryStatistics {
    fn from(value: &PrimitiveStatisticsCounter) -> Self {
        Self {
            whole_count: value.whole_count(),
            completely_correct_count: value.completely_correct_count(),
            wrong_count: value.wrong_count(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[deprecated(note = "Use `TypingResult` instead.")]
/// An struct representing the result statistics of typing.
pub struct TypingResultStatistics {
    key_stroke: TypingResultStatisticsTarget,
    ideal_key_stroke: TypingResultStatisticsTarget,
    total_time: Duration,
}

impl TypingResultStatistics {
    #[cfg(test)]
    pub(crate) fn new(
        key_stroke: TypingResultStatisticsTarget,
        ideal_key_stroke: TypingResultStatisticsTarget,
        total_time: Duration,
    ) -> Self {
        Self {
            key_stroke,
            ideal_key_stroke,
            total_time,
        }
    }

    pub fn key_stroke(&self) -> &TypingResultStatisticsTarget {
        &self.key_stroke
    }

    pub fn ideal_key_stroke(&self) -> &TypingResultStatisticsTarget {
        &self.ideal_key_stroke
    }

    pub fn total_time(&self) -> Duration {
        self.total_time
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[deprecated(note = "Use `EntitySummaryStatistics` instead.")]
pub struct TypingResultStatisticsTarget {
    whole_count: usize,
    completely_correct_count: usize,
    missed_count: usize,
}

impl TypingResultStatisticsTarget {
    #[cfg(test)]
    pub(crate) fn new(
        whole_count: usize,
        completely_correct_count: usize,
        missed_count: usize,
    ) -> Self {
        Self {
            whole_count,
            completely_correct_count,
            missed_count,
        }
    }

    pub fn whole_count(&self) -> usize {
        self.whole_count
    }

    pub fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }

    pub fn missed_count(&self) -> usize {
        self.missed_count
    }
}

pub(crate) fn construct_result(
    confirmed_chunks: &[ChunkConfirmed],
    _lap_request: LapRequest,
) -> TypingResultStatistics {
    assert!(!confirmed_chunks.is_empty());

    let mut statistics_counter = StatisticsCounter::new();

    confirmed_chunks.iter().for_each(|confirmed_chunk| {
        statistics_counter.on_add_chunk(
            confirmed_chunk
                .effective_candidate()
                .construct_key_stroke_element_count(),
            confirmed_chunk
                .ideal_key_stroke_candidate()
                .construct_key_stroke_element_count(),
            confirmed_chunk.spell().count(),
        );
        statistics_counter.on_start_chunk(
            confirmed_chunk
                .confirmed_key_stroke_candidates()
                .whole_key_stroke()
                .chars()
                .count(),
            confirmed_chunk
                .ideal_key_stroke_candidate()
                .whole_key_stroke()
                .chars()
                .count(),
        );

        confirmed_chunk
            .actual_key_strokes()
            .iter()
            .zip(confirmed_chunk.construct_spell_end_vector().iter())
            .for_each(|(actual_key_stroke, spell_end)| {
                statistics_counter.on_stroke_key(
                    actual_key_stroke.is_correct(),
                    confirmed_chunk.effective_spell_count(),
                );

                if actual_key_stroke.is_correct() {
                    if let Some(delta) = spell_end {
                        statistics_counter.on_finish_spell(*delta);
                    }
                }
            });

        statistics_counter.on_finish_chunk();
    });

    let total_time = *(confirmed_chunks
        .last()
        .unwrap()
        .actual_key_strokes()
        .last()
        .unwrap()
        .elapsed_time());

    let (key_stroke_ots, ideal_key_stroke_ots, _, _) = statistics_counter.emit();

    TypingResultStatistics {
        key_stroke: TypingResultStatisticsTarget {
            whole_count: key_stroke_ots.whole_count(),
            completely_correct_count: key_stroke_ots.completely_correct_count(),
            missed_count: key_stroke_ots.wrong_count(),
        },
        ideal_key_stroke: TypingResultStatisticsTarget {
            whole_count: ideal_key_stroke_ots.whole_count(),
            completely_correct_count: ideal_key_stroke_ots.completely_correct_count(),
            missed_count: ideal_key_stroke_ots.wrong_count(),
        },
        total_time,
    }
}
