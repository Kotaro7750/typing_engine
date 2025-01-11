use crate::typing_primitive_types::key_stroke::ActualKeyStroke;

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

    /// チャンクの綴り要素のそれぞれ（基本的には1つだが複数文字を個別で打った場合には2つ）の初期化済みboolベクタを構築する
    fn initialized_spell_element_vector(&self) -> Vec<bool> {
        vec![
            false;
            if self.effective_candidate().is_splitted() {
                2
            } else {
                1
            }
        ]
    }

    /// チャンクのキーストロークのそれぞれの初期化済みboolベクタを構築する
    fn initialized_key_strokes_vector(&self) -> Vec<bool> {
        vec![false; self.effective_candidate().calc_key_stroke_count()]
    }

    /// キーストロークが何個の綴りに対するものなのか
    /// 基本的には1だが複数文字の綴りをまとめて打つ場合には2となる
    fn effective_spell_count(&self) -> usize {
        // 複数文字の綴りをまとめて打つ場合には綴りの統計は2文字分カウントする必要がある
        if self.effective_candidate().is_splitted() {
            1
        } else {
            self.as_ref().spell().count()
        }
    }
}
