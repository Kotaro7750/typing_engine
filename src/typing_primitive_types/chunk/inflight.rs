use std::time::Duration;

use crate::typing_primitive_types::chunk::confirmed::ChunkConfirmed;
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::typing_primitive_types::chunk::key_stroke_candidate::ChunkKeyStrokeCandidate;
use crate::typing_primitive_types::chunk::Chunk;
use crate::typing_primitive_types::chunk::ChunkSpell;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
use crate::typing_primitive_types::key_stroke::KeyStrokeResult;
use crate::typing_primitive_types::spell::SpellString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct representing a fundamental typing unit.
/// For alphabet, it is just a single character.
/// For Japanese, it can be a single character or a combination of two characters such as "きょ".
pub struct ChunkInflight {
    spell: ChunkSpell,
    /// Candidates of key strokes to type this chunk.
    /// Ex. For a chunk "きょ", there are key strokes like "kyo" and "kilyo".
    key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
    inactive_key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
    /// A key stroke candidate that is the shortest when typed.
    /// This is determined when key strokes are assigned, so it may not be possible to type this
    /// candidate depending on the actual key stroke sequence.
    ideal_candidate: ChunkKeyStrokeCandidate,
    /// Actual key strokes that also includes wrong key strokes.
    actual_key_strokes: Vec<ActualKeyStroke>,
    /// Cursur position index of the key stroke candidates.
    key_stroke_cursor_position: usize,
}

impl ChunkInflight {
    pub fn new(
        spell: SpellString,
        key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
        inactive_key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
        ideal_candidate: ChunkKeyStrokeCandidate,
        actual_key_strokes: Vec<ActualKeyStroke>,
        key_stroke_cursor_position: usize,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            key_stroke_candidates,
            inactive_key_stroke_candidates,
            ideal_candidate,
            actual_key_strokes,
            key_stroke_cursor_position,
        }
    }

    /// Returns the key stroke cursor position.
    pub(crate) fn key_stroke_cursor_position(&self) -> usize {
        self.key_stroke_cursor_position
    }

    pub(crate) fn key_stroke_candidates(&self) -> &[ChunkKeyStrokeCandidate] {
        &self.key_stroke_candidates
    }

    /// Consume this chunk and return a confirmed chunk.
    /// This method must be called only when this chunk is confirmed.
    pub(crate) fn try_into_confirmed(mut self) -> Result<ChunkConfirmed, ()> {
        if self.is_confirmed() {
            Ok(ChunkConfirmed::new(
                self.spell.into(),
                self.key_stroke_candidates.pop().unwrap(),
                self.inactive_key_stroke_candidates,
                self.ideal_candidate,
                self.actual_key_strokes,
            ))
        } else {
            Err(())
        }
    }

    /// Returns the ideal key stroke candidate of this chunk.
    pub(crate) fn ideal_key_stroke_candidate(&self) -> &ChunkKeyStrokeCandidate {
        &self.ideal_candidate
    }

    /// Returns the key stroke candidate that is the shortest when typed and satisfies the chunk
    /// head restriction.
    /// When there are multiple candidates with the same key stroke count, the one that appears
    /// earlier is selected.
    pub(crate) fn min_candidate(
        &self,
        chunk_head_striction: Option<KeyStrokeChar>,
    ) -> &ChunkKeyStrokeCandidate {
        let min_candidate = self
            .key_stroke_candidates()
            .iter()
            .filter(|candidate| {
                if let Some(chunk_head_striction) = &chunk_head_striction {
                    &candidate.key_stroke_char_at_position(0) == chunk_head_striction
                } else {
                    true
                }
            })
            .reduce(|min_candidate, candidate| {
                if candidate.calc_key_stroke_count() < min_candidate.calc_key_stroke_count() {
                    candidate
                } else {
                    min_candidate
                }
            });

        assert!(min_candidate.is_some());

        min_candidate.as_ref().unwrap()
    }

    /// Reduce the candidates of this chunk.
    /// Retain only the candidates whose index is true in the retain_vector.
    pub(crate) fn reduce_candidate(&mut self, retain_index: &[usize]) {
        let contain_set = retain_index
            .iter()
            .collect::<std::collections::HashSet<_>>();

        // This reversion is required for removing correctly.
        // If we remove from the first element, the index of the remaining elements will be
        // changed.

        let removed_candidates_reverse_order = (0..self.key_stroke_candidates.len())
            .rev()
            .filter(|i| !contain_set.contains(i))
            .map(|remove_index| self.key_stroke_candidates.remove(remove_index))
            .collect::<Vec<ChunkKeyStrokeCandidate>>();

        removed_candidates_reverse_order
            .into_iter()
            .rev()
            .for_each(|removed_candidate| {
                self.inactive_key_stroke_candidates.push(removed_candidate);
            });
    }

    /// Return if the passed candidate is confirmed.
    pub(crate) fn is_candidate_confirmed(&self, candidate: &ChunkKeyStrokeCandidate) -> bool {
        self.key_stroke_cursor_position() == candidate.calc_key_stroke_count()
    }

    /// チャンクが確定したか
    /// 遅延確定候補自体を打ち終えても確定自体はまだのとき確定としてはいけない
    pub(crate) fn is_confirmed(&mut self) -> bool {
        let key_stroke_candidates = self.key_stroke_candidates();

        // 確定している条件は
        // * 候補が1つである
        // * その候補を打ち終えている

        if key_stroke_candidates.len() != 1 {
            return false;
        }

        self.is_candidate_confirmed(key_stroke_candidates.first().unwrap())
    }

    /// 遅延確定候補があるとしたらそれを打ち終えているかどうか
    /// ないときには常にfalseを返す
    pub(crate) fn is_delayed_confirmable(&self) -> bool {
        let mut is_delayed_confirmable = false;

        self.key_stroke_candidates()
            .iter()
            .filter(|candidate| candidate.is_delayed_confirmed_candidate())
            .for_each(|candidate| {
                if self.is_candidate_confirmed(candidate) {
                    // 同時に遅延確定候補が複数あることはない
                    assert!(!is_delayed_confirmable);
                    is_delayed_confirmable = true;
                }
            });

        is_delayed_confirmable
    }

    /// 現在タイピング中のチャンクに対して1キーストロークのタイプを行う
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());

        // 前回のキーストロークよりも時間的に後でなくてはならない
        if let Some(last_key_stroke) = self.actual_key_strokes.last() {
            assert!(&elapsed_time >= last_key_stroke.elapsed_time());
        }

        let key_stroke_candidates = self.key_stroke_candidates();
        // For confirmation check correctness, save current status.
        // This is required when this key stroke will confirm this chunk.
        let is_delayed_confirmable = self.is_delayed_confirmable();

        if is_delayed_confirmable {
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
                .delayed_confirmed_candidate_info()
                .as_ref()
                .unwrap()
                .can_confirm_with_key_stroke(key_stroke.clone())
            {
                self.reduce_candidate(&[delayed_confirmed_candidate_index]);

                return KeyStrokeResult::Correct;
            }
        }

        // それぞれの候補においてタイプされたキーストロークが有効かどうか
        let hit_candidate_index: Vec<usize> = key_stroke_candidates
            .iter()
            .enumerate()
            .filter_map(|(i, candidate)| {
                // At this time, delayed confirmed candidate is already determined to be wrong.
                if self.is_delayed_confirmable() && candidate.is_delayed_confirmed_candidate() {
                    None
                } else {
                    candidate
                        .is_hit(&key_stroke, self.key_stroke_cursor_position())
                        .then_some(i)
                }
            })
            .collect();

        let is_hit = !hit_candidate_index.is_empty();

        // If any candidate is hit, only those candidates are left and the cursor position is
        // advanced.
        if is_hit {
            self.reduce_candidate(&hit_candidate_index);
            self.key_stroke_cursor_position += 1;
        }

        // If the chunk is delayed confirmable, key strokes are not added at this time.
        // This is because such key strokes can belong to the next chunk.
        if !is_delayed_confirmable {
            self.append_actual_key_stroke(ActualKeyStroke::new(elapsed_time, key_stroke, is_hit));
        }

        if is_hit {
            KeyStrokeResult::Correct
        } else {
            KeyStrokeResult::Wrong
        }
    }

    /// Just append the actual key stroke to this chunk.
    /// This is usefull when drain the key strokes from pending list.
    pub(crate) fn append_actual_key_stroke(&mut self, actual_key_stroke: ActualKeyStroke) {
        self.actual_key_strokes.push(actual_key_stroke);
    }

    // チャンクの綴りのどこにカーソルを当てるべきか
    // 基本的にはチャンク全体だが複数文字を個別で入力している場合にはそれぞれの文字になる
    pub(crate) fn current_spell_cursor_positions(&self) -> Vec<usize> {
        let mut cursor_positions: Vec<usize> = vec![];

        if self.min_candidate(None).is_splitted() {
            // 複数文字チャンクをまとめて入力する場合には現在入力中の綴りのみにカーソルを当てる
            cursor_positions.push(
                self.min_candidate(None)
                    .belonging_element_index_of_key_stroke(self.key_stroke_cursor_position()),
            );
        } else {
            // チャンクをまとめて入力している場合にはチャンクの綴り全体にカーソルを当てる
            self.spell()
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

impl Chunk for ChunkInflight {
    fn spell(&self) -> &ChunkSpell {
        &self.spell
    }
}

impl ChunkHasActualKeyStrokes for ChunkInflight {
    fn effective_candidate(&self) -> &ChunkKeyStrokeCandidate {
        self.key_stroke_candidates()
            .first()
            .expect("key stroke candidates must not be empty")
    }

    fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.actual_key_strokes
    }
}
