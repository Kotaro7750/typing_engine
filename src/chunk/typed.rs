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

    pub(crate) fn actual_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.key_strokes
    }

    pub(crate) fn pending_key_strokes(&self) -> &[ActualKeyStroke] {
        &self.pending_key_strokes
    }

    /// チャンクが確定したか
    /// 遅延確定候補自体を打ち終えても確定自体はまだのとき確定としてはいけない
    pub(crate) fn is_confirmed(&mut self) -> bool {
        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        // 確定している条件は
        // * 候補が1つである
        // * その候補を打ち終えている

        if key_stroke_candidates.len() != 1 {
            return false;
        }

        let mut is_confirmed = false;

        key_stroke_candidates
            .iter()
            .zip(&self.cursor_positions_of_candidates)
            .for_each(|(candidate, cursor_position)| {
                if *cursor_position >= candidate.calc_key_stroke_count() {
                    assert!(!is_confirmed);

                    is_confirmed = true;
                }
            });

        is_confirmed
    }

    /// 遅延確定候補があるとしたらそれを打ち終えているかどうか
    /// ないときには常にfalseを返す
    pub(crate) fn is_delayed_confirmable(&self) -> bool {
        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        let mut is_delayed_confirmable = false;

        key_stroke_candidates
            .iter()
            .zip(&self.cursor_positions_of_candidates)
            .filter(|(candidate, _)| candidate.is_delayed_confirmed_candidate())
            .for_each(|(candidate, cursor_position)| {
                if *cursor_position >= candidate.calc_key_stroke_count() {
                    // 同時に２つの遅延確定候補が終了することはないはずである
                    assert!(!is_delayed_confirmable);

                    is_delayed_confirmable = true;
                }
            });

        is_delayed_confirmable
    }

    /// 遅延確定候補のために保持しているキーストロークの中にミスタイプがあるかどうか
    pub(crate) fn has_wrong_stroke_in_pending_key_strokes(&self) -> bool {
        self.pending_key_strokes
            .iter()
            .map(|actual_key_stroke| !actual_key_stroke.is_correct())
            .reduce(|accum, is_correct| accum || is_correct)
            .map_or(false, |r| r)
    }

    /// 現在タイピング中のチャンクに対して1キーストロークのタイプを行う
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());

        // 前回のキーストロークよりも時間的に後でなくてはならない
        if let Some(last_key_stroke) = self.key_strokes.last() {
            assert!(&elapsed_time >= last_key_stroke.elapsed_time());
        }

        // 打ち終えた遅延確定候補がある場合とそうでない場合で処理を分ける
        let key_stroke_result = if self.is_delayed_confirmable() {
            self.stroke_key_to_delayed_confirmable(key_stroke, elapsed_time)
        } else {
            self.stroke_key_to_no_delayed_confirmable(key_stroke, elapsed_time)
        };

        // 遅延確定候補以外の候補で確定した場合にはpendingしていたキーストロークを加える必要がある
        if self.is_confirmed() && !self.is_delayed_confirmable() {
            self.pending_key_strokes
                .drain(..)
                .for_each(|key_stroke| self.key_strokes.push(key_stroke));
        }

        key_stroke_result
    }

    /// 遅延確定候補ではないかそうであってもまだ打ち終えていないチャンクを対象にキーストロークを行う
    fn stroke_key_to_no_delayed_confirmable(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());
        assert!(!self.is_delayed_confirmable());

        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        assert_eq!(
            key_stroke_candidates.len(),
            self.cursor_positions_of_candidates.len()
        );

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

    /// 遅延確定候補を持つその候補を打ち終えたチャンクに対してキーストロークを行う
    fn stroke_key_to_delayed_confirmable(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(!self.is_confirmed());
        assert!(self.is_delayed_confirmable());

        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_ref().unwrap();

        assert_eq!(
            key_stroke_candidates.len(),
            self.cursor_positions_of_candidates.len()
        );

        // 打ち終えている遅延確定候補がある場合にはキーストロークが有効かの比較は遅延確定候補とそうでない候補で比較の仕方が異なる
        // 遅延確定候補の比較は次のチャンク先頭との比較で行う
        // そうでない候補の比較は通常のやり方と同じである

        let delayed_confirmed_candidate_index = key_stroke_candidates
            .iter()
            .position(|candidate| candidate.is_delayed_confirmed_candidate())
            .unwrap();

        // 次のチャンク先頭にヒットするなら遅延確定候補で確定する
        if key_stroke_candidates
            .get(delayed_confirmed_candidate_index)
            .unwrap()
            .delayed_confirmed_candiate_info()
            .as_ref()
            .unwrap()
            .is_valid_key_stroke(key_stroke.clone())
        {
            // 遅延確定候補以外の候補を削除する
            let mut candidate_reduce_vec = vec![false; key_stroke_candidates.len()];
            candidate_reduce_vec[delayed_confirmed_candidate_index] = true;

            self.chunk.reduce_candidate(&candidate_reduce_vec);

            // 遅延確定候補以外のカーソル位置も削除する
            let mut index = 0;
            self.cursor_positions_of_candidates.retain(|_| {
                let is_hit = *candidate_reduce_vec.get(index).unwrap();
                index += 1;
                is_hit
            });

            self.pending_key_strokes
                .push(ActualKeyStroke::new(elapsed_time, key_stroke, true));

            return KeyStrokeResult::Correct;
        }

        // それぞれの候補においてタイプされたキーストロークが有効かどうか
        let candidate_hit_miss: Vec<bool> = key_stroke_candidates
            .iter()
            .zip(self.cursor_positions_of_candidates.iter())
            .map(|(candidate, cursor_position)| {
                // 遅延確定候補は既にミスであることが確定している
                if candidate.is_delayed_confirmed_candidate() {
                    false
                } else {
                    candidate.key_stroke_char_at_position(*cursor_position) == key_stroke
                }
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

        self.pending_key_strokes
            .push(ActualKeyStroke::new(elapsed_time, key_stroke, is_hit));

        if is_hit {
            KeyStrokeResult::Correct
        } else {
            KeyStrokeResult::Wrong
        }
    }

    pub(crate) fn take_pending_key_strokes(&mut self) -> Vec<ActualKeyStroke> {
        self.pending_key_strokes.drain(..).collect()
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

    // 確定したキーストロークのそれぞれの位置は綴り末尾かどうか
    // もし末尾だったとしたら何個の綴りの末尾かどうか
    pub(crate) fn construct_spell_end_vector(&self) -> Vec<Option<usize>> {
        let mut spell_end_vector = vec![None; self.key_strokes.len()];
        let confirmed_candidate = self.chunk.min_candidate(None);

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
                ],
                gen_candidate!(["jo"])
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
                    ],
                    gen_candidate!(["jo"])
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
                    ],
                    gen_candidate!(["jo"])
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
                chunk: gen_chunk!("じょ", vec![gen_candidate!(["jo"])], gen_candidate!(["jo"])),
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
                ],
                gen_candidate!(["n"], ['j'])
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
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                    gen_candidate!(["n"], ['j'])
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
        assert!(typed_chunk.is_delayed_confirmable());

        let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Wrong);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                    gen_candidate!(["n"], ['j'])
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
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["nn"])],
                    gen_candidate!(["n"], ['j'])
                ),
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
                ],
                gen_candidate!(["n"], ['j'])
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
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                    gen_candidate!(["n"], ['j'])
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
        assert!(typed_chunk.is_delayed_confirmable());

        let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Wrong);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                    gen_candidate!(["n"], ['j'])
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
                ),]
            }
        );

        let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(3, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                chunk: gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['j'])],
                    gen_candidate!(["n"], ['j'])
                ),
                cursor_positions_of_candidates: vec![1],
                key_strokes: vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                pending_key_strokes: vec![
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), true)
                ]
            }
        );

        assert!(typed_chunk.is_confirmed());
    }
}
