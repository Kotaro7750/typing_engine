use crate::key_stroke::ActualKeyStroke;

use super::{Chunk, ChunkKeyStrokeCandidate};

pub(crate) trait ChunkHasActualKeyStrokes: AsRef<Chunk> {
    fn actual_key_strokes(&self) -> &[ActualKeyStroke];
    /// 表示などに使う候補
    fn effective_candidate(&self) -> &ChunkKeyStrokeCandidate;

    /// 打たれたキーストロークのそれぞれの位置は綴り末尾かどうか
    /// もし末尾だったとしたら何個の綴りの末尾かどうか
    fn construct_spell_end_vector(&self) -> Vec<Option<usize>> {
        let mut spell_end_vector = vec![None; self.actual_key_strokes().len()];
        let confirmed_candidate = self.effective_candidate();

        let mut correct_key_stroke_index = 0;

        self.actual_key_strokes()
            .iter()
            .enumerate()
            .for_each(|(i, key_stroke)| {
                if key_stroke.is_correct() {
                    if confirmed_candidate
                        .is_element_end_at_key_stroke_index(correct_key_stroke_index)
                    {
                        if !confirmed_candidate.is_splitted() {
                            spell_end_vector[i] = Some(self.as_ref().spell().count());
                        } else {
                            spell_end_vector[i] = Some(1);
                        }
                    }
                    correct_key_stroke_index += 1;
                }
            });

        spell_end_vector
    }

    /// チャンクの綴りのそれぞれ（基本的には1つだが複数文字を個別で打った場合には2つ）でミスタイプがあったかどうか
    fn construct_wrong_spell_element_vector(&self) -> Vec<bool> {
        let element_count = if self.effective_candidate().is_splitted() {
            2
        } else {
            1
        };

        // 複数文字のチャンクを個別で打った場合には要素数は2になる
        let mut wrong_spell_element_vector: Vec<bool> = vec![false; element_count];

        // 打たれたキーストロークではなく候補中のインデックス
        let mut current_key_stroke_index = 0;

        self.actual_key_strokes()
            .iter()
            .for_each(|actual_key_stroke| {
                if actual_key_stroke.is_correct() {
                    current_key_stroke_index += 1;
                } else {
                    wrong_spell_element_vector[self
                        .effective_candidate()
                        // キーストロークに対応する位置に変換する
                        .element_index_at_key_stroke_index(current_key_stroke_index)] = true;
                }
            });

        wrong_spell_element_vector
    }
}
