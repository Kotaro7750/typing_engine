use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::statistics::OnTypingStatisticsManager;
use crate::typing_primitive_types::chunk::confirmed::ConfirmedChunk;
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::LapRequest;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TypingResultStatistics {
    key_stroke: TypingResultStatisticsTarget,
    ideal_key_stroke: TypingResultStatisticsTarget,
    total_time: Duration,
}

impl TypingResultStatistics {
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

    let mut spell = String::new();
    let mut spell_head_position = 0;
    let mut spell_wrong_positions: Vec<usize> = vec![];

    let mut key_stroke = String::new();
    let mut key_stroke_cursor_position = 0;
    let mut key_stroke_wrong_positions: Vec<usize> = vec![];
    let mut on_typing_stat_manager = OnTypingStatisticsManager::new(lap_request);

    confirmed_chunks.iter().for_each(|confirmed_chunk| {
        let mut in_candidate_cursor_position = 0;
        let mut wrong_spell_element_vector = confirmed_chunk.initialized_spell_element_vector();
        let mut wrong_key_strokes_vector = confirmed_chunk.initialized_key_strokes_vector();
        // 複数文字の綴りをまとめて打つ場合には綴りの統計は2文字分カウントする必要がある
        let spell_count = confirmed_chunk.effective_spell_count();

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

        // まず実際のキーストローク系列から統計情報を更新しチャンク内ミス位置を構築する

        confirmed_chunk
            .actual_key_strokes()
            .iter()
            .zip(confirmed_chunk.construct_spell_end_vector().iter())
            .for_each(|(actual_key_stroke, spell_end)| {
                on_typing_stat_manager.on_actual_key_stroke(
                    actual_key_stroke.is_correct(),
                    spell_count,
                    *actual_key_stroke.elapsed_time(),
                );

                if actual_key_stroke.is_correct() {
                    in_candidate_cursor_position += 1;

                    if let Some(delta) = spell_end {
                        on_typing_stat_manager.finish_spell(*delta);
                    }
                } else {
                    wrong_key_strokes_vector[in_candidate_cursor_position] = true;

                    wrong_spell_element_vector[confirmed_chunk
                        .confirmed_candidate()
                        .belonging_element_index_of_key_stroke(in_candidate_cursor_position)] =
                        true;
                }
            });

        // 次に構築したチャンク内ミス位置からキーストロークと綴りのそれぞれのカーソル位置とミス位置を更新する

        wrong_key_strokes_vector
            .iter()
            .enumerate()
            .for_each(|(i, is_wrong)| {
                if *is_wrong {
                    key_stroke_wrong_positions.push(key_stroke_cursor_position + i);
                }
            });
        key_stroke_cursor_position += in_candidate_cursor_position;

        confirmed_chunk
            .as_ref()
            .spell()
            .as_ref()
            .chars()
            .enumerate()
            .for_each(|(i, _)| {
                // 複数文字チャンクを個別に入力した場合はそれぞれの綴りについて
                // それ以外ではチャンク全体の綴りについて
                // タイプミス判定をする
                let element_index = if wrong_spell_element_vector.len() == 1 {
                    0
                } else {
                    i
                };

                if wrong_spell_element_vector[element_index] {
                    spell_wrong_positions.push(spell_head_position);
                }

                spell_head_position += 1;
            });

        // 最後にチャンクの統計情報と表示用の文字列を更新する
        key_stroke.push_str(&confirmed_chunk.confirmed_candidate().whole_key_stroke());
        spell.push_str(confirmed_chunk.as_ref().spell().as_ref());

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

    let (key_stroke_ots, ideal_key_stroke_ots, spell_ots, c_ots) = on_typing_stat_manager.emit();

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
