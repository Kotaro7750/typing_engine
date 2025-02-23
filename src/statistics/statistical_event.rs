use crate::typing_primitive_types::chunk::{
    has_actual_key_strokes::ChunkHasActualKeyStrokes, key_stroke_candidate::KeyStrokeElementCount,
    Chunk,
};

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
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Representing events generated when statistically meaningfull things happened
pub(crate) enum StatisticalEvent {
    /// Event generated when chunk is added
    ChunkAdded(ChunkAddedContext),
    /// Event generated when chunk is confirmed
    ChunkConfirmed(ChunkConfirmationInfo),
}

impl StatisticalEvent {
    /// Create ChunkAdded event from chunk
    pub(crate) fn new_from_added_chunk(added_chunk: &Chunk) -> StatisticalEvent {
        let spell_count = added_chunk.as_ref().spell().count();
        let ideal_key_stroke_element_count = added_chunk
            .ideal_key_stroke_candidate()
            .as_ref()
            .unwrap()
            .construct_key_stroke_element_count();

        StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
            spell_count,
            ideal_key_stroke_element_count,
        ))
    }

    /// Create ChunkConfirmed event from chunk
    pub(crate) fn new_from_confirmed_chunk(confirmed_chunk: &Chunk) -> StatisticalEvent {
        let key_stroke_element_count = confirmed_chunk
            .min_candidate(None)
            .construct_key_stroke_element_count();

        let ideal_key_stroke_element_count = confirmed_chunk
            .ideal_key_stroke_candidate()
            .as_ref()
            .unwrap()
            .construct_key_stroke_element_count();

        let spell_count = confirmed_chunk.as_ref().spell().count();

        let candidate_key_stroke_count = confirmed_chunk
            .confirmed_candidate()
            .whole_key_stroke()
            .chars()
            .count();
        let ideal_candidate_key_stroke_count = confirmed_chunk
            .ideal_key_stroke_candidate()
            .as_ref()
            .unwrap()
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

        StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
            key_stroke_element_count,
            ideal_key_stroke_element_count,
            spell_count,
            candidate_key_stroke_count,
            ideal_candidate_key_stroke_count,
            effective_spell_count,
            actual_key_stroke_info,
        ))
    }
}
