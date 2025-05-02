use std::collections::VecDeque;
use std::time::Duration;

use crate::statistics::lap_statistics::{LapInfo, LapStatiticsBuilder};
use crate::statistics::statistical_event::{
    IdealKeyStrokeDeemedFinishedContext, InflightSpellSnapshottedContext, KeyStrokeCorrectContext,
    KeyStrokeSnapshottedContext, SpellFinishedContext, StatisticalEvent,
};
use crate::statistics::LapRequest;
use crate::typing_primitive_types::chunk::confirmed::ChunkConfirmed;
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::typing_primitive_types::chunk::inflight::{ChunkInflight, KeyStrokeResult};
use crate::typing_primitive_types::chunk::unprocessed::ChunkUnprocessed;
use crate::typing_primitive_types::chunk::Chunk;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
use crate::typing_primitive_types::key_stroke::KeyStrokeHitMiss;
use crate::typing_primitive_types::vocabulary::ViewPosition;

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
            let current_inflight_chunk = self.inflight_chunk.take().unwrap();
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

                    if let Some(correct_context) = &result.correct_context() {
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

    /// Snapshot current state and generate [`StatisticalEvent`](StatisticalEvent) for correction.
    /// This correction is assumed to be supplied for consumer already consumes events generated by
    /// [`stroke_key()`](Self::stroke_key).
    /// This method only generated snapshotted or deemed events, so caller should not treat them as confirmed event.
    pub(crate) fn snapshot(&self) -> Vec<StatisticalEvent> {
        let mut events = vec![];

        // Snapshotting is done following steps.
        // 1. Snapshot inflight chunk
        // 2. Snapshot unprocessed chunk

        // 1
        if let Some(inflight_chunk) = self.inflight_chunk.as_ref() {
            // Snapshot current ideal key strokes (deemed finished) in inflight chunk.
            inflight_chunk
                .wrong_key_stroke_count_of_each_ideal_key_stroke_index_only_confirmed(
                    inflight_chunk.ideal_key_stroke_candidate(),
                )
                .iter()
                .for_each(|wrong_count| {
                    events.push(StatisticalEvent::IdealKeyStrokeDeemedFinished(
                        IdealKeyStrokeDeemedFinishedContext::new(*wrong_count),
                    ));
                });

            if inflight_chunk.delayed_confirmable_candidate().is_none() {
                // Snapshot current(unfinished) key stroke in inflight chunk.
                let current_key_stroke_char = inflight_chunk
                    .effective_candidate()
                    .whole_key_stroke()
                    .key_stroke_chars()
                    .get(inflight_chunk.key_stroke_cursor_position())
                    .unwrap()
                    .clone();

                let wrong_key_stroke_char_of_current_key_stroke = inflight_chunk
                    .wrong_key_strokes_of_current_key_stroke()
                    .into_iter()
                    .map(|c| c.key_stroke().clone())
                    .collect();

                events.push(StatisticalEvent::KeyStrokeSnapshotted(
                    KeyStrokeSnapshottedContext::new_started(
                        &current_key_stroke_char,
                        wrong_key_stroke_char_of_current_key_stroke,
                    ),
                ));

                // Snapshot remaining (unfinished and not started) key strokes in inflight chunk.
                inflight_chunk
                    .remaining_key_stroke_chars(inflight_chunk.effective_candidate())
                    .iter()
                    // skip is needed because first key stroke is under being typed and already snapshotted.
                    .skip(1)
                    .for_each(|c| {
                        events.push(StatisticalEvent::KeyStrokeSnapshotted(
                            KeyStrokeSnapshottedContext::new_unstarted(c),
                        ));
                    });

                // Snapshot current (unfinished) spell in inflight chunk.
                events.push(StatisticalEvent::InflightSpellSnapshotted(
                    InflightSpellSnapshottedContext::new(
                        inflight_chunk
                            .spell()
                            .spell_at_index(inflight_chunk.spell_cursor_position().into()),
                        inflight_chunk.spell_cursor_position(),
                        inflight_chunk
                            .wrong_key_strokes_of_chunk_element_index(
                                inflight_chunk.spell_cursor_position().into(),
                            )
                            .into_iter()
                            .map(|c| c.key_stroke().clone())
                            .collect(),
                    ),
                ));
            } else {
                // When inflight chunk is delayed confirmable, this spell is deemed as finished
                events.push(StatisticalEvent::SpellDeemedFinished(
                    SpellFinishedContext::new(
                        inflight_chunk
                            .spell()
                            .spell_at_index(inflight_chunk.spell_cursor_position().into()),
                        inflight_chunk
                            .wrong_key_strokes_of_chunk_element_index(
                                inflight_chunk.spell_cursor_position().into(),
                            )
                            .len(),
                    ),
                ));
            }
        }

        // 2
        let mut next_chunk_head_constraint =
            self.inflight_chunk.as_ref().and_then(|inflight_chunk| {
                inflight_chunk
                    .min_candidate(None)
                    .next_chunk_head_constraint()
                    .clone()
            });

        self.unprocessed_chunks
            .iter()
            .enumerate()
            .for_each(|(i, unprocessed_chunk)| {
                let candidate = unprocessed_chunk.min_candidate(next_chunk_head_constraint.clone());
                // In first unprocessed chunk, first key stroke may be processed in delayed
                // confirmable candidate processing.
                let mut skip_first_key_stroke = false;

                // 未確定のチャンクの一番最初のみ現在打っているチャンクの遅延確定候補によって補正をする
                if i == 0 {
                    if let Some(inflight_chunk) = self.inflight_chunk.as_ref() {
                        if let Some(delayed_confirmable_candidate) =
                            inflight_chunk.delayed_confirmable_candidate()
                        {
                            skip_first_key_stroke = true;
                            // When inflight chunk is delayed confirmable, wrong key strokes in
                            // pending key strokes are treated as wrong strokes in next chunk.

                            let wrong_key_stroke_chars_in_pending_key_strokes: Vec<KeyStrokeChar> =
                                inflight_chunk
                                    .wrong_key_strokes_in_pending_key_strokes()
                                    .into_iter()
                                    .map(|c| c.key_stroke().clone())
                                    .collect();

                            let next_key_stroke_char = delayed_confirmable_candidate
                                .delayed_confirmed_candidate_info()
                                .as_ref()
                                .unwrap()
                                .first_next_chunk_head()
                                .clone();

                            events.push(StatisticalEvent::KeyStrokeSnapshotted(
                                KeyStrokeSnapshottedContext::new_started(
                                    &next_key_stroke_char,
                                    wrong_key_stroke_chars_in_pending_key_strokes.clone(),
                                ),
                            ));

                            let next_chunk_spell = unprocessed_chunk.spell().clone();
                            let next_chunk_spell_cursor_position = unprocessed_chunk
                                .chunk_spell_cursor_position_for_candidate(
                                    unprocessed_chunk
                                        .min_candidate(next_chunk_head_constraint.clone()),
                                );

                            events.push(StatisticalEvent::InflightSpellSnapshotted(
                                InflightSpellSnapshottedContext::new(
                                    next_chunk_spell,
                                    next_chunk_spell_cursor_position,
                                    wrong_key_stroke_chars_in_pending_key_strokes,
                                ),
                            ));
                        }
                    }
                }

                candidate
                    .key_stroke()
                    .whole_key_stroke()
                    .key_stroke_chars()
                    .iter()
                    .skip(if skip_first_key_stroke { 1 } else { 0 })
                    .for_each(|c| {
                        events.push(StatisticalEvent::KeyStrokeSnapshotted(
                            KeyStrokeSnapshottedContext::new_unstarted(c),
                        ));
                    });

                match candidate.next_chunk_head_constraint().clone() {
                    Some(constraint) => next_chunk_head_constraint.replace(constraint),
                    None => next_chunk_head_constraint.take(),
                };
            });

        events
    }

    /// Construct [`LapStatisticsBuilder`](LapStatiticsBuilder) for current state.
    /// First 3 elements in tuple are spell, key stroke and ideal key stroke lap statistics builder.
    pub(crate) fn construct_lap_statistics(
        &self,
        lap_request: LapRequest,
        view_position_of_spell: &[ViewPosition],
    ) -> LapInfo {
        let mut lap_statistics_builder = LapStatiticsBuilder::new(lap_request);

        // 1. 確定したチャンク
        // 2. タイプ中のチャンク
        // 3. 未処理のチャンク

        // 1. 確定したチャンク
        self.confirmed_chunks.iter().for_each(|confirmed_chunk| {
            update_lap_statistics(&mut lap_statistics_builder, confirmed_chunk, true);
        });

        // 2. タイプ中のチャンク
        if self.inflight_chunk.is_none() {
        } else {
            let inflight_chunk = self.inflight_chunk.as_ref().unwrap();
            update_lap_statistics(&mut lap_statistics_builder, inflight_chunk, false);
        }

        // 3. 未処理のチャンク
        let mut next_chunk_head_constraint =
            self.inflight_chunk.as_ref().and_then(|inflight_chunk| {
                inflight_chunk
                    .min_candidate(None)
                    .next_chunk_head_constraint()
                    .clone()
            });

        self.unprocessed_chunks
            .iter()
            .enumerate()
            .for_each(|(i, unprocessed_chunk)| {
                // 未確定のチャンクの一番最初のみ現在打っているチャンクの遅延確定候補によって補正をする
                if i == 0 {
                    if let Some(inflight_chunk) = self.inflight_chunk.as_ref() {
                        if inflight_chunk
                            .delayed_confirmable_candidate_index()
                            .is_some()
                        {
                            inflight_chunk.pending_key_strokes().iter().for_each(
                                |actual_key_stroke| {
                                    lap_statistics_builder.on_actual_key_stroke(
                                        actual_key_stroke.is_correct(),
                                        *actual_key_stroke.elapsed_time(),
                                    );
                                },
                            );
                        }
                    }
                }

                let key_stroke_element_count = unprocessed_chunk
                    .ideal_key_stroke_candidate()
                    .construct_key_stroke_element_count();

                lap_statistics_builder.on_add_chunk(
                    key_stroke_element_count.clone(),
                    key_stroke_element_count,
                    unprocessed_chunk.spell().count(),
                );

                let candidate = unprocessed_chunk.min_candidate(next_chunk_head_constraint.clone());
                match candidate.next_chunk_head_constraint().clone() {
                    Some(constraint) => next_chunk_head_constraint.replace(constraint),
                    None => next_chunk_head_constraint.take(),
                };
            });

        lap_statistics_builder.construct_lap_info(view_position_of_spell)
    }

    /// Returns the elapsed time of the last key stroke
    /// This return `None` if there is no key stroke.
    pub(crate) fn last_key_stroke_elapsed_time(&self) -> Option<Duration> {
        if let Some(inflight_chunk) = self.inflight_chunk.as_ref() {
            if let Some(last_key_stroke_in_pending) = inflight_chunk.pending_key_strokes().last() {
                Some(*last_key_stroke_in_pending.elapsed_time())
            } else {
                inflight_chunk.last_key_stroke_elapsed_time()
            }
        } else if let Some(last_confirmed_chunk) = self.confirmed_chunks.last() {
            last_confirmed_chunk.last_key_stroke_elapsed_time()
        } else {
            None
        }
    }
}

/// Update `LapStatisticsBuilder` using `ChunkHasActualKeyStrokes`.
fn update_lap_statistics(
    lap_statistics_builder: &mut LapStatiticsBuilder,
    chunk: &dyn ChunkHasActualKeyStrokes,
    fininsh_chunk: bool,
) {
    lap_statistics_builder.on_add_chunk(
        chunk
            .effective_candidate()
            .construct_key_stroke_element_count(),
        chunk
            .ideal_key_stroke_candidate()
            .construct_key_stroke_element_count(),
        chunk.spell().count(),
    );

    lap_statistics_builder.on_start_chunk(
        chunk
            .effective_candidate()
            .whole_key_stroke()
            .chars()
            .count(),
        chunk
            .ideal_key_stroke_candidate()
            .whole_key_stroke()
            .chars()
            .count(),
    );

    // Update statistics using actual key strokes
    chunk
        .actual_key_strokes()
        .iter()
        .zip(chunk.construct_spell_end_vector().iter())
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

    if fininsh_chunk {
        lap_statistics_builder.on_finish_chunk();
    }
}
