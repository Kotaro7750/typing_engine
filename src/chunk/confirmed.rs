use crate::chunk::{has_actual_key_strokes::ChunkHasActualKeyStrokes, Chunk};
use crate::key_stroke::ActualKeyStroke;
use crate::key_stroke::KeyStrokeChar;

use super::ChunkKeyStrokeCandidate;

// 確定したチャンク
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ConfirmedChunk {
    chunk: Chunk,
    // ミスタイプも含めた実際のキーストローク
    key_strokes: Vec<ActualKeyStroke>,
}

impl ConfirmedChunk {
    pub(crate) fn new(chunk: Chunk, key_strokes: Vec<ActualKeyStroke>) -> Self {
        Self { chunk, key_strokes }
    }

    pub(crate) fn confirmed_candidate(&self) -> &ChunkKeyStrokeCandidate {
        assert!(self.chunk.key_stroke_candidates().as_ref().unwrap().len() == 1);

        self.chunk
            .key_stroke_candidates()
            .as_ref()
            .unwrap()
            .get(0)
            .unwrap()
    }

    // 確定した候補について次のチャンク先頭への制限を生成する
    pub(crate) fn next_chunk_head_constraint(&mut self) -> Option<KeyStrokeChar> {
        self.confirmed_candidate()
            .next_chunk_head_constraint()
            .clone()
    }
}

impl ChunkHasActualKeyStrokes for ConfirmedChunk {
    fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.key_strokes
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
    use crate::{gen_candidate, gen_chunk};
    use std::time::Duration;

    #[test]
    fn construct_spell_end_vector_1() {
        let cc = ConfirmedChunk::new(
            gen_chunk!(
                "きょ",
                vec![gen_candidate!(["ki", "xyo"]),],
                gen_candidate!(["kyo"])
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
                vec![gen_candidate!(["kyo"]),],
                gen_candidate!(["kyo"])
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
