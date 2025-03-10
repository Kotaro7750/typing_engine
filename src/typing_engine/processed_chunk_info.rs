use std::collections::VecDeque;
use std::time::Duration;

use crate::display_info::{KeyStrokeDisplayInfo, SpellDisplayInfo};
use crate::statistics::lap_statistics::LapStatiticsBuilder;
use crate::statistics::statistical_event::{KeyStrokeCorrectContext, StatisticalEvent};
use crate::statistics::statistics_counter::{PrimitiveStatisticsCounter, StatisticsCounter};
use crate::statistics::{construct_on_typing_statistics_target, LapRequest};
use crate::typing_primitive_types::chunk::confirmed::ChunkConfirmed;
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::typing_primitive_types::chunk::inflight::{ChunkInflight, KeyStrokeResult};
use crate::typing_primitive_types::chunk::unprocessed::ChunkUnprocessed;
use crate::typing_primitive_types::chunk::Chunk;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
use crate::typing_primitive_types::key_stroke::KeyStrokeHitMiss;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ProcessedChunkInfo {
    unprocessed_chunks: VecDeque<ChunkUnprocessed>,
    inflight_chunk: Option<ChunkInflight>,
    confirmed_chunks: Vec<ChunkConfirmed>,
}

impl ProcessedChunkInfo {
    #[must_use]
    pub(crate) fn new(chunks: Vec<ChunkUnprocessed>) -> (Self, Vec<StatisticalEvent>) {
        let chunk_added_events = chunks
            .iter()
            .map(StatisticalEvent::new_from_added_chunk)
            .collect();

        (
            Self {
                unprocessed_chunks: chunks.into(),
                inflight_chunk: None,
                confirmed_chunks: vec![],
            },
            chunk_added_events,
        )
    }

    pub(crate) fn is_finished(&self) -> bool {
        // 処理すべきチャンクがない場合には終了である
        self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none()
    }

    #[must_use]
    pub(crate) fn append_chunks(&mut self, chunks: Vec<ChunkUnprocessed>) -> Vec<StatisticalEvent> {
        let chunk_added_events = chunks
            .iter()
            .map(StatisticalEvent::new_from_added_chunk)
            .collect();

        let mut chunks: VecDeque<ChunkUnprocessed> = chunks.into();

        // 終了している状態で追加されたら先頭のチャンクを処理中にする必要がある
        if self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none() {
            let next_inflight_chunk = chunks.pop_front().unwrap();
            self.inflight_chunk
                .replace(next_inflight_chunk.into_inflight(None));
        }

        self.unprocessed_chunks.append(&mut chunks);

        chunk_added_events
    }

    // 現在打っているチャンクを確定させ未処理のチャンク列の先頭のチャンクの処理を開始する
    pub(crate) fn move_next_chunk(&mut self) -> Option<&ChunkConfirmed> {
        let mut confirmed_chunk: Option<&ChunkConfirmed> = None;

        // まずは現在打っているチャンクを確定済みチャンク列に追加する
        let next_chunk_head_constraint = if self.inflight_chunk.is_some() {
            let mut current_inflight_chunk = self.inflight_chunk.take().unwrap();
            assert!(current_inflight_chunk.is_confirmed());

            let confirming_chunk = current_inflight_chunk.try_into_confirmed().unwrap();
            let next_chunk_head_constraint = confirming_chunk
                .confirmed_key_stroke_candidates()
                .next_chunk_head_constraint()
                .clone();

            self.confirmed_chunks.push(confirming_chunk);

            confirmed_chunk.replace(self.confirmed_chunks.last().unwrap());

            next_chunk_head_constraint
        } else {
            None
        };

        assert!(self.inflight_chunk.is_none());

        // 未処理チャンク列の先頭チャンクを処理中のチャンクにする
        if let Some(next_inflight_chunk) = self.unprocessed_chunks.pop_front() {
            self.inflight_chunk
                .replace(next_inflight_chunk.into_inflight(next_chunk_head_constraint));
        }

        confirmed_chunk
    }

    /// 遅延確定候補のために保持しているキーストロークの中にミスタイプがあるかどうか
    fn has_wrong_stroke_in_pending_key_strokes(&self) -> bool {
        self.inflight_chunk.as_ref().is_some_and(|inflight_chunk| {
            inflight_chunk.wrong_key_stroke_count_in_pending_key_strokes() != 0
        })
    }

    // 1タイプのキーストロークを与える
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> (KeyStrokeHitMiss, Vec<StatisticalEvent>) {
        assert!(self.inflight_chunk.is_some());

        let mut statistical_events = vec![];

        let mut original_result: Option<KeyStrokeResult> = None;
        let mut key_strokes = vec![(key_stroke, elapsed_time)];

        while !key_strokes.is_empty() {
            let mut next_key_strokes = vec![];

            key_strokes
                .iter()
                .for_each(|(key_stroke_char, elapsed_time)| {
                    let inflight_chunk = self.inflight_chunk.as_mut().unwrap();

                    let result = inflight_chunk.stroke_key(key_stroke_char.clone(), *elapsed_time);

                    // この関数の引数に与えられたキーストロークの結果を返す必要がある
                    if original_result.is_none() {
                        original_result.replace(result.clone());
                    }

                    if let KeyStrokeResult::Correct(correct_context, _)
                    | KeyStrokeResult::ConfirmDelayed(correct_context) = &result
                    {
                        if let Some(wrong_key_strokes) = result.wrong_key_strokes() {
                            statistical_events.push(StatisticalEvent::new_from_key_stroke_correct(
                                KeyStrokeCorrectContext::new(
                                    key_stroke_char.clone(),
                                    wrong_key_strokes
                                        .iter()
                                        .map(|key_stroke| key_stroke.key_stroke().clone())
                                        .collect(),
                                ),
                            ));
                        }

                        if let Some(spell_finished_context) =
                            correct_context.spell_finished_context()
                        {
                            statistical_events.push(StatisticalEvent::new_from_spell_finished(
                                spell_finished_context.clone(),
                            ))
                        }

                        if let Some(pending_key_strokes) = correct_context.chunk_confirmation() {
                            let confirmed_chunk = self.move_next_chunk().unwrap();
                            statistical_events
                                .push(StatisticalEvent::new_from_confirmed_chunk(confirmed_chunk));

                            if !pending_key_strokes.is_empty() {
                                // 遅延確定候補で確定した場合にはpendingしていたキーストロークを次のチャンクに入力する必要がある
                                pending_key_strokes.iter().for_each(|actual_key_stroke| {
                                    next_key_strokes.push((
                                        actual_key_stroke.key_stroke().clone(),
                                        *actual_key_stroke.elapsed_time(),
                                    ));
                                });
                            }
                        }
                    }
                });

            // pending listには2つ以上の正しいキーストロークが入ることはないので愚直に追加してよい
            key_strokes.clear();
            key_strokes = next_key_strokes.into_iter().collect();
        }

        (original_result.unwrap().into(), statistical_events)
    }

    pub(crate) fn confirmed_chunks(&self) -> &[ChunkConfirmed] {
        &self.confirmed_chunks
    }

    pub(crate) fn construct_display_info(
        &self,
        lap_request: LapRequest,
        confirmed_only_statistics_counter: &StatisticsCounter,
        key_stroke_statistics_counter: &PrimitiveStatisticsCounter,
        spell_statistics_counter: &PrimitiveStatisticsCounter,
    ) -> (SpellDisplayInfo, KeyStrokeDisplayInfo) {
        let mut spell = String::new();
        let mut spell_head_position = 0;
        let mut spell_cursor_positions;
        let mut spell_wrong_positions: Vec<usize> = vec![];

        let mut key_stroke = String::new();
        let mut key_stroke_cursor_position = 0;
        let mut key_stroke_wrong_positions: Vec<usize> = vec![];
        let mut realtime_statistics_counter = confirmed_only_statistics_counter.clone();
        let mut lap_statistics_builder = LapStatiticsBuilder::new(lap_request);
        let mut key_stroke_statistics_counter = key_stroke_statistics_counter.clone();
        let mut spell_statistics_counter = spell_statistics_counter.clone();

        // 1. 確定したチャンク
        // 2. タイプ中のチャンク
        // 3. 未処理のチャンク
        //
        // という順番で表示用の情報を構築する

        // 1. 確定したチャンク
        self.confirmed_chunks.iter().for_each(|confirmed_chunk| {
            lap_statistics_builder.on_add_chunk(
                confirmed_chunk
                    .effective_candidate()
                    .construct_key_stroke_element_count(),
                confirmed_chunk
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count(),
                confirmed_chunk.spell().count(),
            );
            lap_statistics_builder.on_start_chunk(
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

            // Update statistics using actual key strokes
            confirmed_chunk
                .actual_key_strokes()
                .iter()
                .zip(confirmed_chunk.construct_spell_end_vector().iter())
                .for_each(|(actual_key_stroke, spell_end)| {
                    lap_statistics_builder.on_actual_key_stroke(
                        actual_key_stroke.is_correct(),
                        *actual_key_stroke.elapsed_time(),
                    );

                    if actual_key_stroke.is_correct() {
                        if let Some(delta) = spell_end {
                            lap_statistics_builder.on_finish_spell(*delta);
                        }
                    }
                });

            // Update cursor positions and wrong positions for key stroke
            key_stroke_wrong_positions
                .extend(confirmed_chunk.wrong_key_stroke_positions(key_stroke_cursor_position));
            key_stroke_cursor_position += confirmed_chunk
                .confirmed_key_stroke_candidates()
                .calc_key_stroke_count();

            // Update cursor positions and wrong positions for spell
            spell_wrong_positions
                .extend(confirmed_chunk.wrong_spell_positions(spell_head_position));
            spell_head_position += confirmed_chunk.spell().count();

            // 最後にチャンクの統計情報と表示用の文字列を更新する
            key_stroke.push_str(
                &confirmed_chunk
                    .confirmed_key_stroke_candidates()
                    .whole_key_stroke(),
            );
            spell.push_str(confirmed_chunk.spell().as_ref());

            lap_statistics_builder.on_finish_chunk();
        });

        // 2. タイプ中のチャンク

        if self.inflight_chunk.is_none() {
            spell_cursor_positions = vec![spell_head_position];
            assert!(self.is_finished());
        } else {
            // Processing inflight chunk consists of 4 steps.
            // 2-1. Update statistics
            // 2-2. Update cursor and wrong positions of key stroke
            // 2-3. Update cursor and wrong positions of spell
            // 2-4. Update key stroke and spell strings for display

            let inflight_chunk = self.inflight_chunk.as_ref().unwrap();

            // 2-1
            let spell_count = inflight_chunk.effective_spell_count();

            realtime_statistics_counter.on_add_chunk(
                inflight_chunk
                    .min_candidate(None)
                    .construct_key_stroke_element_count(),
                inflight_chunk
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count(),
                inflight_chunk.spell().count(),
            );
            lap_statistics_builder.on_add_chunk(
                inflight_chunk
                    .min_candidate(None)
                    .construct_key_stroke_element_count(),
                inflight_chunk
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count(),
                inflight_chunk.spell().count(),
            );

            realtime_statistics_counter.on_start_chunk(
                inflight_chunk
                    .min_candidate(None)
                    .whole_key_stroke()
                    .chars()
                    .count(),
                inflight_chunk
                    .ideal_key_stroke_candidate()
                    .whole_key_stroke()
                    .chars()
                    .count(),
            );
            lap_statistics_builder.on_start_chunk(
                inflight_chunk
                    .min_candidate(None)
                    .whole_key_stroke()
                    .chars()
                    .count(),
                inflight_chunk
                    .ideal_key_stroke_candidate()
                    .whole_key_stroke()
                    .chars()
                    .count(),
            );

            // Update statistics using actual key strokes
            inflight_chunk
                .actual_key_strokes()
                .iter()
                .zip(inflight_chunk.construct_spell_end_vector().iter())
                .for_each(|(actual_key_stroke, spell_end)| {
                    realtime_statistics_counter
                        .on_stroke_key(actual_key_stroke.is_correct(), spell_count);
                    lap_statistics_builder.on_actual_key_stroke(
                        actual_key_stroke.is_correct(),
                        *actual_key_stroke.elapsed_time(),
                    );

                    if actual_key_stroke.is_correct() {
                        if let Some(delta) = spell_end {
                            realtime_statistics_counter.on_finish_spell(*delta);
                            lap_statistics_builder.on_finish_spell(*delta);
                        }
                    }
                });

            // 2 corrections to key stroke statistics counter are needed.
            // 2-1-1: Correction for unfinished key strokes in inflight chunk.
            // 2-1-2: Correction for wrong key strokes for current (unfinished) key stroke in inflight chunk.

            // 2-1-1
            key_stroke_statistics_counter.on_target_add(
                inflight_chunk.remaining_key_strokes(inflight_chunk.effective_candidate()),
            );
            // 2-1-2
            key_stroke_statistics_counter
                .on_wrong(inflight_chunk.wrong_key_stroke_count_of_current_key_stroke());

            // 2 corrections to spell statistics counter are needed.
            // 2-1-3: Correction for wrong key strokes for unfinished spells in inflight chunk.
            // 2-1-4: Correction regarding as finished for delayed confirmable candidate.

            // 2-1-3
            let current_spell_wrong_count = inflight_chunk.wrong_key_stroke_count_of_element_index(
                inflight_chunk.spell_cursor_position().into(),
            );
            let current_spell_count = inflight_chunk
                .spell()
                .spell_at_index(inflight_chunk.spell_cursor_position().into())
                .count();
            spell_statistics_counter.on_wrong(current_spell_wrong_count * current_spell_count);

            // 2-1-4
            if inflight_chunk
                .delayed_confirmable_candidate_index()
                .is_some()
            {
                spell_statistics_counter
                    .on_finished(current_spell_count, current_spell_wrong_count == 0);
            }

            // 2-2
            key_stroke_wrong_positions
                .extend(inflight_chunk.wrong_key_stroke_positions(key_stroke_cursor_position));
            key_stroke_cursor_position += inflight_chunk.key_stroke_cursor_position(); // この時点ではカーソル位置はこのチャンクの先頭を指しているので単純に足すだけで良い

            // 2-3
            spell_cursor_positions = inflight_chunk
                .spell_cursor_position()
                .into_absolute_cursor_position(spell_head_position);
            spell_wrong_positions.extend(inflight_chunk.wrong_spell_positions(spell_head_position));
            spell_head_position += inflight_chunk.spell().count();

            // 2-4
            key_stroke.push_str(&inflight_chunk.min_candidate(None).whole_key_stroke());
            spell.push_str(inflight_chunk.spell().as_ref());
        }

        // 3. 未処理のチャンク

        let mut next_chunk_head_constraint = if self.inflight_chunk.is_some() {
            self.inflight_chunk
                .as_ref()
                .unwrap()
                .min_candidate(None)
                .next_chunk_head_constraint()
                .clone()
        } else {
            None
        };

        self.unprocessed_chunks
            .iter()
            .enumerate()
            .for_each(|(i, unprocessed_chunk)| {
                // 未処理のチャンクの内最初のチャンクのみタイピング中のチャンクの遅延確定候補の補正をする

                let candidate = unprocessed_chunk.min_candidate(next_chunk_head_constraint.clone());
                let spell_count = if candidate.key_stroke().is_double_splitted() {
                    2
                } else {
                    unprocessed_chunk.spell().count()
                };

                // 未確定のチャンクの一番最初のみ現在打っているチャンクの遅延確定候補によって補正をする
                if i == 0 {
                    if let Some(inflight_chunk) = self.inflight_chunk.as_ref() {
                        if inflight_chunk
                            .delayed_confirmable_candidate_index()
                            .is_some()
                        {
                            // キーストロークのカーソル位置は特に何も処理しなくて良い
                            // 遅延確定候補の候補内カーソル位置は次のチャンク先頭を指す位置にあるため
                            //
                            // 綴りのカーソルは次のチャンク先頭を指す
                            spell_cursor_positions.clear();
                            for i in 0..spell_count {
                                spell_cursor_positions.push(spell_head_position + i);
                            }

                            // When inflight chunk is delayed confirmable, wrong key strokes in
                            // pending key strokes are treated as wrong strokes in next chunk.
                            key_stroke_statistics_counter.on_wrong(
                                inflight_chunk.wrong_key_stroke_count_in_pending_key_strokes(),
                            );
                            spell_statistics_counter.on_wrong(
                                spell_count
                                    * inflight_chunk
                                        .wrong_key_stroke_count_in_pending_key_strokes(),
                            );

                            // 保留中のミスタイプは次のチャンクのミスタイプとみなす
                            if self.has_wrong_stroke_in_pending_key_strokes() {
                                key_stroke_wrong_positions.push(key_stroke_cursor_position);
                                for i in 0..spell_count {
                                    spell_wrong_positions.push(spell_head_position + i);
                                }
                            }

                            inflight_chunk.pending_key_strokes().iter().for_each(
                                |actual_key_stroke| {
                                    realtime_statistics_counter
                                        .on_stroke_key(actual_key_stroke.is_correct(), spell_count);
                                    lap_statistics_builder.on_actual_key_stroke(
                                        actual_key_stroke.is_correct(),
                                        *actual_key_stroke.elapsed_time(),
                                    );
                                },
                            );
                        }
                    }
                }

                // 表示用の文字列を更新する
                key_stroke.push_str(&candidate.whole_key_stroke());

                spell.push_str(unprocessed_chunk.spell().as_ref());
                spell_head_position += unprocessed_chunk.spell().count();

                // チャンクの統計情報を更新する
                key_stroke_statistics_counter.on_target_add(candidate.calc_key_stroke_count());

                let key_stroke_element_count = unprocessed_chunk
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count();

                realtime_statistics_counter.on_add_chunk(
                    key_stroke_element_count.clone(),
                    key_stroke_element_count.clone(),
                    unprocessed_chunk.spell().count(),
                );
                lap_statistics_builder.on_add_chunk(
                    key_stroke_element_count.clone(),
                    key_stroke_element_count,
                    unprocessed_chunk.spell().count(),
                );

                // 次のチャンクへの制限を更新
                match candidate.next_chunk_head_constraint().clone() {
                    Some(constraint) => next_chunk_head_constraint.replace(constraint),
                    None => next_chunk_head_constraint.take(),
                };
            });

        let (_, ideal_key_stroke_statistics_counter, _, _) = realtime_statistics_counter.emit();
        let (
            key_stroke_lap_statistics_builder,
            ideal_key_stroke_lap_statistics_builder,
            spell_lap_statistics_builder,
            _,
        ) = lap_statistics_builder.emit();

        (
            SpellDisplayInfo::new(
                spell,
                spell_cursor_positions,
                spell_wrong_positions,
                spell_head_position - 1,
                construct_on_typing_statistics_target(
                    &spell_statistics_counter,
                    &spell_lap_statistics_builder,
                ),
            ),
            KeyStrokeDisplayInfo::new(
                key_stroke,
                key_stroke_cursor_position,
                key_stroke_wrong_positions,
                construct_on_typing_statistics_target(
                    &key_stroke_statistics_counter,
                    &key_stroke_lap_statistics_builder,
                ),
                construct_on_typing_statistics_target(
                    &ideal_key_stroke_statistics_counter,
                    &ideal_key_stroke_lap_statistics_builder,
                ),
            ),
        )
    }
}
