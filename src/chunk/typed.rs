use crate::chunk::Chunk;
use crate::key_stroke::ActualKeyStroke;

// 現在打たれているチャンクや打ち終わったチャンク
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedChunk {
    state: TypedChunkState,
    chunk: Chunk,
    // キーストローク候補のそれぞれに対するカーソル位置
    cursor_positions_of_candidates: Vec<usize>,
    // ミスタイプも含めた実際のキーストローク
    key_strokes: Vec<ActualKeyStroke>,
}

impl TypedChunk {
    pub(crate) fn is_confirmed(&self) -> bool {
        match self.state {
            TypedChunkState::Inflight => false,
            TypedChunkState::Confirmed => true,
        }
    }
}

impl From<Chunk> for TypedChunk {
    fn from(chunk: Chunk) -> Self {
        let key_stroke_candidates_count = match chunk.key_stroke_candidates_count() {
            Some(c) => c,
            None => panic!(),
        };

        // チャンクから直接確定したチャンクが生成されることはない
        Self {
            state: TypedChunkState::Inflight,
            chunk,
            cursor_positions_of_candidates: vec![0; key_stroke_candidates_count],
            key_strokes: vec![],
        }
    }
}

// タイピング中のチャンクの状態
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TypedChunkState {
    // 現在打たれている
    Inflight,
    // チャンクを打ち終わり確定している
    Confirmed,
}
