use crate::chunk::Chunk;
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

    // 確定した候補について次のチャンク先頭への制限を生成する
    pub(crate) fn next_chunk_head_constraint(&mut self) -> Option<KeyStrokeChar> {
        assert!(self.chunk.key_stroke_candidates().as_ref().unwrap().len() == 1);

        let confirmed_candidate = self
            .chunk
            .key_stroke_candidates()
            .as_ref()
            .unwrap()
            .get(0)
            .unwrap();

        confirmed_candidate.next_chunk_head_constraint.clone()
    }
}
