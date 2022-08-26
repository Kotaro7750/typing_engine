use std::time::Duration;

use crate::chunk::Chunk;
use crate::key_stroke::{ActualKeyStroke, KeyStrokeChar};

use super::confirmed::ConfirmedChunk;

// 現在打たれているチャンク
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct TypedChunk {
    chunk: Chunk,
    // キーストローク候補のそれぞれに対するカーソル位置
    cursor_positions_of_candidates: Vec<usize>,
    // ミスタイプも含めた実際のキーストローク
    key_strokes: Vec<ActualKeyStroke>,
    // 遅延確定候補がある場合にはキーストロークが最終的にこのチャンクに属するのか次のチャンクに属するのかが確定しないのでそれを一時的に保持しておく
    pending_key_strokes: Vec<ActualKeyStroke>,
}

impl TypedChunk {
    #[cfg(test)]
    pub(crate) fn new(
        chunk: Chunk,
        cursor_positions_of_candidates: Vec<usize>,
        key_strokes: Vec<ActualKeyStroke>,
        pending_key_strokes: Vec<ActualKeyStroke>,
    ) -> Self {
        Self {
            chunk,
            cursor_positions_of_candidates,
            key_strokes,
            pending_key_strokes,
        }
    }

    pub(crate) fn is_confirmed(&mut self) -> bool {
        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        let mut is_confirmed = false;

        key_stroke_candidates
            .iter()
            .zip(&self.cursor_positions_of_candidates)
            .for_each(|(candidate, cursor_position)| {
                // 同時に２つの候補が終了することはないはずである
                assert!(!is_confirmed);

                if *cursor_position >= candidate.calc_key_stroke_count() {
                    is_confirmed = true;
                }
            });

        is_confirmed
    }

    // 現在タイピング中のチャンクに対して1キーストロークのタイプを行う
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());

        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        assert_eq!(
            key_stroke_candidates.len(),
            self.cursor_positions_of_candidates.len()
        );

        // 前回のキーストロークよりも時間的に後でなくてはならない
        if let Some(last_key_stroke) = self.key_strokes.last() {
            assert!(&elapsed_time >= last_key_stroke.elapsed_time());
        }

        // それぞれの候補においてタイプされたキーストロークが有効かどうか
        let candidate_hit_miss: Vec<bool> = key_stroke_candidates
            .iter()
            .zip(self.cursor_positions_of_candidates.iter())
            .map(|(candidate, cursor_position)| {
                candidate.key_stroke_char_at_position(*cursor_position) == key_stroke
            })
            .collect();

        let is_hit = candidate_hit_miss.contains(&true);

        // 何かしらの候補についてキーストロークが有効だったらそれらの候補のみを残しカーソル位置を進める
        if is_hit {
            self.chunk.reduce_candidate(&candidate_hit_miss);

            let mut index = 0;
            self.cursor_positions_of_candidates.retain(|_| {
                let is_hit = *candidate_hit_miss.get(index).unwrap();
                index += 1;
                is_hit
            });

            self.cursor_positions_of_candidates
                .iter_mut()
                .for_each(|cursor_position| *cursor_position += 1);
        }

        self.key_strokes
            .push(ActualKeyStroke::new(elapsed_time, key_stroke, is_hit));

        if is_hit {
            KeyStrokeResult::Correct
        } else {
            KeyStrokeResult::Wrong
        }
    }

    // チャンクの綴りのそれぞれ（基本的には1つだが複数文字を個別で打った場合には2つ）でミスタイプがあったかどうか
    pub(crate) fn construct_wrong_spell_element_vector(&self) -> Vec<bool> {
        let element_count = if self.as_ref().min_candidate(None).is_splitted() {
            2
        } else {
            1
        };

        // 複数文字のチャンクを個別で打った場合には要素数は2になる
        let mut wrong_spell_element_vector: Vec<bool> = vec![false; element_count];

        // 打たれたキーストロークではなく候補中のインデックス
        let mut current_key_stroke_index = 0;

        self.key_strokes.iter().for_each(|actual_key_stroke| {
            if actual_key_stroke.is_correct() {
                current_key_stroke_index += 1;
            } else {
                wrong_spell_element_vector[self
                    .as_ref()
                    .min_candidate(None)
                    // キーストロークに対応する位置に変換する
                    .element_index_at_key_stroke_index(current_key_stroke_index)] = true;
            }
        });

        wrong_spell_element_vector
    }

    // チャンクのキーストロークのそれぞれでミスタイプがあったかどうか
    pub(crate) fn construct_wrong_key_stroke_vector(&self) -> Vec<bool> {
        let mut wrong_key_stroke_vector =
            vec![false; self.chunk.min_candidate(None).calc_key_stroke_count()];

        // 打たれたキーストロークではなく候補中のインデックス
        let mut current_key_stroke_index = 0;

        self.key_strokes.iter().for_each(|actual_key_stroke| {
            if actual_key_stroke.is_correct() {
                current_key_stroke_index += 1;
            } else {
                wrong_key_stroke_vector[current_key_stroke_index] = true;
            }
        });

        wrong_key_stroke_vector
    }

    // チャンクのキーストロークのどこにカーソルを当てるべきか
    pub(crate) fn current_key_stroke_cursor_position(&self) -> usize {
        *self
            .cursor_positions_of_candidates
            .iter()
            .reduce(|cursor_position, cursor_position_of_candidate| {
                // XXX 適切に候補を削減していれば全ての候補でカーソル位置は同じなはず
                assert!(cursor_position == cursor_position_of_candidate);
                cursor_position_of_candidate
            })
            .unwrap()
    }

    // チャンクの綴りのどこにカーソルを当てるべきか
    // 基本的にはチャンク全体だが複数文字を個別で入力している場合にはそれぞれの文字になる
    pub(crate) fn current_spell_cursor_positions(&self) -> Vec<usize> {
        let mut cursor_positions: Vec<usize> = vec![];

        if self.as_ref().min_candidate(None).is_splitted() {
            // 複数文字チャンクをまとめて入力する場合には現在入力中の綴りのみにカーソルを当てる
            cursor_positions.push(
                self.as_ref()
                    .min_candidate(None)
                    .element_index_at_key_stroke_index(self.current_key_stroke_cursor_position()),
            );
        } else {
            // チャンクをまとめて入力している場合にはチャンクの綴り全体にカーソルを当てる
            self.as_ref()
                .spell()
                .as_ref()
                .chars()
                .enumerate()
                .for_each(|(i, _)| {
                    cursor_positions.push(i);
                });
        }

        cursor_positions
    }
}

impl From<Chunk> for TypedChunk {
    fn from(chunk: Chunk) -> Self {
        let key_stroke_candidates_count = match chunk.key_stroke_candidates_count() {
            Some(c) => c,
            None => panic!(),
        };

        Self {
            chunk,
            cursor_positions_of_candidates: vec![0; key_stroke_candidates_count],
            key_strokes: vec![],
            pending_key_strokes: vec![],
        }
    }
}

impl Into<ConfirmedChunk> for TypedChunk {
    fn into(self) -> ConfirmedChunk {
        ConfirmedChunk::new(self.chunk, self.key_strokes)
    }
}

impl AsRef<Chunk> for TypedChunk {
    fn as_ref(&self) -> &Chunk {
        &self.chunk
    }
}

// キーストロークの結果
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum KeyStrokeResult {
    // キーストロークが正しかったケース
    Correct,
    // キーストロークが間違っていたケース
    Wrong,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{gen_candidate, gen_chunk};

    #[test]
    fn stroke_key_1() {
        let mut typed_chunk = TypedChunk {
            chunk: gen_chunk!(
                "じょ",
                vec![
                    gen_candidate!(["jo"]),
                    gen_candidate!(["zyo"]),
                    gen_candidate!(["jyo"]),
                    gen_candidate!(["zi", "lyo"]),
                    gen_candidate!(["zi", "xyo"]),
                    gen_candidate!(["ji", "lyo"]),
                    gen_candidate!(["ji", "xyo"]),
                ]
            ),
            cursor_positions_of_candidates: vec![0; 7],
            key_strokes: vec![],
            pending_key_strokes: vec![],
        };

        let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "じょ",
                    vec![
                        gen_candidate!(["jo"]),
                        gen_candidate!(["jyo"]),
                        gen_candidate!(["ji", "lyo"]),
                        gen_candidate!(["ji", "xyo"]),
                    ]
                ),
                cursor_positions_of_candidates: vec![1; 4],
                key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'j'.try_into().unwrap(),
                    true
                )],
                pending_key_strokes: vec![],
            }
        );

        let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(2, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Wrong);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "じょ",
                    vec![
                        gen_candidate!(["jo"]),
                        gen_candidate!(["jyo"]),
                        gen_candidate!(["ji", "lyo"]),
                        gen_candidate!(["ji", "xyo"]),
                    ]
                ),
                cursor_positions_of_candidates: vec![1; 4],
                key_strokes: vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false)
                ],
                pending_key_strokes: vec![],
            }
        );

        let stroke_result = typed_chunk.stroke_key('o'.try_into().unwrap(), Duration::new(3, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!("じょ", vec![gen_candidate!(["jo"]),]),
                cursor_positions_of_candidates: vec![2],
                key_strokes: vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'o'.try_into().unwrap(), true)
                ],
                pending_key_strokes: vec![],
            }
        );
    }

    #[test]
    fn stroke_key_2() {
        let mut typed_chunk = TypedChunk {
            chunk: gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["n"], ['j']),
                    gen_candidate!(["nn"]),
                    gen_candidate!(["xn"]),
                ]
            ),
            cursor_positions_of_candidates: vec![0; 3],
            key_strokes: vec![],
            pending_key_strokes: vec![],
        };

        let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])]
                ),
                cursor_positions_of_candidates: vec![1, 1],
                key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                pending_key_strokes: vec![]
            }
        );

        assert!(!typed_chunk.is_confirmed());

        let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Wrong);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])]
                ),
                cursor_positions_of_candidates: vec![1, 1],
                key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                pending_key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(2, 0),
                    'm'.try_into().unwrap(),
                    false
                )]
            }
        );

        let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!("ん", vec![gen_candidate!(["nn"])]),
                cursor_positions_of_candidates: vec![2],
                key_strokes: vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'n'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                ],
                pending_key_strokes: vec![]
            }
        );

        assert!(typed_chunk.is_confirmed());
    }

    #[test]
    fn stroke_key_3() {
        let mut typed_chunk = TypedChunk {
            chunk: gen_chunk!(
                "ん",
                vec![
                    gen_candidate!(["n"], ['j']),
                    gen_candidate!(["nn"]),
                    gen_candidate!(["xn"]),
                ]
            ),
            cursor_positions_of_candidates: vec![0; 3],
            key_strokes: vec![],
            pending_key_strokes: vec![],
        };

        let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])]
                ),
                cursor_positions_of_candidates: vec![1, 1],
                key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                pending_key_strokes: vec![]
            }
        );

        assert!(!typed_chunk.is_confirmed());

        let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Wrong);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])]
                ),
                cursor_positions_of_candidates: vec![1, 1],
                key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                pending_key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(2, 0),
                    'm'.try_into().unwrap(),
                    false
                )]
            }
        );

        let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(3, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!("ん", vec![gen_candidate!(["n"], ['j'])]),
                cursor_positions_of_candidates: vec![1],
                key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                pending_key_strokes: vec![]
            }
        );

        assert!(typed_chunk.is_confirmed());
    }
}
