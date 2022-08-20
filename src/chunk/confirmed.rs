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

    // チャンクのそれぞれの綴り（基本的には1つだが複数文字を個別で打った場合には2つ）でミスタイプがあったかどうか
    pub(crate) fn construct_wrong_spell_element_vector(&self) -> Vec<bool> {
        let element_count = if self.confirmed_candidate().is_splitted() {
            2
        } else {
            1
        };

        // 複数文字のチャンクを個別で打った場合には要素数は2になる
        let mut wrong_stroke_vector: Vec<bool> = vec![false; element_count];

        // 打たれたキーストロークではなく候補中のインデックス
        let mut current_key_stroke_index = 0;

        self.key_strokes.iter().for_each(|actual_key_stroke| {
            if actual_key_stroke.is_correct() {
                current_key_stroke_index += 1;
            } else {
                wrong_stroke_vector[self
                    .confirmed_candidate()
                    // キーストロークに対応する位置に変換する
                    .element_index_at_key_stroke_index(current_key_stroke_index)] = true;
            }
        });

        wrong_stroke_vector
    }

    // 確定したキーストロークのそれぞれの位置でミスタイプがあったかどうか
    pub(crate) fn construct_key_stroke_wrong_vector(&self) -> Vec<bool> {
        let mut key_stroke_wrong_vector =
            vec![false; self.confirmed_candidate().calc_key_stroke_count()];

        // 打たれたキーストロークではなく候補中のインデックス
        let mut current_key_stroke_index = 0;

        self.key_strokes.iter().for_each(|actual_key_stroke| {
            if actual_key_stroke.is_correct() {
                current_key_stroke_index += 1;
            } else {
                key_stroke_wrong_vector[current_key_stroke_index] = true;
            }
        });

        key_stroke_wrong_vector
    }
}

impl AsRef<Chunk> for ConfirmedChunk {
    fn as_ref(&self) -> &Chunk {
        &self.chunk
    }
}
