use std::collections::VecDeque;
use std::time::Duration;

use crate::chunk::typed::{KeyStrokeResult, TypedChunk};
use crate::chunk::Chunk;
use crate::key_stroke::KeyStrokeChar;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct ProcessedChunkInfo {
    unprocessed_chunks: VecDeque<Chunk>,
    inflight_chunk: Option<TypedChunk>,
    confirmed_chunks: Vec<TypedChunk>,
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

            let next_chunk_head_constraint = current_inflight_chunk.next_chunk_head_constraint();
            self.confirmed_chunks.push(current_inflight_chunk);

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
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::chunk::typed::TypedChunkState;
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
                confirmed_chunks: vec![TypedChunk::new(
                    TypedChunkState::Confirmed,
                    gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                    vec![1],
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
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![1],
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![1],
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
                    TypedChunkState::Inflight,
                    gen_chunk!("う", vec![gen_candidate!(["wu"]), gen_candidate!(["whu"])]).into(),
                    vec![1, 1],
                    vec![ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'w'.try_into().unwrap(),
                        true
                    )]
                )),
                confirmed_chunks: vec![
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![1],
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![1],
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
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![1],
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![1],
                        vec![ActualKeyStroke::new(
                            Duration::new(2, 0),
                            'w'.try_into().unwrap(),
                            true
                        )],
                    ),
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("う", vec![gen_candidate!(["wu"])]),
                        vec![2],
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
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("う", vec![gen_candidate!(["u"]),]),
                        vec![1],
                        vec![ActualKeyStroke::new(
                            Duration::new(1, 0),
                            'u'.try_into().unwrap(),
                            true
                        )],
                    ),
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("っ", vec![gen_candidate!(["w"], 'w'),]),
                        vec![1],
                        vec![ActualKeyStroke::new(
                            Duration::new(2, 0),
                            'w'.try_into().unwrap(),
                            true
                        )],
                    ),
                    TypedChunk::new(
                        TypedChunkState::Confirmed,
                        gen_chunk!("う", vec![gen_candidate!(["wu"])]),
                        vec![2],
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
}
