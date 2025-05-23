use std::time::Duration;

use super::{Chunk, ChunkElementIndex, ChunkKeyStrokeCandidate};
use crate::{
    statistics::multi_target_position_convert::convert_between_key_stroke_delta,
    typing_primitive_types::key_stroke::ActualKeyStroke,
};

pub(crate) trait ChunkHasActualKeyStrokes: Chunk {
    /// Returns actual key strokes of this chunk.
    fn actual_key_strokes(&self) -> &[ActualKeyStroke];
    /// Returns candidate effective for display.
    fn effective_candidate(&self) -> &ChunkKeyStrokeCandidate;
    /// Returns ideal key stroke candidate.
    fn ideal_key_stroke_candidate(&self) -> &ChunkKeyStrokeCandidate;

    /// Returns all wrong ActualKeyStroke that belongs to passed key stroke index
    /// When key stroke at passed index is not confirmed, this method returns tentative wrong key
    /// strokes.
    fn wrong_key_strokes_of_key_stroke_index(
        &self,
        key_stroke_index: usize,
    ) -> Vec<ActualKeyStroke> {
        let mut i = 0;
        let mut wrong_key_strokes = vec![];

        self.actual_key_strokes().iter().for_each(|key_stroke| {
            if key_stroke.is_correct() {
                i += 1;
            } else if i == key_stroke_index {
                wrong_key_strokes.push(key_stroke.clone());
            }
        });

        wrong_key_strokes
    }

    /// Returns the count of wrong key strokes of each key stroke index.
    /// This method only returns the count for confirmed or under typing key strokes.
    fn wrong_key_stroke_count_of_each_key_stroke_index(&self) -> Vec<usize> {
        let mut wrong_key_stroke_count = vec![];
        let mut wrong_count = 0;

        self.actual_key_strokes().iter().for_each(|key_stroke| {
            if key_stroke.is_correct() {
                wrong_key_stroke_count.push(wrong_count);
                wrong_count = 0;
            } else {
                wrong_count += 1;
            }
        });

        // When wrong_count is greater than 0, it means that the last key stroke is wrong and not
        // confirmed.
        if wrong_count > 0 {
            wrong_key_stroke_count.push(wrong_count);
        }

        wrong_key_stroke_count
    }

    /// Returns the count of wrong key strokes of each key stroke index.
    /// This method only returns the count for confirmed.
    /// The count for under typing key strokes is not included.
    fn wrong_key_stroke_count_of_each_key_stroke_index_only_confirmed(&self) -> Vec<usize> {
        let mut wrong_key_stroke_count = vec![];
        let mut wrong_count = 0;

        self.actual_key_strokes().iter().for_each(|key_stroke| {
            if key_stroke.is_correct() {
                wrong_key_stroke_count.push(wrong_count);
                wrong_count = 0;
            } else {
                wrong_count += 1;
            }
        });

        wrong_key_stroke_count
    }

    /// Returns the count of wrong key strokes of the ideal key stroke index.
    /// This method only returns the count for confirmed or under typing key strokes.
    fn wrong_key_stroke_count_of_each_ideal_key_stroke_index_only_confirmed(
        &self,
        ideal_candidate: &ChunkKeyStrokeCandidate,
    ) -> Vec<usize> {
        let mut wrong_ideal_key_stroke_count = vec![];

        self.wrong_key_stroke_count_of_each_key_stroke_index_only_confirmed()
            .into_iter()
            .enumerate()
            .map(|(i, count)| {
                let ideal_key_stroke_index = convert_between_key_stroke_delta(
                    &self
                        .effective_candidate()
                        .construct_key_stroke_element_count(),
                    &ideal_candidate.construct_key_stroke_element_count(),
                    self.spell().count(),
                    i + 1,
                ) - 1;

                (ideal_key_stroke_index, count)
            })
            .for_each(|(ideal_key_stroke_index, count)| {
                if wrong_ideal_key_stroke_count
                    .get(ideal_key_stroke_index)
                    .is_none()
                {
                    wrong_ideal_key_stroke_count.push(0);
                }

                wrong_ideal_key_stroke_count[ideal_key_stroke_index] += count;
            });

        wrong_ideal_key_stroke_count
    }

    /// Returns the count of wrong key strokes that belongs to passed element index
    /// When key stroke at passed index is not confirmed, this method returns tentative wrong key
    /// strokes.
    fn wrong_key_stroke_count_of_chunk_element_index(
        &self,
        element_index: ChunkElementIndex,
    ) -> usize {
        self.wrong_key_stroke_count_of_each_key_stroke_index()
            .iter()
            .enumerate()
            .filter(|(i, _)| {
                self.effective_candidate()
                    .belonging_element_index_of_key_stroke(*i)
                    .map_or(false, |index| index == element_index)
            })
            .map(|(_, count)| *count)
            .reduce(|sum, count| sum + count)
            .map_or(0, |count| count)
    }

    /// Returns all wrong ActualKeyStroke that belongs to passed element index
    /// When key stroke at passed index is not confirmed, this method returns tentative wrong key
    /// strokes.
    fn wrong_key_strokes_of_chunk_element_index(
        &self,
        element_index: ChunkElementIndex,
    ) -> Vec<ActualKeyStroke> {
        let mut wrong_key_strokes = vec![];

        self.effective_candidate()
            .whole_key_stroke()
            .chars()
            .enumerate()
            .for_each(|(i, _)| {
                if self
                    .effective_candidate()
                    .belonging_element_index_of_key_stroke(i)
                    .is_some_and(|index| index == element_index)
                {
                    wrong_key_strokes.extend(self.wrong_key_strokes_of_key_stroke_index(i));
                }
            });

        wrong_key_strokes
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
                        .is_some_and(|is_end| is_end)
                    {
                        if !confirmed_candidate.key_stroke().is_double_splitted() {
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

    /// Returns the elapsed time of the last key stroke.
    /// If there are no key strokes, returns None.
    fn last_key_stroke_elapsed_time(&self) -> Option<Duration> {
        self.actual_key_strokes()
            .last()
            .map(|key_stroke| *key_stroke.elapsed_time())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{gen_candidate, gen_candidate_key_stroke, gen_chunk_inflight};
    use std::time::Duration;

    #[test]
    fn wrong_positions_of_unfinished_key_stroke() {
        let typed_chunk = gen_chunk_inflight!(
            "か",
            vec![gen_candidate!(gen_candidate_key_stroke!(["ka"])),],
            vec![],
            gen_candidate!(gen_candidate_key_stroke!(["ka"])),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'h'.try_into().unwrap(),
                false
            )],
            0,
            []
        );

        assert_eq!(
            typed_chunk.wrong_key_strokes_of_key_stroke_index(0),
            vec![ActualKeyStroke::new(
                Duration::new(1, 0),
                'h'.try_into().unwrap(),
                false
            ),]
        );
        assert_eq!(typed_chunk.wrong_key_strokes_of_key_stroke_index(1), vec![]);
        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_key_stroke_index(),
            vec![1]
        );
        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_key_stroke_index_only_confirmed(),
            vec![]
        );

        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_ideal_key_stroke_index_only_confirmed(
                &gen_candidate!(gen_candidate_key_stroke!(["ka"]))
            ),
            vec![]
        );

        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::OnlyFirst),
            1
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::OnlyFirst),
            vec![ActualKeyStroke::new(
                Duration::new(1, 0),
                'h'.try_into().unwrap(),
                false
            ),]
        );
        assert_eq!(
            typed_chunk
                .wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::DoubleFirst),
            0
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::DoubleFirst),
            vec![]
        );
        assert_eq!(
            typed_chunk
                .wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::DoubleSecond),
            0
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::DoubleSecond),
            vec![]
        );
    }

    #[test]
    fn wrong_positions_of_double_splitted() {
        let typed_chunk = gen_chunk_inflight!(
            "じょ",
            vec![gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),],
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["jo"])),
                gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(5, 0), 'i'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(6, 0), 'x'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(7, 0), 'x'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(8, 0), 'y'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(9, 0), 'o'.try_into().unwrap(), true)
            ],
            5,
            []
        );

        assert_eq!(
            typed_chunk.wrong_key_strokes_of_key_stroke_index(0),
            vec![ActualKeyStroke::new(
                Duration::new(1, 0),
                'h'.try_into().unwrap(),
                false
            ),]
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_key_stroke_index(1),
            vec![
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
            ]
        );
        assert_eq!(typed_chunk.wrong_key_strokes_of_key_stroke_index(2), vec![]);
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_key_stroke_index(3),
            vec![ActualKeyStroke::new(
                Duration::new(7, 0),
                'x'.try_into().unwrap(),
                false
            ),]
        );
        assert_eq!(typed_chunk.wrong_key_strokes_of_key_stroke_index(4), vec![]);
        assert_eq!(typed_chunk.wrong_key_strokes_of_key_stroke_index(5), vec![]);
        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_key_stroke_index(),
            vec![1, 2, 0, 1, 0]
        );
        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_key_stroke_index_only_confirmed(),
            vec![1, 2, 0, 1, 0]
        );

        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_ideal_key_stroke_index_only_confirmed(
                &gen_candidate!(gen_candidate_key_stroke!(["jo"]))
            ),
            vec![3, 1]
        );

        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::OnlyFirst),
            0
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::OnlyFirst),
            vec![]
        );
        assert_eq!(
            typed_chunk
                .wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::DoubleFirst),
            3
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::DoubleFirst),
            vec![
                ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
            ]
        );
        assert_eq!(
            typed_chunk
                .wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::DoubleSecond),
            1
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::DoubleSecond),
            vec![ActualKeyStroke::new(
                Duration::new(7, 0),
                'x'.try_into().unwrap(),
                false
            ),]
        );
    }

    #[test]
    fn wrong_positions_of_not_double_splitted() {
        let typed_chunk = gen_chunk_inflight!(
            "じょ",
            vec![gen_candidate!(gen_candidate_key_stroke!(["jo"])),],
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(5, 0), 'o'.try_into().unwrap(), true)
            ],
            2,
            []
        );

        assert_eq!(typed_chunk.wrong_key_strokes_of_key_stroke_index(0), vec![]);
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_key_stroke_index(1),
            vec![
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
            ]
        );
        assert_eq!(typed_chunk.wrong_key_strokes_of_key_stroke_index(2), vec![]);
        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_key_stroke_index(),
            vec![0, 3]
        );
        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_key_stroke_index_only_confirmed(),
            vec![0, 3]
        );

        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_each_ideal_key_stroke_index_only_confirmed(
                &gen_candidate!(gen_candidate_key_stroke!(["jo"]))
            ),
            vec![0, 3]
        );

        assert_eq!(
            typed_chunk.wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::OnlyFirst),
            3
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::OnlyFirst),
            vec![
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
            ]
        );
        assert_eq!(
            typed_chunk
                .wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::DoubleFirst),
            0
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::DoubleFirst),
            vec![]
        );
        assert_eq!(
            typed_chunk
                .wrong_key_stroke_count_of_chunk_element_index(ChunkElementIndex::DoubleSecond),
            0
        );
        assert_eq!(
            typed_chunk.wrong_key_strokes_of_chunk_element_index(ChunkElementIndex::DoubleSecond),
            vec![]
        );
    }
}
