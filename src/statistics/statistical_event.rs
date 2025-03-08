use crate::utility::calc_ceil_div;
use crate::{
    typing_primitive_types::chunk::{
        confirmed::ChunkConfirmed, has_actual_key_strokes::ChunkHasActualKeyStrokes,
        key_stroke_candidate::KeyStrokeElementCount, unprocessed::ChunkUnprocessed, Chunk,
        ChunkSpell,
    },
    KeyStrokeChar,
};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing the information for updating statistics when chunk is confirmed
pub(crate) struct ChunkConfirmedContext {
    /// Ideal key stroke count of confirmed chunk.
    ideal_key_stroke_count: usize,
    /// Wrong key stroke count for each key stroke of confirmed chunk.
    wrong_key_stroke_count_of_key_stroke_index: Vec<usize>,
}

impl ChunkConfirmedContext {
    pub(crate) fn new(
        ideal_key_stroke_count: usize,
        wrong_key_stroke_count_of_key_stroke_index: Vec<usize>,
    ) -> Self {
        Self {
            ideal_key_stroke_count,
            wrong_key_stroke_count_of_key_stroke_index,
        }
    }

    /// Returns ideal key stroke count of confirmed chunk.
    pub(crate) fn ideal_key_stroke_count(&self) -> usize {
        self.ideal_key_stroke_count
    }

    /// Returns completely correctness of confirmed chunk.
    pub(crate) fn completely_correct(&self) -> bool {
        self.wrong_key_stroke_count_of_key_stroke_index
            .iter()
            .all(|&count| count == 0)
    }

    /// Returns wrong key stroke count for each ideal key stroke of confirmed chunk.
    pub(crate) fn wrong_key_stroke_count_of_ideal_key_stroke_index(&self) -> Vec<usize> {
        let mut wrong_count = vec![0; self.ideal_key_stroke_count];
        let key_stroke_count = self.wrong_key_stroke_count_of_key_stroke_index.len();

        self.wrong_key_stroke_count_of_key_stroke_index
            .iter()
            .enumerate()
            .for_each(|(i, count)| {
                let ideal_key_stroke_index =
                    calc_ceil_div(self.ideal_key_stroke_count() * (i + 1), key_stroke_count) - 1;

                wrong_count[ideal_key_stroke_index] += count;
            });

        wrong_count
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing the information for updating statistics when chunk is confirmed
pub(crate) struct ChunkConfirmationInfo {
    pub(super) key_stroke_element_count: KeyStrokeElementCount,
    pub(super) ideal_key_stroke_element_count: KeyStrokeElementCount,
    pub(super) spell_count: usize,
    pub(super) candidate_key_stroke_count: usize,
    pub(super) ideal_candidate_key_stroke_count: usize,
    pub(super) effective_spell_count: usize,
    pub(super) actual_key_stroke_info: Vec<(bool, Option<usize>)>,
}

impl ChunkConfirmationInfo {
    pub(crate) fn new(
        key_stroke_element_count: KeyStrokeElementCount,
        ideal_key_stroke_element_count: KeyStrokeElementCount,
        spell_count: usize,
        candidate_key_stroke_count: usize,
        ideal_candidate_key_stroke_count: usize,
        effective_spell_count: usize,
        actual_key_stroke_info: Vec<(bool, Option<usize>)>,
    ) -> Self {
        Self {
            key_stroke_element_count,
            ideal_key_stroke_element_count,
            spell_count,
            candidate_key_stroke_count,
            ideal_candidate_key_stroke_count,
            effective_spell_count,
            actual_key_stroke_info,
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing the context for updating statistics when chunk is added
pub(crate) struct ChunkAddedContext {
    /// Spell count of added chunk
    spell_count: usize,
    /// Ideal key stroke element count of added chunk
    ideal_key_stroke_element_count: KeyStrokeElementCount,
}

impl ChunkAddedContext {
    pub(crate) fn new(
        spell_count: usize,
        ideal_key_stroke_element_count: KeyStrokeElementCount,
    ) -> Self {
        Self {
            spell_count,
            ideal_key_stroke_element_count,
        }
    }

    pub(crate) fn spell_count(&self) -> usize {
        self.spell_count
    }

    pub(crate) fn ideal_key_stroke_element_count(&self) -> &KeyStrokeElementCount {
        &self.ideal_key_stroke_element_count
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing the context for updating statistics when spell is finished
pub(crate) struct SpellFinishedContext {
    /// Finished spell.
    /// Finish detection is done with the unit of chunk.
    spell: ChunkSpell,
    /// Wrong key stroke count for typing spell.
    wrong_key_stroke_count: usize,
}

impl SpellFinishedContext {
    pub(crate) fn new(spell: ChunkSpell, wrong_key_stroke_count: usize) -> Self {
        Self {
            spell,
            wrong_key_stroke_count,
        }
    }

    /// Returns finished spell.
    pub(crate) fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    /// Returns wrong key stroke count for typing spell.
    pub(crate) fn wrong_key_stroke_count(&self) -> usize {
        self.wrong_key_stroke_count
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing the context for updating statistics when key stroke is correct
pub(crate) struct KeyStrokeCorrectContext {
    /// Key stroke that generate event.
    key_stroke: KeyStrokeChar,
    /// Wrong key strokes for typing this key stroke.
    wrong_key_strokes: Vec<KeyStrokeChar>,
}

impl KeyStrokeCorrectContext {
    pub(crate) fn new(key_stroke: KeyStrokeChar, wrong_key_strokes: Vec<KeyStrokeChar>) -> Self {
        Self {
            key_stroke,
            wrong_key_strokes,
        }
    }

    /// Returns wrong key strokes for typing this key stroke.
    pub(crate) fn wrong_key_strokes(&self) -> &[KeyStrokeChar] {
        &self.wrong_key_strokes
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Representing events generated when statistically meaningfull things happened
pub(crate) enum StatisticalEvent {
    KeyStrokeCorrect(KeyStrokeCorrectContext),
    /// Event generated when spell is finished
    SpellFinished(SpellFinishedContext),
    /// Event generated when chunk is added
    ChunkAdded(ChunkAddedContext),
    /// Event generated when chunk is confirmed
    ChunkConfirmed((ChunkConfirmationInfo, ChunkConfirmedContext)),
}

impl StatisticalEvent {
    /// Create KeyStrokeWrong event from KeyStrokeWrongContext
    pub(crate) fn new_from_key_stroke_correct(
        key_stroke_correct_context: KeyStrokeCorrectContext,
    ) -> Self {
        StatisticalEvent::KeyStrokeCorrect(key_stroke_correct_context)
    }

    /// Create SpellFinished event from SpellFinishedContext
    pub(crate) fn new_from_spell_finished(spell_finished_context: SpellFinishedContext) -> Self {
        StatisticalEvent::SpellFinished(spell_finished_context)
    }

    /// Create ChunkAdded event from chunk
    pub(crate) fn new_from_added_chunk(added_chunk: &ChunkUnprocessed) -> StatisticalEvent {
        let spell_count = added_chunk.spell().count();
        let ideal_key_stroke_element_count = added_chunk
            .ideal_key_stroke_candidate()
            .construct_key_stroke_element_count();

        StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
            spell_count,
            ideal_key_stroke_element_count,
        ))
    }

    /// Create ChunkConfirmed event from chunk
    pub(crate) fn new_from_confirmed_chunk(confirmed_chunk: &ChunkConfirmed) -> StatisticalEvent {
        let key_stroke_element_count = confirmed_chunk
            .effective_candidate()
            .construct_key_stroke_element_count();

        let ideal_key_stroke_element_count = confirmed_chunk
            .ideal_key_stroke_candidate()
            .construct_key_stroke_element_count();

        let spell_count = confirmed_chunk.spell().count();

        let candidate_key_stroke_count = confirmed_chunk
            .confirmed_key_stroke_candidates()
            .whole_key_stroke()
            .chars()
            .count();
        let ideal_candidate_key_stroke_count = confirmed_chunk
            .ideal_key_stroke_candidate()
            .whole_key_stroke()
            .chars()
            .count();

        let effective_spell_count = confirmed_chunk.effective_spell_count();

        let actual_key_stroke_info = confirmed_chunk
            .actual_key_strokes()
            .iter()
            .zip(confirmed_chunk.construct_spell_end_vector().iter())
            .map(|(actual_key_stroke, spell_end)| {
                (
                    actual_key_stroke.is_correct(),
                    if actual_key_stroke.is_correct() {
                        *spell_end
                    } else {
                        None
                    },
                )
            })
            .collect();

        StatisticalEvent::ChunkConfirmed((
            ChunkConfirmationInfo::new(
                key_stroke_element_count,
                ideal_key_stroke_element_count,
                spell_count,
                candidate_key_stroke_count,
                ideal_candidate_key_stroke_count,
                effective_spell_count,
                actual_key_stroke_info,
            ),
            ChunkConfirmedContext::new(
                confirmed_chunk
                    .ideal_key_stroke_candidate()
                    .calc_key_stroke_count(),
                confirmed_chunk.wrong_key_stroke_count_of_key_stroke_index(),
            ),
        ))
    }
}
