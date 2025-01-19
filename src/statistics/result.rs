use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::statistics::OnTypingStatisticsManager;
use crate::typing_primitive_types::chunk::confirmed::ConfirmedChunk;
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::LapRequest;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    confirmed_chunks: &[ConfirmedChunk],
    lap_request: LapRequest,
) -> TypingResultStatistics {
    assert!(!confirmed_chunks.is_empty());

    let mut on_typing_stat_manager = OnTypingStatisticsManager::new(lap_request);

    confirmed_chunks.iter().for_each(|confirmed_chunk| {
        on_typing_stat_manager.set_this_candidate_key_stroke_count(
            confirmed_chunk
                .confirmed_candidate()
                .whole_key_stroke()
                .chars()
                .count(),
            confirmed_chunk
                .as_ref()
                .ideal_key_stroke_candidate()
                .as_ref()
                .unwrap()
                .whole_key_stroke()
                .chars()
                .count(),
        );

        confirmed_chunk
            .actual_key_strokes()
            .iter()
            .zip(confirmed_chunk.construct_spell_end_vector().iter())
            .for_each(|(actual_key_stroke, spell_end)| {
                on_typing_stat_manager.on_actual_key_stroke(
                    actual_key_stroke.is_correct(),
                    confirmed_chunk.effective_spell_count(),
                    *actual_key_stroke.elapsed_time(),
                );

                if actual_key_stroke.is_correct() {
                    if let Some(delta) = spell_end {
                        on_typing_stat_manager.finish_spell(*delta);
                    }
                }
            });

        on_typing_stat_manager.finish_chunk(
            confirmed_chunk
                .as_ref()
                .min_candidate(None)
                .construct_key_stroke_element_count(),
            confirmed_chunk
                .as_ref()
                .ideal_key_stroke_candidate()
                .as_ref()
                .unwrap()
                .construct_key_stroke_element_count(),
            confirmed_chunk.as_ref().spell().count(),
        );
    });

    let total_time = *(confirmed_chunks
        .last()
        .unwrap()
        .actual_key_strokes()
        .last()
        .unwrap()
        .elapsed_time());

    let (key_stroke_ots, ideal_key_stroke_ots, _, _) = on_typing_stat_manager.emit();

    TypingResultStatistics {
        key_stroke: TypingResultStatisticsTarget {
            whole_count: key_stroke_ots.whole_count(),
            completely_correct_count: key_stroke_ots.completely_correct_count(),
            missed_count: key_stroke_ots.wrong_count,
        },
        ideal_key_stroke: TypingResultStatisticsTarget {
            whole_count: ideal_key_stroke_ots.whole_count(),
            completely_correct_count: ideal_key_stroke_ots.completely_correct_count(),
            missed_count: ideal_key_stroke_ots.wrong_count(),
        },
        total_time,
    }
}
