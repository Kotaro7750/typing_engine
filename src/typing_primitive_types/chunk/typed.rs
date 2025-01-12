use std::time::Duration;

use super::{has_actual_key_strokes::ChunkHasActualKeyStrokes, Chunk};
use crate::typing_primitive_types::key_stroke::{ActualKeyStroke, KeyStrokeChar};

use super::confirmed::ConfirmedChunk;

#[cfg(test)]
mod test;

// 現在打たれているチャンク
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedChunk {
    chunk: Chunk,
    // キーストローク候補のそれぞれに対するカーソル位置
    cursor_positions_of_candidates: Vec<usize>,
    // ミスタイプも含めた実際のキーストローク
    key_strokes: Vec<ActualKeyStroke>,
    // 遅延確定候補がある場合にはキーストロークが最終的にこのチャンクに属するのか次のチャンクに属するのかが確定しないのでそれを一時的に保持しておく
    pending_key_strokes: Vec<ActualKeyStroke>,
}

impl TypedChunk {
    #[cfg(test)]
    pub(crate) fn new(
        chunk: Chunk,
        cursor_positions_of_candidates: Vec<usize>,
        key_strokes: Vec<ActualKeyStroke>,
        pending_key_strokes: Vec<ActualKeyStroke>,
    ) -> Self {
        Self {
            chunk,
            cursor_positions_of_candidates,
            key_strokes,
            pending_key_strokes,
        }
    }

    pub(crate) fn pending_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.pending_key_strokes
    }

    /// チャンクが確定したか
    /// 遅延確定候補自体を打ち終えても確定自体はまだのとき確定としてはいけない
    pub(crate) fn is_confirmed(&mut self) -> bool {
        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        // 確定している条件は
        // * 候補が1つである
        // * その候補を打ち終えている

        if key_stroke_candidates.len() != 1 {
            return false;
        }

        let mut is_confirmed = false;

        key_stroke_candidates
            .iter()
            .zip(&self.cursor_positions_of_candidates)
            .for_each(|(candidate, cursor_position)| {
                if *cursor_position >= candidate.calc_key_stroke_count() {
                    assert!(!is_confirmed);

                    is_confirmed = true;
                }
            });

        is_confirmed
    }

    /// 遅延確定候補があるとしたらそれを打ち終えているかどうか
    /// ないときには常にfalseを返す
    pub(crate) fn is_delayed_confirmable(&self) -> bool {
        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        let mut is_delayed_confirmable = false;

        key_stroke_candidates
            .iter()
            .zip(&self.cursor_positions_of_candidates)
            .filter(|(candidate, _)| candidate.is_delayed_confirmed_candidate())
            .for_each(|(candidate, cursor_position)| {
                if *cursor_position >= candidate.calc_key_stroke_count() {
                    // 同時に２つの遅延確定候補が終了することはないはずである
                    assert!(!is_delayed_confirmable);

                    is_delayed_confirmable = true;
                }
            });

        is_delayed_confirmable
    }

    /// 遅延確定候補のために保持しているキーストロークの中にミスタイプがあるかどうか
    pub(crate) fn has_wrong_stroke_in_pending_key_strokes(&self) -> bool {
        self.pending_key_strokes
            .iter()
            .map(|actual_key_stroke| !actual_key_stroke.is_correct())
            .reduce(|accum, is_correct| accum || is_correct)
            .map_or(false, |r| r)
    }

    /// 現在タイピング中のチャンクに対して1キーストロークのタイプを行う
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());

        // 前回のキーストロークよりも時間的に後でなくてはならない
        if let Some(last_key_stroke) = self.key_strokes.last() {
            assert!(&elapsed_time >= last_key_stroke.elapsed_time());
        }

        // 打ち終えた遅延確定候補がある場合とそうでない場合で処理を分ける
        let key_stroke_result = if self.is_delayed_confirmable() {
            self.stroke_key_to_delayed_confirmable(key_stroke, elapsed_time)
        } else {
            self.stroke_key_to_no_delayed_confirmable(key_stroke, elapsed_time)
        };

        // 遅延確定候補以外の候補で確定した場合にはpendingしていたキーストロークを加える必要がある
        if self.is_confirmed() && !self.is_delayed_confirmable() {
            self.pending_key_strokes
                .drain(..)
                .for_each(|key_stroke| self.key_strokes.push(key_stroke));
        }

        key_stroke_result
    }

    /// 遅延確定候補ではないかそうであってもまだ打ち終えていないチャンクを対象にキーストロークを行う
    fn stroke_key_to_no_delayed_confirmable(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());
        assert!(!self.is_delayed_confirmable());

        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        assert_eq!(
            key_stroke_candidates.len(),
            self.cursor_positions_of_candidates.len()
        );

        // それぞれの候補においてタイプされたキーストロークが有効かどうか
        let candidate_hit_miss: Vec<bool> = key_stroke_candidates
            .iter()
            .zip(self.cursor_positions_of_candidates.iter())
            .map(|(candidate, cursor_position)| {
                candidate.key_stroke_char_at_position(*cursor_position) == key_stroke
            })
            .collect();

        let is_hit = candidate_hit_miss.contains(&true);

        // 何かしらの候補についてキーストロークが有効だったらそれらの候補のみを残しカーソル位置を進める
        if is_hit {
            self.chunk.reduce_candidate(&candidate_hit_miss);

            let mut index = 0;
            self.cursor_positions_of_candidates.retain(|_| {
                let is_hit = *candidate_hit_miss.get(index).unwrap();
                index += 1;
                is_hit
            });

            self.cursor_positions_of_candidates
                .iter_mut()
                .for_each(|cursor_position| *cursor_position += 1);
        }

        self.key_strokes
            .push(ActualKeyStroke::new(elapsed_time, key_stroke, is_hit));

        if is_hit {
            KeyStrokeResult::Correct
        } else {
            KeyStrokeResult::Wrong
        }
    }

    /// 遅延確定候補を持つその候補を打ち終えたチャンクに対してキーストロークを行う
    fn stroke_key_to_delayed_confirmable(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());
        assert!(self.is_delayed_confirmable());

        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        assert_eq!(
            key_stroke_candidates.len(),
            self.cursor_positions_of_candidates.len()
        );

        // 打ち終えている遅延確定候補がある場合にはキーストロークが有効かの比較は遅延確定候補とそうでない候補で比較の仕方が異なる
        // 遅延確定候補の比較は次のチャンク先頭との比較で行う
        // そうでない候補の比較は通常のやり方と同じである

        let delayed_confirmed_candidate_index = key_stroke_candidates
            .iter()
            .position(|candidate| candidate.is_delayed_confirmed_candidate())
            .unwrap();

        // 次のチャンク先頭にヒットするなら遅延確定候補で確定する
        if key_stroke_candidates
            .get(delayed_confirmed_candidate_index)
            .unwrap()
            .delayed_confirmed_candiate_info()
            .as_ref()
            .unwrap()
            .is_valid_key_stroke(key_stroke.clone())
        {
            // 遅延確定候補以外の候補を削除する
            let mut candidate_reduce_vec = vec![false; key_stroke_candidates.len()];
            candidate_reduce_vec[delayed_confirmed_candidate_index] = true;

            self.chunk.reduce_candidate(&candidate_reduce_vec);

            // 遅延確定候補以外のカーソル位置も削除する
            let mut index = 0;
            self.cursor_positions_of_candidates.retain(|_| {
                let is_hit = *candidate_reduce_vec.get(index).unwrap();
                index += 1;
                is_hit
            });

            self.pending_key_strokes
                .push(ActualKeyStroke::new(elapsed_time, key_stroke, true));

            return KeyStrokeResult::Correct;
        }

        // それぞれの候補においてタイプされたキーストロークが有効かどうか
        let candidate_hit_miss: Vec<bool> = key_stroke_candidates
            .iter()
            .zip(self.cursor_positions_of_candidates.iter())
            .map(|(candidate, cursor_position)| {
                // 遅延確定候補は既にミスであることが確定している
                if candidate.is_delayed_confirmed_candidate() {
                    false
                } else {
                    candidate.key_stroke_char_at_position(*cursor_position) == key_stroke
                }
            })
            .collect();

        let is_hit = candidate_hit_miss.contains(&true);

        // 何かしらの候補についてキーストロークが有効だったらそれらの候補のみを残しカーソル位置を進める
        if is_hit {
            self.chunk.reduce_candidate(&candidate_hit_miss);

            let mut index = 0;
            self.cursor_positions_of_candidates.retain(|_| {
                let is_hit = *candidate_hit_miss.get(index).unwrap();
                index += 1;
                is_hit
            });

            self.cursor_positions_of_candidates
                .iter_mut()
                .for_each(|cursor_position| *cursor_position += 1);
        }

        self.pending_key_strokes
            .push(ActualKeyStroke::new(elapsed_time, key_stroke, is_hit));

        if is_hit {
            KeyStrokeResult::Correct
        } else {
            KeyStrokeResult::Wrong
        }
    }

    pub(crate) fn take_pending_key_strokes(&mut self) -> Vec<ActualKeyStroke> {
        self.pending_key_strokes.drain(..).collect()
    }

    // チャンクのキーストロークのどこにカーソルを当てるべきか
    pub(crate) fn current_key_stroke_cursor_position(&self) -> usize {
        *self
            .cursor_positions_of_candidates
            .iter()
            .reduce(|cursor_position, cursor_position_of_candidate| {
                // XXX 適切に候補を削減していれば全ての候補でカーソル位置は同じなはず
                assert!(cursor_position == cursor_position_of_candidate);
                cursor_position_of_candidate
            })
            .unwrap()
    }

    // チャンクの綴りのどこにカーソルを当てるべきか
    // 基本的にはチャンク全体だが複数文字を個別で入力している場合にはそれぞれの文字になる
    pub(crate) fn current_spell_cursor_positions(&self) -> Vec<usize> {
        let mut cursor_positions: Vec<usize> = vec![];

        if self.as_ref().min_candidate(None).is_splitted() {
            // 複数文字チャンクをまとめて入力する場合には現在入力中の綴りのみにカーソルを当てる
            cursor_positions.push(
                self.as_ref()
                    .min_candidate(None)
                    .element_index_at_key_stroke_index(self.current_key_stroke_cursor_position()),
            );
        } else {
            // チャンクをまとめて入力している場合にはチャンクの綴り全体にカーソルを当てる
            self.as_ref()
                .spell()
                .as_ref()
                .chars()
                .enumerate()
                .for_each(|(i, _)| {
                    cursor_positions.push(i);
                });
        }

        cursor_positions
    }
}

impl ChunkHasActualKeyStrokes for TypedChunk {
    fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.key_strokes
    }

    fn effective_candidate(&self) -> &super::ChunkKeyStrokeCandidate {
        self.as_ref().min_candidate(None)
    }
}

impl From<Chunk> for TypedChunk {
    fn from(chunk: Chunk) -> Self {
        let key_stroke_candidates_count = match chunk.key_stroke_candidates_count() {
            Some(c) => c,
            None => panic!(),
        };

        Self {
            chunk,
            cursor_positions_of_candidates: vec![0; key_stroke_candidates_count],
            key_strokes: vec![],
            pending_key_strokes: vec![],
        }
    }
}

impl Into<ConfirmedChunk> for TypedChunk {
    fn into(self) -> ConfirmedChunk {
        ConfirmedChunk::new(self.chunk, self.key_strokes)
    }
}

impl AsRef<Chunk> for TypedChunk {
    fn as_ref(&self) -> &Chunk {
        &self.chunk
    }
}

// キーストロークの結果
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum KeyStrokeResult {
    // キーストロークが正しかったケース
    Correct,
    // キーストロークが間違っていたケース
    Wrong,
}
