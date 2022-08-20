use std::collections::VecDeque;
use std::time::Duration;

use crate::chunk::confirmed::ConfirmedChunk;
use crate::chunk::typed::{KeyStrokeResult, TypedChunk};
use crate::chunk::Chunk;
use crate::display_info::{KeyStrokeDisplayInfo, SpellDisplayInfo};
use crate::key_stroke::KeyStrokeChar;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ProcessedChunkInfo {
    unprocessed_chunks: VecDeque<Chunk>,
    inflight_chunk: Option<TypedChunk>,
    confirmed_chunks: Vec<ConfirmedChunk>,
}

impl ProcessedChunkInfo {
    pub(crate) fn new(chunks: Vec<Chunk>) -> Self {
        Self {
            unprocessed_chunks: chunks.into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
        }
    }

    pub(crate) fn is_finished(&self) -> bool {
        // 処理すべきチャンクがない場合には終了である
        self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none()
    }

    pub(crate) fn append_chunks(&mut self, chunks: Vec<Chunk>) {
        let mut chunks: VecDeque<Chunk> = chunks.into();

        // 終了している状態で追加されたら先頭のチャンクを処理中にする必要がある
        if self.unprocessed_chunks.is_empty() && self.inflight_chunk.is_none() {
            self.inflight_chunk
                .replace(chunks.pop_front().unwrap().into());
        }

        self.unprocessed_chunks.append(&mut (chunks.into()));
    }

    // 現在打っているチャンクを確定させ未処理のチャンク列の先頭のチャンクの処理を開始する
    pub(crate) fn move_next_chunk(&mut self) {
        // まずは現在打っているチャンクを確定済みチャンク列に追加する
        let next_chunk_head_constraint = if self.inflight_chunk.is_some() {
            let mut current_inflight_chunk = self.inflight_chunk.take().unwrap();
            assert!(current_inflight_chunk.is_confirmed());

            let mut current_confirmed_chunk: ConfirmedChunk = current_inflight_chunk.into();
            let next_chunk_head_constraint = current_confirmed_chunk.next_chunk_head_constraint();
            self.confirmed_chunks.push(current_confirmed_chunk);

            next_chunk_head_constraint
        } else {
            None
        };

        assert!(self.inflight_chunk.is_none());

        // 未処理チャンク列の先頭チャンクを処理中のチャンクにする
        if let Some(mut next_inflight_chunk) = self.unprocessed_chunks.pop_front() {
            if let Some(next_chunk_head_constraint) = next_chunk_head_constraint {
                next_inflight_chunk.strict_chunk_head(next_chunk_head_constraint);
            }

            self.inflight_chunk.replace(next_inflight_chunk.into());
        }
    }

    // 1タイプのキーストロークを与える
    pub(crate) fn stroke_key(
        &mut self,
        key_stroke: KeyStrokeChar,
        elapsed_time: Duration,
    ) -> KeyStrokeResult {
        assert!(self.inflight_chunk.is_some());

        let inflight_chunk = self.inflight_chunk.as_mut().unwrap();
        let result = inflight_chunk.stroke_key(key_stroke, elapsed_time);

        // このキーストロークでチャンクが確定したら次のチャンクの処理に移る
        if inflight_chunk.is_confirmed() {
            self.move_next_chunk();
        }

        result
    }

    pub(crate) fn construct_display_info(&self) -> (SpellDisplayInfo, KeyStrokeDisplayInfo) {
        let mut spell = String::new();
        let mut spell_head_position = 0;
        let mut spell_cursor_positions = Vec::<usize>::new();
        let mut spell_wrong_positions: Vec<usize> = vec![];

        let mut key_stroke = String::new();
        let mut key_stroke_cursor_position = 0;
        let mut key_stroke_wrong_positions: Vec<usize> = vec![];

        // 1. 確定したチャンク
        // 2. タイプ中のチャンク
        // 3. 未処理のチャンク
        //
        // という順番で表示用の情報を構築する

        // 1. 確定したチャンク
        self.confirmed_chunks.iter().for_each(|confirmed_chunk| {
            // キーストロークのそれぞれがタイプミスかどうか
            confirmed_chunk
                .construct_key_stroke_wrong_vector()
                .iter()
                .for_each(|is_key_stroke_wrong| {
                    if *is_key_stroke_wrong {
                        key_stroke_wrong_positions.push(key_stroke_cursor_position);
                    }

                    key_stroke_cursor_position += 1;
                });

            key_stroke.push_str(&confirmed_chunk.confirmed_candidate().whole_key_stroke());

            // 綴り要素のそれぞれがタイプミスかどうか
            let wrong_stroke_vector = confirmed_chunk.construct_wrong_stroke_vector();

            // 複数文字チャンクを個別に入力した場合はそれぞれの綴りについて
            // それ以外ではチャンク全体の綴りについて
            // タイプミス判定をする
            confirmed_chunk
                .as_ref()
                .spell()
                .as_ref()
                .chars()
                .enumerate()
                .for_each(|(i, _)| {
                    let element_index = if wrong_stroke_vector.len() == 1 { 0 } else { i };

                    if wrong_stroke_vector[element_index] {
                        spell_wrong_positions.push(spell_head_position);
                    }

                    spell_head_position += 1;
                });

            spell.push_str(confirmed_chunk.as_ref().spell().as_ref());
        });

        // 2. タイプ中のチャンク
        if self.inflight_chunk.is_none() {
            spell_cursor_positions = vec![spell_head_position];
            assert!(self.is_finished());
        } else {
            let inflight_chunk = self.inflight_chunk.as_ref().unwrap();

            // キーストローク

            let in_chunk_current_key_stroke_cursor_position =
                inflight_chunk.current_key_stroke_cursor_position();

            inflight_chunk
                .construct_wrong_key_stroke_vector()
                .iter()
                .enumerate()
                .for_each(|(i, is_key_stroke_wrong)| {
                    if *is_key_stroke_wrong {
                        key_stroke_wrong_positions.push(key_stroke_cursor_position + i);
                    }
                });

            // この時点ではカーソル位置はこのチャンクの先頭を指しているので単純に足すだけで良い
            key_stroke_cursor_position += in_chunk_current_key_stroke_cursor_position;

            key_stroke.push_str(&inflight_chunk.as_ref().min_candidate().whole_key_stroke());

            // 綴り

            // カーソル位置は複数ある場合がある
            let in_chunk_current_spell_cursor_positions =
                inflight_chunk.current_spell_cursor_positions();

            spell_cursor_positions = in_chunk_current_spell_cursor_positions
                .iter()
                .map(|in_chunk_current_spell_cursor_position| {
                    spell_head_position + in_chunk_current_spell_cursor_position
                })
                .collect();

            let wrong_spell_element_vector = inflight_chunk.construct_wrong_spell_element_vector();

            // 複数文字チャンクを個別に入力した場合はそれぞれの綴りについて
            // それ以外ではチャンク全体の綴りについて
            // タイプミス判定をする
            inflight_chunk
                .as_ref()
                .spell()
                .as_ref()
                .chars()
                .enumerate()
                .for_each(|(i, _)| {
                    let element_index = if wrong_spell_element_vector.len() == 1 {
                        0
                    } else {
                        i
                    };

                    if wrong_spell_element_vector[element_index] {
                        spell_wrong_positions.push(spell_head_position);
                    }

                    spell_head_position += 1;
                });

            spell.push_str(inflight_chunk.as_ref().spell().as_ref());
        }

        (
            SpellDisplayInfo::new(
                spell,
                spell_cursor_positions,
                spell_wrong_positions,
                spell_head_position - 1,
            ),
            KeyStrokeDisplayInfo::new(
                key_stroke,
                key_stroke_cursor_position,
                key_stroke_wrong_positions,
            ),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::key_stroke::ActualKeyStroke;
    use crate::{gen_candidate, gen_chunk};

    #[test]
    fn stroke_key_1() {
        // 1. 初期化
        let mut pci = ProcessedChunkInfo::new(vec![
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"]),
                    gen_candidate!(["wu"]),
                    gen_candidate!(["whu"])
                ]
            ),
            gen_chunk!(
                "っ",
                vec![
                    gen_candidate!(["w"], 'w'),
                    gen_candidate!(["ltu"]),
                    gen_candidate!(["xtu"]),
                    gen_candidate!(["ltsu"])
                ]
            ),
            gen_chunk!(
                "う",
                vec![
                    gen_candidate!(["u"]),
                    gen_candidate!(["wu"]),
                    gen_candidate!(["whu"])
                ]
            ),
        ]);

        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"]),
                            gen_candidate!(["wu"]),
                            gen_candidate!(["whu"])
                        ]
                    ),
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["w"], 'w'),
                            gen_candidate!(["ltu"]),
                            gen_candidate!(["xtu"]),
                            gen_candidate!(["ltsu"])
                        ]
                    ),
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"]),
                            gen_candidate!(["wu"]),
                            gen_candidate!(["whu"])
                        ]
                    ),
                ]
                .into(),
                inflight_chunk: None,
                confirmed_chunks: vec![],
            }
        );

        // 2. タイピング開始
        pci.move_next_chunk();
        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["w"], 'w'),
                            gen_candidate!(["ltu"]),
                            gen_candidate!(["xtu"]),
                            gen_candidate!(["ltsu"])
                        ]
                    ),
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"]),
                            gen_candidate!(["wu"]),
                            gen_candidate!(["whu"])
                        ]
                    ),
                ]
                .into(),
                inflight_chunk: Some(
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"]),
                            gen_candidate!(["wu"]),
                            gen_candidate!(["whu"])
                        ]
                    )
                    .into()
                ),
                confirmed_chunks: vec![],
            }
        );

        // 3. 「u」と入力
        pci.stroke_key('u'.try_into().unwrap(), Duration::new(1, 0));
        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"]),
                        gen_candidate!(["wu"]),
                        gen_candidate!(["whu"])
                    ]
                ),]
                .into(),
                inflight_chunk: Some(
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["w"], 'w'),
                            gen_candidate!(["ltu"]),
                            gen_candidate!(["xtu"]),
                            gen_candidate!(["ltsu"])
                        ]
                    )
                    .into()
                ),
                confirmed_chunks: vec![ConfirmedChunk::new(
                    gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                )],
            }
        );

        // 3. 「w」と入力
        pci.stroke_key('w'.try_into().unwrap(), Duration::new(2, 0));
        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![].into(),
                inflight_chunk: Some(
                    gen_chunk!("う", vec![gen_candidate!(["wu"]), gen_candidate!(["whu"])]).into()
                ),
                confirmed_chunks: vec![
                    ConfirmedChunk::new(
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    ConfirmedChunk::new(
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(2, 0),
                            'w'.try_into().unwrap(),
                            true
                        )],
                    )
                ],
            }
        );

        // 4. 「w」と入力
        pci.stroke_key('w'.try_into().unwrap(), Duration::new(3, 0));
        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![].into(),
                inflight_chunk: Some(TypedChunk::new(
                    gen_chunk!("う", vec![gen_candidate!(["wu"]), gen_candidate!(["whu"])]).into(),
                    vec![1, 1],
                    vec![ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'w'.try_into().unwrap(),
                        true
                    )]
                )),
                confirmed_chunks: vec![
                    ConfirmedChunk::new(
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    ConfirmedChunk::new(
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(2, 0),
                            'w'.try_into().unwrap(),
                            true
                        )],
                    )
                ],
            }
        );

        // 5. 「u」と入力
        pci.stroke_key('u'.try_into().unwrap(), Duration::new(4, 0));
        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![].into(),
                inflight_chunk: None,
                confirmed_chunks: vec![
                    ConfirmedChunk::new(
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    ConfirmedChunk::new(
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(2, 0),
                            'w'.try_into().unwrap(),
                            true
                        )],
                    ),
                    ConfirmedChunk::new(
                        gen_chunk!("う", vec![gen_candidate!(["wu"])]),
                        vec![
                            ActualKeyStroke::new(
                                Duration::new(3, 0),
                                'w'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(4, 0),
                                'u'.try_into().unwrap(),
                                true
                            ),
                        ]
                    )
                ],
            }
        );

        assert!(pci.is_finished());

        pci.append_chunks(vec![gen_chunk!(
            "う",
            vec![
                gen_candidate!(["u"]),
                gen_candidate!(["wu"]),
                gen_candidate!(["whu"])
            ]
        )]);

        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![].into(),
                inflight_chunk: Some(
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"]),
                            gen_candidate!(["wu"]),
                            gen_candidate!(["whu"])
                        ]
                    )
                    .into()
                ),
                confirmed_chunks: vec![
                    ConfirmedChunk::new(
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    ConfirmedChunk::new(
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![ActualKeyStroke::new(
                            Duration::new(2, 0),
                            'w'.try_into().unwrap(),
                            true
                        )],
                    ),
                    ConfirmedChunk::new(
                        gen_chunk!("う", vec![gen_candidate!(["wu"])]),
                        vec![
                            ActualKeyStroke::new(
                                Duration::new(3, 0),
                                'w'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(4, 0),
                                'u'.try_into().unwrap(),
                                true
                            ),
                        ]
                    )
                ],
            }
        );
    }

    #[test]
    fn construct_display_info_1() {
        // 1. 初期化
        let mut pci = ProcessedChunkInfo::new(vec![
            gen_chunk!(
                "きょ",
                vec![
                    gen_candidate!(["kyo"]),
                    gen_candidate!(["ki", "lyo"]),
                    gen_candidate!(["ki", "xyo"])
                ]
            ),
            gen_chunk!(
                "きょ",
                vec![
                    gen_candidate!(["kyo"]),
                    gen_candidate!(["ki", "lyo"]),
                    gen_candidate!(["ki", "xyo"])
                ]
            ),
            gen_chunk!(
                "きょ",
                vec![
                    gen_candidate!(["kyo"]),
                    gen_candidate!(["ki", "lyo"]),
                    gen_candidate!(["ki", "xyo"])
                ]
            ),
        ]);

        // 2. タイピング開始
        pci.move_next_chunk();

        // 3. k -> u(ミスタイプ) -> y -> o -> k -> i -> j(ミスタイプ) -> x -> y -> o -> c(ミスタイプ) -> k という順で入力
        pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
        pci.stroke_key('u'.try_into().unwrap(), Duration::new(2, 0));
        pci.stroke_key('y'.try_into().unwrap(), Duration::new(3, 0));
        pci.stroke_key('o'.try_into().unwrap(), Duration::new(4, 0));
        pci.stroke_key('k'.try_into().unwrap(), Duration::new(5, 0));
        pci.stroke_key('i'.try_into().unwrap(), Duration::new(6, 0));
        pci.stroke_key('j'.try_into().unwrap(), Duration::new(7, 0));
        pci.stroke_key('x'.try_into().unwrap(), Duration::new(8, 0));
        pci.stroke_key('y'.try_into().unwrap(), Duration::new(9, 0));
        pci.stroke_key('o'.try_into().unwrap(), Duration::new(10, 0));
        pci.stroke_key('c'.try_into().unwrap(), Duration::new(11, 0));
        pci.stroke_key('k'.try_into().unwrap(), Duration::new(12, 0));

        assert_eq!(
            pci,
            ProcessedChunkInfo {
                unprocessed_chunks: vec![].into(),
                inflight_chunk: Some(TypedChunk::new(
                    gen_chunk!(
                        "きょ",
                        vec![
                            gen_candidate!(["kyo"]),
                            gen_candidate!(["ki", "lyo"]),
                            gen_candidate!(["ki", "xyo"]),
                        ]
                    ),
                    vec![1, 1, 1],
                    vec![
                        ActualKeyStroke::new(Duration::new(11, 0), 'c'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(12, 0), 'k'.try_into().unwrap(), true),
                    ]
                )),
                confirmed_chunks: vec![
                    ConfirmedChunk::new(
                        gen_chunk!("きょ", vec![gen_candidate!(["kyo"]),]),
                        vec![
                            ActualKeyStroke::new(
                                Duration::new(1, 0),
                                'k'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(2, 0),
                                'u'.try_into().unwrap(),
                                false
                            ),
                            ActualKeyStroke::new(
                                Duration::new(3, 0),
                                'y'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(4, 0),
                                'o'.try_into().unwrap(),
                                true
                            )
                        ],
                    ),
                    ConfirmedChunk::new(
                        gen_chunk!("きょ", vec![gen_candidate!(["ki", "xyo"]),]),
                        vec![
                            ActualKeyStroke::new(
                                Duration::new(5, 0),
                                'k'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(6, 0),
                                'i'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(7, 0),
                                'j'.try_into().unwrap(),
                                false
                            ),
                            ActualKeyStroke::new(
                                Duration::new(8, 0),
                                'x'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(9, 0),
                                'y'.try_into().unwrap(),
                                true
                            ),
                            ActualKeyStroke::new(
                                Duration::new(10, 0),
                                'o'.try_into().unwrap(),
                                true
                            )
                        ],
                    ),
                ],
            }
        );

        let (sdi, ksdi) = pci.construct_display_info();

        assert_eq!(
            sdi,
            SpellDisplayInfo::new(
                "きょきょきょ".to_string(),
                vec![4, 5],
                vec![0, 1, 3, 4, 5],
                5
            )
        );

        assert_eq!(
            ksdi,
            KeyStrokeDisplayInfo::new("kyokixyokyo".to_string(), 9, vec![1, 5, 8])
        );
    }
}
