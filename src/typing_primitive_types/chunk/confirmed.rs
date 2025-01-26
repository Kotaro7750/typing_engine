use super::{has_actual_key_strokes::ChunkHasActualKeyStrokes, Chunk};
use crate::typing_primitive_types::key_stroke::{ActualKeyStroke, KeyStrokeChar};

use super::key_stroke_candidate::ChunkKeyStrokeCandidate;

/// A struct representing an already confirmed chunk.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ConfirmedChunk {
    chunk: Chunk,
    actual_key_strokes: Vec<ActualKeyStroke>,
}

impl ConfirmedChunk {
    #[cfg(test)]
    pub(crate) fn new(chunk: Chunk, key_strokes: Vec<ActualKeyStroke>) -> Self {
        Self {
            chunk,
            actual_key_strokes: key_strokes,
        }
    }

    /// Returns a candidate that confirm this chunk.
    pub(crate) fn confirmed_candidate(&self) -> &ChunkKeyStrokeCandidate {
        assert!(self.chunk.key_stroke_candidates().as_ref().unwrap().len() == 1);

        self.chunk
            .key_stroke_candidates()
            .as_ref()
            .unwrap()
            .first()
            .unwrap()
    }

    /// Returns a constraint for the next chunk head based on the confirmed candidate.
    pub(crate) fn next_chunk_head_constraint(&mut self) -> Option<KeyStrokeChar> {
        self.confirmed_candidate()
            .next_chunk_head_constraint()
            .clone()
    }
}

impl From<Chunk> for ConfirmedChunk {
    fn from(chunk: Chunk) -> Self {
        let actual_key_strokes = chunk.actual_key_strokes().to_vec();
        Self {
            chunk,
            actual_key_strokes,
        }
    }
}

impl ChunkHasActualKeyStrokes for ConfirmedChunk {
    fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.actual_key_strokes
    }

    fn effective_candidate(&self) -> &ChunkKeyStrokeCandidate {
        self.confirmed_candidate()
    }
}

impl AsRef<Chunk> for ConfirmedChunk {
    fn as_ref(&self) -> &Chunk {
        &self.chunk
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::typing_primitive_types::chunk::ChunkState;
    use crate::{gen_candidate, gen_chunk};
    use std::time::Duration;

    #[test]
    fn construct_spell_end_vector_1() {
        let cc = ConfirmedChunk::new(
            gen_chunk!(
                "きょ",
                vec![
                    gen_candidate!(["kyo"], false, Some(1)),
                    gen_candidate!(["ki", "lyo"], false, Some(2)),
                    gen_candidate!(["ki", "xyo"], true, Some(5))
                ],
                ChunkState::Confirmed,
                gen_candidate!(["kyo"], true, None)
            ),
            vec![
                ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'i'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'x'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(5, 0), 'y'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(6, 0), 'o'.try_into().unwrap(), true),
            ],
        );

        let spell_end_vector = cc.construct_spell_end_vector();

        assert_eq!(
            spell_end_vector,
            vec![None, Some(1), None, None, None, Some(1)]
        );
    }

    #[test]
    fn construct_spell_end_vector_2() {
        let cc = ConfirmedChunk::new(
            gen_chunk!(
                "きょ",
                vec![
                    gen_candidate!(["kyo"], true, Some(3)),
                    gen_candidate!(["ki", "lyo"], false, Some(1)),
                    gen_candidate!(["ki", "xyo"], false, Some(1))
                ],
                ChunkState::Confirmed,
                gen_candidate!(["kyo"], true, None)
            ),
            vec![
                ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'y'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(4, 0), 'o'.try_into().unwrap(), true),
            ],
        );

        let spell_end_vector = cc.construct_spell_end_vector();

        assert_eq!(spell_end_vector, vec![None, None, None, Some(2)]);
    }
}
