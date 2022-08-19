use crate::chunk::Chunk;
use crate::key_stroke::ActualKeyStroke;

// 確定したチャンク
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ConfirmedChunk {
    chunk: Chunk,
    // ミスタイプも含めた実際のキーストローク
    key_strokes: Vec<ActualKeyStroke>,
}
