use crate::utility::calc_ceil_div;
use crate::{
    typing_primitive_types::chunk::{
        confirmed::ChunkConfirmed, has_actual_key_strokes::ChunkHasActualKeyStrokes,
        key_stroke_candidate::KeyStrokeElementCount, unprocessed::ChunkUnprocessed, Chunk,
        ChunkSpell,
    },
    KeyStrokeChar,
};

use super::ChunkSpellCursorPosition;

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
/// A struct representing the context for updating statistics when chunk is added
pub(crate) struct ChunkAddedContext {
    /// Spell of added chunk.
    spell: ChunkSpell,
    /// Ideal key stroke element count of added chunk
    ideal_key_stroke_element_count: KeyStrokeElementCount,
}

impl ChunkAddedContext {
    pub(crate) fn new(
        spell: ChunkSpell,
        ideal_key_stroke_element_count: KeyStrokeElementCount,
    ) -> Self {
        Self {
            spell,
            ideal_key_stroke_element_count,
        }
    }

    pub(crate) fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    pub(crate) fn spell_count(&self) -> usize {
        self.spell.count()
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

    /// Returns key stroke that generate event.
    pub(crate) fn key_stroke(&self) -> &KeyStrokeChar {
        &self.key_stroke
    }

    /// Returns wrong key strokes for typing this key stroke.
    pub(crate) fn wrong_key_strokes(&self) -> &[KeyStrokeChar] {
        &self.wrong_key_strokes
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing the context for updating statistics when inflight spell is snapshotted
pub(crate) struct KeyStrokeSnapshottedContext {
    /// Key stroke that generate event.
    key_stroke: KeyStrokeChar,
    /// Wrong key strokes for typing this key stroke.
    /// This is None when this key stroke is not started.
    wrong_key_strokes: Option<Vec<KeyStrokeChar>>,
}

impl KeyStrokeSnapshottedContext {
    pub(crate) fn new_unstarted(key_stroke: &KeyStrokeChar) -> Self {
        Self {
            key_stroke: key_stroke.clone(),
            wrong_key_strokes: None,
        }
    }

    pub(crate) fn new_started(
        key_stroke: &KeyStrokeChar,
        wrong_key_strokes: Vec<KeyStrokeChar>,
    ) -> Self {
        Self {
            key_stroke: key_stroke.clone(),
            wrong_key_strokes: Some(wrong_key_strokes),
        }
    }

    /// Returns key stroke that generate event.
    pub(crate) fn key_stroke(&self) -> &KeyStrokeChar {
        &self.key_stroke
    }

    /// Returns wrong key strokes for typing this key stroke.
    pub(crate) fn wrong_key_strokes(&self) -> Option<&[KeyStrokeChar]> {
        self.wrong_key_strokes.as_deref()
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing the context for updating statistics when inflight spell is snapshotted
pub(crate) struct InflightSpellSnapshottedContext {
    /// Spell that generate event.
    spell: ChunkSpell,
    /// Cursor position of chunk spell
    chunk_spell_cursor_position: ChunkSpellCursorPosition,
    /// Wrong key strokes for typing this inflight spell.
    wrong_key_strokes: Vec<KeyStrokeChar>,
}

impl InflightSpellSnapshottedContext {
    pub(crate) fn new(
        spell: ChunkSpell,
        chunk_spell_cursor_position: ChunkSpellCursorPosition,
        wrong_key_strokes: Vec<KeyStrokeChar>,
    ) -> Self {
        Self {
            spell,
            chunk_spell_cursor_position,
            wrong_key_strokes,
        }
    }

    pub(crate) fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    pub(crate) fn chunk_spell_cursor_position(&self) -> &ChunkSpellCursorPosition {
        &self.chunk_spell_cursor_position
    }

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
    ChunkConfirmed(ChunkConfirmedContext),
    // Below events are snapshotted or deemed events.
    // These events should not be treated as confirmed event because information of these events
    // may change in the future.
    /// Event generated when key stroke is snapshotted.
    KeyStrokeSnapshotted(KeyStrokeSnapshottedContext),
    /// Event generated when inflight spell is snapshotted.
    InflightSpellSnapshotted(InflightSpellSnapshottedContext),
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
        let ideal_key_stroke_element_count = added_chunk
            .ideal_key_stroke_candidate()
            .construct_key_stroke_element_count();

        StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
            added_chunk.spell().clone(),
            ideal_key_stroke_element_count,
        ))
    }

    /// Create ChunkConfirmed event from chunk
    pub(crate) fn new_from_confirmed_chunk(confirmed_chunk: &ChunkConfirmed) -> StatisticalEvent {
        StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(
            confirmed_chunk
                .ideal_key_stroke_candidate()
                .calc_key_stroke_count(),
            confirmed_chunk.wrong_key_stroke_count_of_key_stroke_index(),
        ))
    }
}
