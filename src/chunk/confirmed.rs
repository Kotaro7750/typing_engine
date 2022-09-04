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

    pub(crate) fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.key_strokes
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

    // 確定したキーストロークのそれぞれの位置は綴り末尾かどうか
    // もし末尾だったとしたら何個の綴りの末尾かどうか
    pub(crate) fn construct_spell_end_vector(&self) -> Vec<Option<usize>> {
        let mut spell_end_vector = vec![None; self.key_strokes.len()];
        let confirmed_candidate = self.confirmed_candidate();

        let mut correct_key_stroke_index = 0;

        self.key_strokes
            .iter()
            .enumerate()
            .for_each(|(i, key_stroke)| {
                if key_stroke.is_correct() {
                    if confirmed_candidate
                        .is_element_end_at_key_stroke_index(correct_key_stroke_index)
                    {
                        if !confirmed_candidate.is_splitted() {
                            spell_end_vector[i] = Some(self.chunk.spell().count());
                        } else {
                            spell_end_vector[i] = Some(1);
                        }
                    }
                    correct_key_stroke_index += 1;
                }
            });

        spell_end_vector
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
