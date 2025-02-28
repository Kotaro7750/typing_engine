use super::{Chunk, ChunkKeyStrokeCandidate, ChunkSpell};
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;

pub(crate) trait ChunkHasActualKeyStrokes: Chunk {
    fn actual_key_strokes(&self) -> &[ActualKeyStroke];
    /// 表示などに使う候補
    fn effective_candidate(&self) -> &ChunkKeyStrokeCandidate;

    /// Returns the position indexes of wrong key strokes of this chunk.
    /// Basically indexes are relative inner the chunk, but offset can be used for adjusting absolute position.
    fn wrong_key_stroke_positions(&self, offset: usize) -> Vec<usize> {
        let mut wrong_positions: Vec<usize> = vec![];
        let mut i = 0;
        let mut is_prev_wrong = false;

        self.actual_key_strokes().iter().for_each(|key_stroke| {
            if key_stroke.is_correct() {
                i += 1;
                is_prev_wrong = false;
            } else {
                if !is_prev_wrong {
                    wrong_positions.push(i + offset);
                }
                is_prev_wrong = true;
            }
        });

        wrong_positions
    }

    /// Returns the position indexes of wrong spell elements of this chunk.
    /// Basically indexes are relative inner the chunk, but offset can be used for adjusting absolute position.
    fn wrong_spell_element_positions(&self, offset: usize) -> Vec<usize> {
        self.wrong_key_stroke_positions(0)
            .iter()
            .map(|key_stroke_index| {
                self.effective_candidate()
                    .belonging_element_index_of_key_stroke(*key_stroke_index)
                    + offset
            })
            .collect()
    }

    /// Returns the position indexes of wrong spell of this chunk.
    /// Basically indexes are relative inner the chunk, but offset can be used for adjusting absolute position.
    fn wrong_spell_positions(&self, offset: usize) -> Vec<usize> {
        let wrong_spell_element_positions = self.wrong_spell_element_positions(0);

        match wrong_spell_element_positions.len() {
            0 => vec![],
            1 => match self.spell() {
                ChunkSpell::DoubleChar(_) => {
                    if self.effective_candidate().is_splitted() {
                        vec![wrong_spell_element_positions.first().unwrap() + offset]
                    } else {
                        assert_eq!(wrong_spell_element_positions, vec![0]);
                        vec![offset, offset + 1]
                    }
                }
                _ => {
                    assert_eq!(wrong_spell_element_positions, vec![0]);
                    vec![offset]
                }
            },
            2 => {
                assert_eq!(wrong_spell_element_positions, vec![0, 1]);
                assert!(self.effective_candidate().is_splitted());

                vec![offset, offset + 1]
            }
            _ => {
                unreachable!()
            }
        }
    }

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
                            spell_end_vector[i] = Some(self.spell().count());
                        } else {
                            spell_end_vector[i] = Some(1);
                        }
                    }
                    correct_key_stroke_index += 1;
                }
            });

        spell_end_vector
    }

    /// キーストロークが何個の綴りに対するものなのか
    /// 基本的には1だが複数文字の綴りをまとめて打つ場合には2となる
    fn effective_spell_count(&self) -> usize {
        // 複数文字の綴りをまとめて打つ場合には綴りの統計は2文字分カウントする必要がある
        if self.effective_candidate().is_splitted() {
            1
        } else {
            self.spell().count()
        }
    }
}
