use std::collections::VecDeque;
use std::time::Duration;

use crate::chunk::confirmed::ConfirmedChunk;
use crate::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::chunk::typed::{KeyStrokeResult, TypedChunk};
use crate::chunk::Chunk;
use crate::display_info::{KeyStrokeDisplayInfo, SpellDisplayInfo};
use crate::key_stroke::KeyStrokeChar;
use crate::statistics::OnTypingStatisticsManager;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ProcessedChunkInfo {
    unprocessed_chunks: VecDeque<Chunk>,
    inflight_chunk: Option<TypedChunk>,
    confirmed_chunks: Vec<ConfirmedChunk>,
}

impl ProcessedChunkInfo {
    pub(crate) fn new(chunks: Vec<Chunk>) -> Self {
        Self {
            unprocessed_chunks: chunks.into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
        }
    }

    pub(crate) fn is_finished(&self) -> bool {
        // 処理すべきチャンクがない場合には終了である
        self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none()
    }

    pub(crate) fn append_chunks(&mut self, chunks: Vec<Chunk>) {
        let mut chunks: VecDeque<Chunk> = chunks.into();

        // 終了している状態で追加されたら先頭のチャンクを処理中にする必要がある
        if self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none() {
            self.inflight_chunk
                .replace(chunks.pop_front().unwrap().into());
        }

        self.unprocessed_chunks.append(&mut chunks);
    }

    // 現在打っているチャンクを確定させ未処理のチャンク列の先頭のチャンクの処理を開始する
    pub(crate) fn move_next_chunk(&mut self) {
        // まずは現在打っているチャンクを確定済みチャンク列に追加する
        let next_chunk_head_constraint = if self.inflight_chunk.is_some() {
            let mut current_inflight_chunk = self.inflight_chunk.take().unwrap();
            assert!(current_inflight_chunk.is_confirmed());

            let mut current_confirmed_chunk: ConfirmedChunk = current_inflight_chunk.into();
            let next_chunk_head_constraint = current_confirmed_chunk.next_chunk_head_constraint();
            self.confirmed_chunks.push(current_confirmed_chunk);

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

            self.inflight_chunk.replace(next_inflight_chunk.into());
        }
    }

    // 1タイプのキーストロークを与える
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(self.inflight_chunk.is_some());

        let inflight_chunk = self.inflight_chunk.as_mut().unwrap();
        let result = inflight_chunk.stroke_key(key_stroke, elapsed_time);

        // このキーストロークでチャンクが確定したら次のチャンクの処理に移る
        if inflight_chunk.is_confirmed() {
            let pending_key_strokes = inflight_chunk.take_pending_key_strokes();
            let is_delayed_confirmable = inflight_chunk.is_delayed_confirmable();

            self.move_next_chunk();

            // 遅延確定候補で確定した場合にはpendingしていたキーストロークを次のチャンクに入力する必要がある
            if is_delayed_confirmable {
                assert!(self.inflight_chunk.is_some());

                pending_key_strokes.iter().for_each(|actual_key_stroke| {
                    self.inflight_chunk.as_mut().unwrap().stroke_key(
                        actual_key_stroke.key_stroke().clone(),
                        actual_key_stroke.elapsed_time().clone(),
                    );
                });
            } else {
                assert!(pending_key_strokes.is_empty());
            }
        }

        result
    }

    pub(crate) fn construct_display_info(&self) -> (SpellDisplayInfo, KeyStrokeDisplayInfo) {
        let mut spell = String::new();
        let mut spell_head_position = 0;
        let mut spell_cursor_positions;
        let mut spell_wrong_positions: Vec<usize> = vec![];

        let mut key_stroke = String::new();
        let mut key_stroke_cursor_position = 0;
        let mut key_stroke_wrong_positions: Vec<usize> = vec![];
        let mut on_typing_stat_manager = OnTypingStatisticsManager::new();

        // 1. 確定したチャンク
        // 2. タイプ中のチャンク
        // 3. 未処理のチャンク
        //
        // という順番で表示用の情報を構築する

        // 1. 確定したチャンク
        self.confirmed_chunks.iter().for_each(|confirmed_chunk| {
            // 複数文字の綴りをまとめて打つ場合には綴りの統計は2文字分カウントする必要がある
            let spell_count = if confirmed_chunk.confirmed_candidate().is_splitted() {
                1
            } else {
                confirmed_chunk.as_ref().spell().count()
            };

            let mut wrong_spell_element_vector = vec![
                false;
                if confirmed_chunk.confirmed_candidate().is_splitted() {
                    2
                } else {
                    1
                }
            ];

            let mut in_candidate_cursor_position = 0;

            confirmed_chunk
                .actual_key_strokes()
                .iter()
                .zip(confirmed_chunk.construct_spell_end_vector().iter())
                .for_each(|(actual_key_stroke, spell_end)| {
                    on_typing_stat_manager
                        .on_actual_key_stroke(actual_key_stroke.is_correct(), spell_count);

                    if actual_key_stroke.is_correct() {
                        in_candidate_cursor_position += 1;

                        if let Some(delta) = spell_end {
                            //spell_head_position += *delta;
                            on_typing_stat_manager.finish_spell(*delta);
                        }
                    } else {
                        if key_stroke_wrong_positions.last().map_or(true, |e| {
                            *e != (key_stroke_cursor_position + in_candidate_cursor_position)
                        }) {
                            key_stroke_wrong_positions
                                .push(key_stroke_cursor_position + in_candidate_cursor_position);
                        }

                        wrong_spell_element_vector[confirmed_chunk
                            .confirmed_candidate()
                            .element_index_at_key_stroke_index(in_candidate_cursor_position)] =
                            true;
                    }
                });

            key_stroke_cursor_position += in_candidate_cursor_position;

            // 複数文字チャンクを個別に入力した場合はそれぞれの綴りについて
            // それ以外ではチャンク全体の綴りについて
            // タイプミス判定をする
            confirmed_chunk
                .as_ref()
                .spell()
                .as_ref()
                .chars()
                .enumerate()
                .for_each(|(i, _)| {
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

            key_stroke.push_str(&confirmed_chunk.confirmed_candidate().whole_key_stroke());
            spell.push_str(confirmed_chunk.as_ref().spell().as_ref());

            on_typing_stat_manager.finish_chunk(
                confirmed_chunk
                    .as_ref()
                    .min_candidate(None)
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
                confirmed_chunk.as_ref().spell().count(),
            );
        });

        // 2. タイプ中のチャンク

        if self.inflight_chunk.is_none() {
            spell_cursor_positions = vec![spell_head_position];
            assert!(self.is_finished());
        } else {
            let inflight_chunk = self.inflight_chunk.as_ref().unwrap();

            let is_delayed_confirmable = inflight_chunk.is_delayed_confirmable();

            let spell_count = if inflight_chunk.as_ref().min_candidate(None).is_splitted() {
                1
            } else {
                inflight_chunk.as_ref().spell().count()
            };

            let mut in_candidate_cursor_position = 0;

            // キーストローク
            inflight_chunk
                .actual_key_strokes()
                .iter()
                .zip(inflight_chunk.construct_spell_end_vector().iter())
                .for_each(|(actual_key_stroke, spell_end)| {
                    on_typing_stat_manager
                        .on_actual_key_stroke(actual_key_stroke.is_correct(), spell_count);

                    if actual_key_stroke.is_correct() {
                        in_candidate_cursor_position += 1;

                        if let Some(delta) = spell_end {
                            on_typing_stat_manager.finish_spell(*delta);
                        }
                    } else {
                        if key_stroke_wrong_positions.last().map_or(true, |e| {
                            *e != (key_stroke_cursor_position + in_candidate_cursor_position)
                        }) {
                            key_stroke_wrong_positions
                                .push(key_stroke_cursor_position + in_candidate_cursor_position);
                        }
                    }
                });

            // この時点ではカーソル位置はこのチャンクの先頭を指しているので単純に足すだけで良い
            key_stroke_cursor_position += inflight_chunk.current_key_stroke_cursor_position();

            // 遅延確定候補を打ち終えている場合にはミスタイプは次のチャンク先頭に属するとみなす
            //
            // カーソル位置は特殊な処理をする必要はない
            // 遅延確定候補の候補内カーソル位置は次のチャンク先頭を指す位置にあるため
            if is_delayed_confirmable {
                if inflight_chunk.has_wrong_stroke_in_pending_key_strokes() {
                    key_stroke_wrong_positions.push(key_stroke_cursor_position);
                }
            }

            key_stroke.push_str(
                &inflight_chunk
                    .as_ref()
                    .min_candidate(None)
                    .whole_key_stroke(),
            );

            // 綴り

            // カーソル位置は複数ある場合がある
            let in_chunk_current_spell_cursor_positions =
                inflight_chunk.current_spell_cursor_positions();

            spell_cursor_positions = in_chunk_current_spell_cursor_positions
                .iter()
                .map(|in_chunk_current_spell_cursor_position| {
                    spell_head_position + in_chunk_current_spell_cursor_position
                })
                .collect();

            let wrong_spell_element_vector = inflight_chunk.construct_wrong_spell_element_vector();

            // 複数文字チャンクを個別に入力した場合はそれぞれの綴りについて
            // それ以外ではチャンク全体の綴りについて
            // タイプミス判定をする
            inflight_chunk
                .as_ref()
                .spell()
                .as_ref()
                .chars()
                .enumerate()
                .for_each(|(i, _)| {
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

            // 遅延確定候補を打ち終えている場合にはカーソルは次のチャンク先頭を指し保留中のミスタイプは次のチャンクのミスタイプとみなす
            if is_delayed_confirmable {
                spell_cursor_positions.clear();
                // この時点でspell_head_positionは次のチャンク先頭の綴りの位置を指している
                spell_cursor_positions.push(spell_head_position);

                if inflight_chunk.has_wrong_stroke_in_pending_key_strokes() {
                    spell_wrong_positions.push(spell_head_position);
                }
            }

            spell.push_str(inflight_chunk.as_ref().spell().as_ref());

            on_typing_stat_manager.add_unfinished_chunk(
                inflight_chunk
                    .as_ref()
                    .min_candidate(None)
                    .whole_key_stroke()
                    .chars()
                    .count(),
                inflight_chunk
                    .as_ref()
                    .ideal_key_stroke_candidate()
                    .as_ref()
                    .unwrap()
                    .whole_key_stroke()
                    .chars()
                    .count(),
                inflight_chunk.as_ref().spell().count(),
            );
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
                // キーストローク
                let candidate = unprocessed_chunk.min_candidate(next_chunk_head_constraint.clone());

                key_stroke.push_str(&candidate.whole_key_stroke());

                // 綴り
                spell.push_str(unprocessed_chunk.spell().as_ref());

                let spell_count = unprocessed_chunk.spell().count();
                spell_head_position += spell_count;

                // 現在打っているチャンクの遅延確定候補の間違いは未確定のチャンクの一番最初に帰属させる
                if i == 0 {
                    if let Some(inflight_chunk) = self.inflight_chunk.as_ref() {
                        if inflight_chunk.is_delayed_confirmable() {
                            inflight_chunk.pending_key_strokes().iter().for_each(
                                |actual_key_stroke| {
                                    on_typing_stat_manager.on_actual_key_stroke(
                                        actual_key_stroke.is_correct(),
                                        spell_count,
                                    );
                                },
                            );
                        }
                    }
                }

                let key_stroke_count = unprocessed_chunk
                    .ideal_key_stroke_candidate()
                    .as_ref()
                    .unwrap()
                    .whole_key_stroke()
                    .chars()
                    .count();

                on_typing_stat_manager.add_unfinished_chunk(
                    key_stroke_count,
                    key_stroke_count,
                    unprocessed_chunk.spell().count(),
                );

                // 次のチャンクへの制限を更新
                match candidate.next_chunk_head_constraint().clone() {
                    Some(constraint) => next_chunk_head_constraint.replace(constraint),
                    None => next_chunk_head_constraint.take(),
                };
            });

        let (key_stroke_on_typing_statistics, spell_on_typing_statistics, _) =
            on_typing_stat_manager.emit();

        (
            SpellDisplayInfo::new(
                spell,
                spell_cursor_positions,
                spell_wrong_positions,
                spell_head_position - 1,
                spell_on_typing_statistics,
            ),
            KeyStrokeDisplayInfo::new(
                key_stroke,
                key_stroke_cursor_position,
                key_stroke_wrong_positions,
                key_stroke_on_typing_statistics,
            ),
        )
    }
}
