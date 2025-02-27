use crate::typing_primitive_types::spell::SpellString;
use has_actual_key_strokes::ChunkHasActualKeyStrokes;
use key_stroke_candidate::ChunkKeyStrokeCandidate;
use key_stroke_candidate::ChunkKeyStrokeCandidateWithoutCursor;

pub(crate) mod chunk_candidate_unappended;
pub(crate) mod confirmed;
pub(crate) mod has_actual_key_strokes;
pub(crate) mod inflight;
pub(crate) mod key_stroke_candidate;
mod single_n_availability;
pub(crate) mod unprocessed;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// An enum representing possible spell of a chunk.
pub(crate) enum ChunkSpell {
    DisplayableAscii(SpellString),
    SingleChar(SpellString),
    DoubleChar(SpellString),
}

impl ChunkSpell {
    fn new(ss: SpellString) -> Self {
        if ss.contains_displayable_ascii() {
            assert!(ss.chars().count() == 1);
            Self::DisplayableAscii(ss)
        } else {
            match ss.chars().count() {
                1 => Self::SingleChar(ss),
                2 => Self::DoubleChar(ss),
                _ => unreachable!("ChunkSpell's length must be 1 or 2"),
            }
        }
    }

    /// Split ChunkSpell::DoubleChar into two spells.
    fn split_double_char(&self) -> (SpellString, SpellString) {
        match self {
            Self::DoubleChar(spell_string) => (
                spell_string
                    .chars()
                    .next()
                    .unwrap()
                    .to_string()
                    .try_into()
                    .unwrap(),
                spell_string
                    .chars()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .try_into()
                    .unwrap(),
            ),
            _ => panic!("cannot split this ChunkSpell type"),
        }
    }

    /// Returns the number of characters in this spell.
    pub(crate) fn count(&self) -> usize {
        match self {
            ChunkSpell::DoubleChar(_) => 2,
            _ => 1,
        }
    }
}

impl AsRef<SpellString> for ChunkSpell {
    fn as_ref(&self) -> &SpellString {
        match self {
            ChunkSpell::DisplayableAscii(ss)
            | ChunkSpell::SingleChar(ss)
            | ChunkSpell::DoubleChar(ss) => ss,
        }
    }
}

/// A trait for a chunk.
pub(crate) trait Chunk {
    /// Returns the spell of this chunk.
    fn spell(&self) -> &ChunkSpell;
}
