use std::collections::VecDeque;
use std::time::Duration;

use crate::display_info::{KeyStrokeDisplayInfo, SpellDisplayInfo};
use crate::statistics::lap_statistics::LapStatiticsBuilder;
use crate::statistics::statistical_event::StatisticalEvent;
use crate::statistics::statistics_counter::StatisticsCounter;
use crate::statistics::{construct_on_typing_statistics_target, LapRequest};
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::typing_primitive_types::chunk::Chunk;
use crate::typing_primitive_types::chunk::ChunkState;
use crate::typing_primitive_types::key_stroke::KeyStrokeResult;
use crate::typing_primitive_types::key_stroke::{ActualKeyStroke, KeyStrokeChar};

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ProcessedChunkInfo {
    unprocessed_chunks: VecDeque<Chunk>,
    inflight_chunk: Option<Chunk>,
    confirmed_chunks: Vec<Chunk>,
    pending_key_strokes: Vec<ActualKeyStroke>,
}

impl ProcessedChunkInfo {
    #[must_use]
    pub(crate) fn new(chunks: Vec<Chunk>) -> (Self, Vec<StatisticalEvent>) {
        let chunk_added_events = chunks
            .iter()
            .map(StatisticalEvent::new_from_added_chunk)
            .collect();

        (
            Self {
                unprocessed_chunks: chunks.into(),
                inflight_chunk: None,
                confirmed_chunks: vec![],
                pending_key_strokes: vec![],
            },
            chunk_added_events,
        )
    }

    pub(crate) fn is_finished(&self) -> bool {
        // 処理すべきチャンクがない場合には終了である
        self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none()
    }

    #[must_use]
    pub(crate) fn append_chunks(&mut self, chunks: Vec<Chunk>) -> Vec<StatisticalEvent> {
        let chunk_added_events = chunks
            .iter()
            .map(StatisticalEvent::new_from_added_chunk)
            .collect();

        let mut chunks: VecDeque<Chunk> = chunks.into();

        // 終了している状態で追加されたら先頭のチャンクを処理中にする必要がある
        if self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none() {
            let mut next_inflight_chunk = chunks.pop_front().unwrap();
            next_inflight_chunk.change_state(ChunkState::Inflight);
            self.inflight_chunk.replace(next_inflight_chunk);
        }

        self.unprocessed_chunks.append(&mut chunks);

        chunk_added_events
    }

    // 現在打っているチャンクを確定させ未処理のチャンク列の先頭のチャンクの処理を開始する
    pub(crate) fn move_next_chunk(&mut self) -> Option<&Chunk> {
        let mut confirmed_chunk: Option<&Chunk> = None;

        // まずは現在打っているチャンクを確定済みチャンク列に追加する
        let next_chunk_head_constraint = if self.inflight_chunk.is_some() {
            let mut current_inflight_chunk = self.inflight_chunk.take().unwrap();
            assert!(current_inflight_chunk.is_confirmed());
            current_inflight_chunk.change_state(ChunkState::Confirmed);

            let next_chunk_head_constraint = current_inflight_chunk.next_chunk_head_constraint();
            self.confirmed_chunks.push(current_inflight_chunk);

            confirmed_chunk.replace(self.confirmed_chunks.last().unwrap());

            next_chunk_head_constraint
        } else {
            None
        };

        assert!(self.inflight_chunk.is_none());

        // 未処理チャンク列の先頭チャンクを処理中のチャンクにする
        if let Some(mut next_inflight_chunk) = self.unprocessed_chunks.pop_front() {
            if let Some(next_chunk_head_constraint) = next_chunk_head_constraint {
                next_inflight_chunk.strict_chunk_head(next_chunk_head_constraint);
            }

            next_inflight_chunk.change_state(ChunkState::Inflight);
            self.inflight_chunk.replace(next_inflight_chunk);
        }

        confirmed_chunk
    }

    /// Clear and drain all pending key strokes
    fn take_pending_key_strokes(&mut self) -> Vec<ActualKeyStroke> {
        let pending_key_strokes = self.pending_key_strokes.clone();
        self.pending_key_strokes.clear();
        pending_key_strokes
    }

    /// 遅延確定候補のために保持しているキーストロークの中にミスタイプがあるかどうか
    fn has_wrong_stroke_in_pending_key_strokes(&self) -> bool {
        self.pending_key_strokes
            .iter()
            .map(|actual_key_stroke| !actual_key_stroke.is_correct())
            .reduce(|accum, is_correct| accum || is_correct)
            .map_or(false, |r| r)
    }

    // 1タイプのキーストロークを与える
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> (KeyStrokeResult, Vec<StatisticalEvent>) {
        assert!(self.inflight_chunk.is_some());

        let mut statistical_events = vec![];

        let inflight_chunk = self.inflight_chunk.as_mut().unwrap();
        let need_add_to_pending = inflight_chunk.is_delayed_confirmable();
        let result = inflight_chunk.stroke_key(key_stroke.clone(), elapsed_time);

        // Add key stroke to pending list if the chunk is delayed confirmable
        if need_add_to_pending {
            self.pending_key_strokes.push(ActualKeyStroke::new(
                elapsed_time,
                key_stroke,
                matches!(result, KeyStrokeResult::Correct),
            ));
        }

        // このキーストロークでチャンクが確定したら次のチャンクの処理に移る
        // Key strokes in pending list should be added to chunk.
        // What chunk to be added is determined whether it is delayed confirmable or not.
        if inflight_chunk.is_confirmed() {
            let is_delayed_confirmable = inflight_chunk.is_delayed_confirmable();
            let pending_key_strokes = self.take_pending_key_strokes();

            // 遅延確定候補でない場合にはpendingしていたキーストロークを現在のチャンクに入力する
            if !is_delayed_confirmable {
                let inflight_chunk = self.inflight_chunk.as_mut().unwrap();
                pending_key_strokes.iter().for_each(|actual_key_stroke| {
                    inflight_chunk.append_actual_key_stroke(actual_key_stroke.clone());
                });
            }

            let confirmed_chunk = self.move_next_chunk().unwrap();
            statistical_events.push(StatisticalEvent::new_from_confirmed_chunk(confirmed_chunk));

            // 遅延確定候補で確定した場合にはpendingしていたキーストロークを次のチャンクに入力する必要がある
            if is_delayed_confirmable {
                assert!(self.inflight_chunk.is_some());
                let inflight_chunk = self.inflight_chunk.as_mut().unwrap();

                pending_key_strokes.iter().for_each(|actual_key_stroke| {
                    inflight_chunk.stroke_key(
                        actual_key_stroke.key_stroke().clone(),
                        *actual_key_stroke.elapsed_time(),
                    );
                });

                // pendingしていたキーストロークの入力によって次のチャンクが終了する場合に対処する
                if inflight_chunk.is_confirmed() {
                    // pendingしていたキーストローク中には正しいキーストロークは一つしか無いはずなので遅延確定候補が終了することはない
                    assert!(!inflight_chunk.is_delayed_confirmable());

                    let confirmed_chunk = self.move_next_chunk().unwrap();
                    statistical_events
                        .push(StatisticalEvent::new_from_confirmed_chunk(confirmed_chunk));
                }
            }
        }

        (result, statistical_events)
    }

    pub(crate) fn confirmed_chunks(&self) -> &Vec<Chunk> {
        &self.confirmed_chunks
    }

    pub(crate) fn construct_display_info(
        &self,
        lap_request: LapRequest,
        confirmed_only_statistics_counter: &StatisticsCounter,
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

        // 1. 確定したチャンク
        // 2. タイプ中のチャンク
        // 3. 未処理のチャンク
        //
        // という順番で表示用の情報を構築する

        // 1. 確定したチャンク
        self.confirmed_chunks.iter().for_each(|confirmed_chunk| {
            lap_statistics_builder.on_add_chunk(
                confirmed_chunk
                    .as_ref()
                    .min_candidate(None)
                    .construct_key_stroke_element_count(),
                confirmed_chunk
                    .as_ref()
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count(),
                confirmed_chunk.as_ref().spell().count(),
            );
            lap_statistics_builder.on_start_chunk(
                confirmed_chunk
                    .confirmed_candidate()
                    .whole_key_stroke()
                    .chars()
                    .count(),
                confirmed_chunk
                    .as_ref()
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
                .confirmed_candidate()
                .calc_key_stroke_count();

            // Update cursor positions and wrong positions for spell
            spell_wrong_positions
                .extend(confirmed_chunk.wrong_spell_positions(spell_head_position));
            spell_head_position += confirmed_chunk.as_ref().spell().count();

            // 最後にチャンクの統計情報と表示用の文字列を更新する
            key_stroke.push_str(&confirmed_chunk.confirmed_candidate().whole_key_stroke());
            spell.push_str(confirmed_chunk.as_ref().spell().as_ref());

            lap_statistics_builder.on_finish_chunk();
        });

        // 2. タイプ中のチャンク

        if self.inflight_chunk.is_none() {
            spell_cursor_positions = vec![spell_head_position];
            assert!(self.is_finished());
        } else {
            let inflight_chunk = self.inflight_chunk.as_ref().unwrap();

            let spell_count = inflight_chunk.effective_spell_count();

            realtime_statistics_counter.on_add_chunk(
                inflight_chunk
                    .as_ref()
                    .min_candidate(None)
                    .construct_key_stroke_element_count(),
                inflight_chunk
                    .as_ref()
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count(),
                inflight_chunk.as_ref().spell().count(),
            );
            lap_statistics_builder.on_add_chunk(
                inflight_chunk
                    .as_ref()
                    .min_candidate(None)
                    .construct_key_stroke_element_count(),
                inflight_chunk
                    .as_ref()
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count(),
                inflight_chunk.as_ref().spell().count(),
            );

            realtime_statistics_counter.on_start_chunk(
                inflight_chunk
                    .as_ref()
                    .min_candidate(None)
                    .whole_key_stroke()
                    .chars()
                    .count(),
                inflight_chunk
                    .as_ref()
                    .ideal_key_stroke_candidate()
                    .whole_key_stroke()
                    .chars()
                    .count(),
            );
            lap_statistics_builder.on_start_chunk(
                inflight_chunk
                    .as_ref()
                    .min_candidate(None)
                    .whole_key_stroke()
                    .chars()
                    .count(),
                inflight_chunk
                    .as_ref()
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

            // Update cursor positions and wrong positions for key stroke
            key_stroke_wrong_positions
                .extend(inflight_chunk.wrong_key_stroke_positions(key_stroke_cursor_position));
            key_stroke_cursor_position += inflight_chunk.current_key_stroke_cursor_position(); // この時点ではカーソル位置はこのチャンクの先頭を指しているので単純に足すだけで良い

            // Update cursor positions and wrong positions for spell
            spell_cursor_positions = inflight_chunk
                .current_spell_cursor_positions()
                .iter()
                .map(|in_chunk_current_spell_cursor_position| {
                    spell_head_position + in_chunk_current_spell_cursor_position
                })
                .collect();
            spell_wrong_positions.extend(inflight_chunk.wrong_spell_positions(spell_head_position));
            spell_head_position += inflight_chunk.as_ref().spell().count();

            // 最後にチャンクの統計情報と表示用の文字列を更新する

            key_stroke.push_str(
                &inflight_chunk
                    .as_ref()
                    .min_candidate(None)
                    .whole_key_stroke(),
            );
            spell.push_str(inflight_chunk.as_ref().spell().as_ref());
        }

        // 3. 未処理のチャンク

        let mut next_chunk_head_constraint = if self.inflight_chunk.is_some() {
            self.inflight_chunk
                .as_ref()
                .unwrap()
                .as_ref()
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
                let spell_count = if candidate.is_splitted() {
                    2
                } else {
                    unprocessed_chunk.spell().count()
                };

                // 未確定のチャンクの一番最初のみ現在打っているチャンクの遅延確定候補によって補正をする
                if i == 0 {
                    if let Some(inflight_chunk) = self.inflight_chunk.as_ref() {
                        if inflight_chunk.is_delayed_confirmable() {
                            // キーストロークのカーソル位置は特に何も処理しなくて良い
                            // 遅延確定候補の候補内カーソル位置は次のチャンク先頭を指す位置にあるため
                            //
                            // 綴りのカーソルは次のチャンク先頭を指す
                            spell_cursor_positions.clear();
                            for i in 0..spell_count {
                                spell_cursor_positions.push(spell_head_position + i);
                            }

                            // 保留中のミスタイプは次のチャンクのミスタイプとみなす
                            if self.has_wrong_stroke_in_pending_key_strokes() {
                                key_stroke_wrong_positions.push(key_stroke_cursor_position);
                                for i in 0..spell_count {
                                    spell_wrong_positions.push(spell_head_position + i);
                                }
                            }

                            self.pending_key_strokes
                                .iter()
                                .for_each(|actual_key_stroke| {
                                    realtime_statistics_counter
                                        .on_stroke_key(actual_key_stroke.is_correct(), spell_count);
                                    lap_statistics_builder.on_actual_key_stroke(
                                        actual_key_stroke.is_correct(),
                                        *actual_key_stroke.elapsed_time(),
                                    );
                                });
                        }
                    }
                }

                // 表示用の文字列を更新する
                key_stroke.push_str(&candidate.whole_key_stroke());

                spell.push_str(unprocessed_chunk.spell().as_ref());
                spell_head_position += unprocessed_chunk.spell().count();

                // チャンクの統計情報を更新する

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

        let (
            key_stroke_statistics_counter,
            ideal_key_stroke_statistics_counter,
            spell_statistics_counter,
            _,
        ) = realtime_statistics_counter.emit();
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
