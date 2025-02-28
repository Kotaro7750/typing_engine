use super::{Chunk, ChunkHasActualKeyStrokes};
use crate::typing_primitive_types::chunk::{ChunkKeyStrokeCandidate, ChunkSpell};
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::SpellString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct representing a fundamental typing unit.
/// For alphabet, it is just a single character.
/// For Japanese, it can be a single character or a combination of two characters such as "きょ".
pub(crate) struct ChunkConfirmed {
    spell: ChunkSpell,
    /// Key stroke candidate that confirm this chunk.
    confirmed_key_stroke_candidates: ChunkKeyStrokeCandidate,
    inactive_key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
    /// A key stroke candidate that is the shortest when typed.
    /// This is determined when key strokes are assigned, so it may not be possible to type this
    /// candidate depending on the actual key stroke sequence.
    ideal_candidate: ChunkKeyStrokeCandidate,
    /// Actual key strokes that also includes wrong key strokes.
    actual_key_strokes: Vec<ActualKeyStroke>,
}

impl ChunkConfirmed {
    pub(crate) fn new(
        spell: SpellString,
        confirmed_key_stroke_candidates: ChunkKeyStrokeCandidate,
        inactive_key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
        ideal_candidate: ChunkKeyStrokeCandidate,
        actual_key_strokes: Vec<ActualKeyStroke>,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            confirmed_key_stroke_candidates,
            inactive_key_stroke_candidates,
            ideal_candidate,
            actual_key_strokes,
        }
    }

    /// Returns ideal key stroke candidate.
    pub(crate) fn ideal_key_stroke_candidate(&self) -> &ChunkKeyStrokeCandidate {
        &self.ideal_candidate
    }

    /// Returns confirmed key stroke candidate.
    pub(crate) fn confirmed_key_stroke_candidates(&self) -> &ChunkKeyStrokeCandidate {
        &self.confirmed_key_stroke_candidates
    }
}

impl Chunk for ChunkConfirmed {
    fn spell(&self) -> &ChunkSpell {
        &self.spell
    }
}

impl ChunkHasActualKeyStrokes for ChunkConfirmed {
    fn effective_candidate(&self) -> &ChunkKeyStrokeCandidate {
        self.confirmed_key_stroke_candidates()
    }

    fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.actual_key_strokes
    }
}
