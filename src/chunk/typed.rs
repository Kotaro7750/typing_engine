use std::time::Duration;

use crate::chunk::Chunk;
use crate::key_stroke::{ActualKeyStroke, KeyStrokeChar};

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

    // 現在タイピング中のチャンクに対して1キーストロークのタイプを行う
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert_eq!(self.state, TypedChunkState::Inflight);

        assert!(self.chunk.key_stroke_candidates().is_some());
        let key_stroke_candidates = self.chunk.key_stroke_candidates().as_mut().unwrap();

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
            let mut index = 0;
            key_stroke_candidates.retain(|_| {
                let is_hit = *candidate_hit_miss.get(index).unwrap();
                index += 1;
                is_hit
            });

            index = 0;
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

        // 確定した候補を探す
        if is_hit {
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

            if is_confirmed {
                assert!(key_stroke_candidates.len() == 1);
                self.state = TypedChunkState::Confirmed;
            }
        }

        if is_hit {
            KeyStrokeResult::Correct
        } else {
            KeyStrokeResult::Wrong
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
            state: TypedChunkState::Inflight,
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
        };

        let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                state: TypedChunkState::Inflight,
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
                )]
            }
        );

        let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(2, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Wrong);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                state: TypedChunkState::Inflight,
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
                ]
            }
        );

        let stroke_result = typed_chunk.stroke_key('o'.try_into().unwrap(), Duration::new(3, 0));
        assert_eq!(stroke_result, KeyStrokeResult::Correct);

        assert_eq!(
            typed_chunk,
            TypedChunk {
                state: TypedChunkState::Confirmed,
                chunk: gen_chunk!("じょ", vec![gen_candidate!(["jo"]),]),
                cursor_positions_of_candidates: vec![2],
                key_strokes: vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'o'.try_into().unwrap(), true)
                ]
            }
        );
    }
}
