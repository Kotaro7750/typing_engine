use std::time::Duration;

use crate::statistics::statistical_event::SpellFinishedContext;
use crate::typing_primitive_types::chunk::confirmed::ChunkConfirmed;
use crate::typing_primitive_types::chunk::has_actual_key_strokes::ChunkHasActualKeyStrokes;
use crate::typing_primitive_types::chunk::key_stroke_candidate::ChunkKeyStrokeCandidate;
use crate::typing_primitive_types::chunk::Chunk;
use crate::typing_primitive_types::chunk::ChunkElementIndex;
use crate::typing_primitive_types::chunk::ChunkSpell;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
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
    /// Key strokes that are not yet decided to be assigned to this chunk.
    /// This is used when this chunk is delayed confirmable.
    pending_key_strokes: Vec<ActualKeyStroke>,
}

impl ChunkInflight {
    pub fn new(
        spell: SpellString,
        key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
        inactive_key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
        ideal_candidate: ChunkKeyStrokeCandidate,
        actual_key_strokes: Vec<ActualKeyStroke>,
        key_stroke_cursor_position: usize,
        pending_key_strokes: Vec<ActualKeyStroke>,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            key_stroke_candidates,
            inactive_key_stroke_candidates,
            ideal_candidate,
            actual_key_strokes,
            key_stroke_cursor_position,
            pending_key_strokes,
        }
    }

    /// Returns the key stroke cursor position.
    pub(crate) fn key_stroke_cursor_position(&self) -> usize {
        self.key_stroke_cursor_position
    }

    pub(crate) fn key_stroke_candidates(&self) -> &[ChunkKeyStrokeCandidate] {
        &self.key_stroke_candidates
    }

    /// Returns the pending key strokes.
    pub(crate) fn pending_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.pending_key_strokes
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

    /// Returns the count of wrong key strokes in pending key strokes.
    pub(crate) fn wrong_key_stroke_count_in_pending_key_strokes(&self) -> usize {
        self.pending_key_strokes()
            .iter()
            .filter(|actual_key_stroke| !actual_key_stroke.is_correct())
            .count()
    }

    /// Returns the count of remaining key strokes of passed candidate.
    pub(crate) fn remaining_key_strokes(&self, candidate: &ChunkKeyStrokeCandidate) -> usize {
        candidate.calc_key_stroke_count() - self.key_stroke_cursor_position()
    }

    /// Returns the count of wrong key strokes for current typing key stroke.
    pub(crate) fn wrong_key_stroke_count_of_current_key_stroke(&self) -> usize {
        self.wrong_key_strokes_for_correct_key_stroke_index(self.key_stroke_cursor_position())
            .len()
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

        // When there is a delayed confirmable candidate, it must be the minimum candidate.
        if let Some(i) = self.delayed_confirmable_candidate_index() {
            assert!(min_candidate.unwrap().is_delayed_confirmed_candidate());
            assert!(
                i == self
                    .key_stroke_candidates
                    .iter()
                    .position(|c| *c == **(min_candidate.as_ref().unwrap()))
                    .unwrap()
            );
        }

        min_candidate.as_ref().unwrap()
    }

    /// Returns element index of finishable spell when passed key stroke cursor position is
    /// correct.
    fn finishable_spell_index(
        &self,
        key_stroke_cursor_position: usize,
    ) -> Option<ChunkElementIndex> {
        self.effective_candidate()
            .is_element_end_at_key_stroke_index(key_stroke_cursor_position)
            .and_then(|is_end| {
                if is_end {
                    self.effective_candidate()
                        .belonging_element_index_of_key_stroke(key_stroke_cursor_position)
                } else {
                    None
                }
            })
    }

    /// Returns SpellFinishedContext of passed finished spell index.
    /// This method assumes that all key strokes in pending list are drained.
    fn spell_finished_context(
        &self,
        finished_spell_index: ChunkElementIndex,
    ) -> SpellFinishedContext {
        SpellFinishedContext::new(
            self.spell().spell_at_index(finished_spell_index),
            self.wrong_key_stroke_count_of_element_index(finished_spell_index),
        )
    }

    /// Advance cursor position.
    /// Returns element index of finished spell if exist.
    fn advance_cursor(&mut self) -> Option<ChunkElementIndex> {
        self.key_stroke_cursor_position += 1;
        self.finishable_spell_index(self.key_stroke_cursor_position() - 1)
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

    /// Returns the index of delayed confirmable candidate.
    /// None is returned when there is no delayed confirmable candidate or the delayed confirmable
    /// candidate is not confirmed yet.
    pub(crate) fn delayed_confirmable_candidate_index(&self) -> Option<usize> {
        let index: Vec<usize> = self
            .key_stroke_candidates()
            .iter()
            .enumerate()
            .filter_map(|(i, candidate)| {
                if candidate.is_delayed_confirmed_candidate()
                    && self.is_candidate_confirmed(candidate)
                {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();

        // 同時に遅延確定候補が複数あることはない
        assert!(index.len() <= 1);

        index.first().copied()
    }

    /// Stroke a key to this chunk.
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
        let delayed_confirmable_index = self.delayed_confirmable_candidate_index();

        if let Some(delayed_confirmable_candidate_index) = delayed_confirmable_index {
            if key_stroke_candidates
                .get(delayed_confirmable_candidate_index)
                .unwrap()
                .delayed_confirmed_candidate_info()
                .as_ref()
                .unwrap()
                .can_confirm_with_key_stroke(key_stroke.clone())
            {
                self.pending_key_strokes
                    .push(ActualKeyStroke::new(elapsed_time, key_stroke, true));
                self.reduce_candidate(&[delayed_confirmable_candidate_index]);

                return KeyStrokeResult::ConfirmDelayed(KeyStrokeCorrectResult::new(
                    // At this point, key_stroke_cursor_position is already advanced.
                    self.finishable_spell_index(self.key_stroke_cursor_position() - 1)
                        .map(|index| self.spell_finished_context(index)),
                    Some(self.pending_key_strokes.drain(..).collect()),
                ));
            }
        }

        // それぞれの候補においてタイプされたキーストロークが有効かどうか
        let hit_candidate_index: Vec<usize> = key_stroke_candidates
            .iter()
            .enumerate()
            .filter_map(|(i, candidate)| {
                // At this time, delayed confirmed candidate is already determined to be wrong.
                if self
                    .delayed_confirmable_candidate_index()
                    .is_some_and(|index| index == i)
                {
                    None
                } else {
                    candidate
                        .is_hit(&key_stroke, self.key_stroke_cursor_position())
                        .then_some(i)
                }
            })
            .collect();

        let is_hit = !hit_candidate_index.is_empty();

        // If the chunk is delayed confirmable, key strokes are not added at this time.
        // This is because such key strokes can belong to the next chunk.
        if delayed_confirmable_index.is_some() {
            self.pending_key_strokes
                .push(ActualKeyStroke::new(elapsed_time, key_stroke, is_hit));
        } else {
            self.append_actual_key_stroke(ActualKeyStroke::new(elapsed_time, key_stroke, is_hit));
        }

        if is_hit {
            // If any candidate is hit, only those candidates are left and the cursor position is
            // advanced.
            self.reduce_candidate(&hit_candidate_index);
            let finished_spell_index = self.advance_cursor();

            let key_stroke_correct_ctx = if self.is_confirmed() {
                let pending_key_strokes: Vec<ActualKeyStroke> =
                    self.pending_key_strokes.drain(..).collect();

                pending_key_strokes.into_iter().for_each(|key_stroke| {
                    self.append_actual_key_stroke(key_stroke);
                });

                KeyStrokeCorrectResult::new(
                    finished_spell_index.map(|index| self.spell_finished_context(index)),
                    Some(self.pending_key_strokes.clone()),
                )
            } else {
                // If chunk become delayed confirmable after this key stroke, spell must not
                // finidhed at this time.
                let spell_finished_context = if self.delayed_confirmable_candidate_index().is_none()
                {
                    finished_spell_index.map(|index| self.spell_finished_context(index))
                } else {
                    None
                };

                KeyStrokeCorrectResult::new(spell_finished_context, None)
            };

            KeyStrokeResult::Correct(
                key_stroke_correct_ctx,
                self.wrong_key_strokes_for_correct_key_stroke_index(
                    self.key_stroke_cursor_position() - 1,
                ),
            )
        } else {
            KeyStrokeResult::Wrong
        }
    }

    /// Just append the actual key stroke to this chunk.
    /// This is usefull when drain the key strokes from pending list.
    pub(crate) fn append_actual_key_stroke(&mut self, actual_key_stroke: ActualKeyStroke) {
        self.actual_key_strokes.push(actual_key_stroke);
    }

    /// Returns the cursor position of the spell for this chunk.
    pub(crate) fn spell_cursor_position(&self) -> ChunkSpellCursorPosition {
        if let Some(element_index) = self
            .effective_candidate()
            .belonging_element_index_of_key_stroke(self.key_stroke_cursor_position())
        {
            match element_index {
                ChunkElementIndex::OnlyFirst => match self.spell() {
                    ChunkSpell::DoubleChar(_) => ChunkSpellCursorPosition::DoubleCombined,
                    _ => ChunkSpellCursorPosition::Single,
                },
                ChunkElementIndex::DoubleFirst => ChunkSpellCursorPosition::DoubleFirst,
                ChunkElementIndex::DoubleSecond => ChunkSpellCursorPosition::DoubleSecond,
            }
        } else {
            // If cursor position is out of range, it is considered to be at the end of the spell.
            // This heppens when the chunk is delayed confirmable.
            if self.effective_candidate().key_stroke().is_double_splitted() {
                ChunkSpellCursorPosition::DoubleSecond
            } else if self.effective_candidate().key_stroke().is_double() {
                ChunkSpellCursorPosition::DoubleCombined
            } else {
                ChunkSpellCursorPosition::Single
            }
        }
    }
}

impl Chunk for ChunkInflight {
    fn spell(&self) -> &ChunkSpell {
        &self.spell
    }
}

impl ChunkHasActualKeyStrokes for ChunkInflight {
    fn effective_candidate(&self) -> &ChunkKeyStrokeCandidate {
        self.min_candidate(None)
    }

    fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.actual_key_strokes
    }

    fn ideal_key_stroke_candidate(&self) -> &ChunkKeyStrokeCandidate {
        &self.ideal_candidate
    }
}

/// An enum representing the cursor position of spell for chunk.
pub(crate) enum ChunkSpellCursorPosition {
    /// A cursor is on the first and only character of the spell.
    Single,
    /// A cursor is on the first character of the spell.
    DoubleFirst,
    /// A cursor is on the second character of the spell.
    DoubleSecond,
    /// A cursor is on the first and second characters of the spell.
    DoubleCombined,
}

impl ChunkSpellCursorPosition {
    /// Returns the absolute cursor position of the spell for this chunk with passed offset added.
    pub(crate) fn into_absolute_cursor_position(self, offset: usize) -> Vec<usize> {
        match self {
            Self::Single | Self::DoubleFirst => vec![offset],
            Self::DoubleSecond => vec![offset + 1],
            Self::DoubleCombined => vec![offset, offset + 1],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// An enum representing the result of a key stroke to ChunkInflight.
pub(crate) enum KeyStrokeResult {
    /// A key stroke is correct.
    /// The second element of tuple is wrong key strokes for this correct key stroke.
    Correct(KeyStrokeCorrectResult, Vec<ActualKeyStroke>),
    /// A key stroke is correct and confirm delayed confirmed candidate.
    ConfirmDelayed(KeyStrokeCorrectResult),
    /// A key stroke is wrong.
    Wrong,
}

impl KeyStrokeResult {
    /// Returns if this result is correct.
    pub(crate) fn is_correct(&self) -> bool {
        matches!(self, Self::Correct(_, _) | Self::ConfirmDelayed(_))
    }

    /// Returns the correct context if this result is correct.
    pub(crate) fn correct_context(&self) -> Option<&KeyStrokeCorrectResult> {
        match self {
            Self::Correct(ctx, _) | Self::ConfirmDelayed(ctx) => Some(ctx),
            Self::Wrong => None,
        }
    }

    /// Returns the wrong key strokes if this result is correct.
    pub(crate) fn wrong_key_strokes(&self) -> Option<&[ActualKeyStroke]> {
        match self {
            Self::Correct(_, wrong_key_strokes) => Some(wrong_key_strokes),
            Self::ConfirmDelayed(_) => None,
            Self::Wrong => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct KeyStrokeCorrectResult {
    /// If Some, this key stroke finishes spell.
    spell_finished_context: Option<SpellFinishedContext>,
    /// If Some, this key stroke confirm the chunk.
    /// The vector is pending key strokes for the next chunk.
    chunk_confirmation: Option<Vec<ActualKeyStroke>>,
}

impl KeyStrokeCorrectResult {
    pub(crate) fn new(
        spell_finished_context: Option<SpellFinishedContext>,
        chunk_confirmation: Option<Vec<ActualKeyStroke>>,
    ) -> Self {
        Self {
            spell_finished_context,
            chunk_confirmation,
        }
    }

    pub(crate) fn spell_finished_context(&self) -> &Option<SpellFinishedContext> {
        &self.spell_finished_context
    }

    pub(crate) fn chunk_confirmation(&self) -> &Option<Vec<ActualKeyStroke>> {
        &self.chunk_confirmation
    }
}
