use super::*;

use std::num::NonZeroUsize;
use std::time::Duration;

use crate::statistics::OnTypingStatisticsTarget;
use crate::typing_engine::processed_chunk_info::ConfirmedChunk;
use crate::typing_engine::processed_chunk_info::KeyStrokeDisplayInfo;
use crate::typing_engine::processed_chunk_info::SpellDisplayInfo;
use crate::typing_engine::processed_chunk_info::TypedChunk;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::{gen_candidate, gen_chunk};

#[test]
fn stroke_key_1() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "う",
            vec![
                gen_candidate!(["u"], true, None),
                gen_candidate!(["wu"], true, None),
                gen_candidate!(["whu"], true, None)
            ],
            gen_candidate!(["u"], true, None)
        ),
        gen_chunk!(
            "っ",
            vec![
                gen_candidate!(["w"], true, None, 'w'),
                gen_candidate!(["ltu"], true, None),
                gen_candidate!(["xtu"], true, None),
                gen_candidate!(["ltsu"], true, None)
            ],
            gen_candidate!(["w"], true, None, 'w')
        ),
        gen_chunk!(
            "う",
            vec![
                gen_candidate!(["u"], true, None),
                gen_candidate!(["wu"], true, None),
                gen_candidate!(["whu"], true, None)
            ],
            gen_candidate!(["wu"], true, None)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, None),
                        gen_candidate!(["wu"], true, None),
                        gen_candidate!(["whu"], true, None)
                    ],
                    gen_candidate!(["u"], true, None)
                ),
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["w"], true, None, 'w'),
                        gen_candidate!(["ltu"], true, None),
                        gen_candidate!(["xtu"], true, None),
                        gen_candidate!(["ltsu"], true, None)
                    ],
                    gen_candidate!(["w"], true, None, 'w')
                ),
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, None),
                        gen_candidate!(["wu"], true, None),
                        gen_candidate!(["whu"], true, None)
                    ],
                    gen_candidate!(["wu"], true, None)
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
                        gen_candidate!(["w"], true, None, 'w'),
                        gen_candidate!(["ltu"], true, None),
                        gen_candidate!(["xtu"], true, None),
                        gen_candidate!(["ltsu"], true, None)
                    ],
                    gen_candidate!(["w"], true, None, 'w')
                ),
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, None),
                        gen_candidate!(["wu"], true, None),
                        gen_candidate!(["whu"], true, None)
                    ],
                    gen_candidate!(["wu"], true, None)
                ),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, Some(0)),
                        gen_candidate!(["wu"], true, Some(0)),
                        gen_candidate!(["whu"], true, Some(0))
                    ],
                    gen_candidate!(["u"], true, None)
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
                    gen_candidate!(["u"], true, None),
                    gen_candidate!(["wu"], true, None),
                    gen_candidate!(["whu"], true, None)
                ],
                gen_candidate!(["wu"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["w"], true, Some(0), 'w'),
                        gen_candidate!(["ltu"], true, Some(0)),
                        gen_candidate!(["xtu"], true, Some(0)),
                        gen_candidate!(["ltsu"], true, Some(0))
                    ],
                    gen_candidate!(["w"], true, None, 'w')
                )
                .into()
            ),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, Some(1)),
                        gen_candidate!(["wu"], false, Some(0)),
                        gen_candidate!(["whu"], false, Some(0))
                    ],
                    gen_candidate!(["u"], true, None)
                ),
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
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], false, Some(0)),
                        gen_candidate!(["wu"], true, Some(0)),
                        gen_candidate!(["whu"], true, Some(0))
                    ],
                    gen_candidate!(["wu"], true, None)
                )
                .into()
            ),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"], true, Some(1)),
                            gen_candidate!(["wu"], false, Some(0)),
                            gen_candidate!(["whu"], false, Some(0))
                        ],
                        gen_candidate!(["u"], true, None)
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["w"], true, Some(1), 'w'),
                            gen_candidate!(["ltu"], false, Some(0)),
                            gen_candidate!(["xtu"], false, Some(0)),
                            gen_candidate!(["ltsu"], false, Some(0))
                        ],
                        gen_candidate!(["w"], true, None, 'w')
                    ),
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
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], false, Some(0)),
                        gen_candidate!(["wu"], true, Some(1)),
                        gen_candidate!(["whu"], true, Some(1))
                    ],
                    gen_candidate!(["wu"], true, None)
                )
                .into(),
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'w'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"], true, Some(1)),
                            gen_candidate!(["wu"], false, Some(0)),
                            gen_candidate!(["whu"], false, Some(0))
                        ],
                        gen_candidate!(["u"], true, None)
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["w"], true, Some(1), 'w'),
                            gen_candidate!(["ltu"], false, Some(0)),
                            gen_candidate!(["xtu"], false, Some(0)),
                            gen_candidate!(["ltsu"], false, Some(0))
                        ],
                        gen_candidate!(["w"], true, None, 'w')
                    ),
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
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"], true, Some(1)),
                            gen_candidate!(["wu"], false, Some(0)),
                            gen_candidate!(["whu"], false, Some(0))
                        ],
                        gen_candidate!(["u"], true, None)
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["w"], true, Some(1), 'w'),
                            gen_candidate!(["ltu"], false, Some(0)),
                            gen_candidate!(["xtu"], false, Some(0)),
                            gen_candidate!(["ltsu"], false, Some(0))
                        ],
                        gen_candidate!(["w"], true, None, 'w')
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"], false, Some(0)),
                            gen_candidate!(["wu"], true, Some(2)),
                            gen_candidate!(["whu"], false, Some(1))
                        ],
                        gen_candidate!(["wu"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(3, 0), 'w'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'u'.try_into().unwrap(), true),
                    ]
                )
            ],
        }
    );

    assert!(pci.is_finished());

    pci.append_chunks(vec![gen_chunk!(
        "う",
        vec![
            gen_candidate!(["u"], true, None),
            gen_candidate!(["wu"], true, None),
            gen_candidate!(["whu"], true, None)
        ],
        gen_candidate!(["u"], true, None)
    )]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, Some(0)),
                        gen_candidate!(["wu"], true, Some(0)),
                        gen_candidate!(["whu"], true, Some(0))
                    ],
                    gen_candidate!(["u"], true, None)
                )
                .into()
            ),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"], true, Some(1)),
                            gen_candidate!(["wu"], false, Some(0)),
                            gen_candidate!(["whu"], false, Some(0))
                        ],
                        gen_candidate!(["u"], true, None)
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["w"], true, Some(1), 'w'),
                            gen_candidate!(["ltu"], false, Some(0)),
                            gen_candidate!(["xtu"], false, Some(0)),
                            gen_candidate!(["ltsu"], false, Some(0))
                        ],
                        gen_candidate!(["w"], true, None, 'w')
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "う",
                        vec![
                            gen_candidate!(["u"], false, Some(0)),
                            gen_candidate!(["wu"], true, Some(2)),
                            gen_candidate!(["whu"], false, Some(1))
                        ],
                        gen_candidate!(["wu"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(3, 0), 'w'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'u'.try_into().unwrap(), true),
                    ]
                )
            ],
        }
    );
}

#[test]
fn stroke_key_2() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "か",
            vec![
                gen_candidate!(["ka"], true, None),
                gen_candidate!(["ca"], true, None),
            ],
            gen_candidate!(["ka"], true, None)
        ),
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None, ['k']),
                gen_candidate!(["nn"], true, None),
                gen_candidate!(["xn"], true, None)
            ],
            gen_candidate!(["n"], true, None, ['k'])
        ),
        gen_chunk!(
            "き",
            vec![gen_candidate!(["ki"], true, None),],
            gen_candidate!(["ki"], true, None)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, None),
                        gen_candidate!(["ca"], true, None),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['k']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                ),
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, None),],
                    gen_candidate!(["ki"], true, None)
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
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['k']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                ),
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, None),],
                    gen_candidate!(["ki"], true, None)
                ),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(0)),
                        gen_candidate!(["ca"], true, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                )
                .into()
            ),
            confirmed_chunks: vec![],
        }
    );

    // 3. 「k」と入力
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['k']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                ),
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, None),],
                    gen_candidate!(["ki"], true, None)
                ),
            ]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(1)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'k'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![],
        }
    );

    // 3. 「a」と入力
    pci.stroke_key('a'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "き",
                vec![gen_candidate!(["ki"], true, None),],
                gen_candidate!(["ki"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(0), ['k']),
                        gen_candidate!(["nn"], true, Some(0)),
                        gen_candidate!(["xn"], true, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                )
                .into()
            ),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(2)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ],
            )],
        }
    );

    // 4. 「n」と入力
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "き",
                vec![gen_candidate!(["ki"], true, None),],
                gen_candidate!(["ki"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['k']),
                        gen_candidate!(["nn"], true, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                )
                .into(),
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(2)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ],
            ),],
        }
    );

    // 5. 「j」と入力（ミスタイプ）
    // 遅延確定候補が確定していないのでミスタイプはどのチャンクにも属さない
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(4, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "き",
                vec![gen_candidate!(["ki"], true, None),],
                gen_candidate!(["ki"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['k']),
                        gen_candidate!(["nn"], true, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                )
                .into(),
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                vec![ActualKeyStroke::new(
                    Duration::new(4, 0),
                    'j'.try_into().unwrap(),
                    false
                )]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(2)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ],
            ),],
        }
    );

    // 6. 「k」と入力
    // 遅延確定候補が確定したのでミスタイプは次のチャンクに属する
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(5, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, Some(1)),],
                    gen_candidate!(["ki"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(5, 0), 'k'.try_into().unwrap(), true)
                ],
                vec![]
            )),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "か",
                        vec![
                            gen_candidate!(["ka"], true, Some(2)),
                            gen_candidate!(["ca"], false, Some(0)),
                        ],
                        gen_candidate!(["ka"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![
                            gen_candidate!(["n"], true, Some(1), ['k']),
                            gen_candidate!(["nn"], false, Some(1)),
                            gen_candidate!(["xn"], false, Some(0))
                        ],
                        gen_candidate!(["n"], true, None, ['k'])
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    ),],
                )
            ],
        }
    );

    // 7. 「i」と入力
    pci.stroke_key('i'.try_into().unwrap(), Duration::new(6, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: None,
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "か",
                        vec![
                            gen_candidate!(["ka"], true, Some(2)),
                            gen_candidate!(["ca"], false, Some(0)),
                        ],
                        gen_candidate!(["ka"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![
                            gen_candidate!(["n"], true, Some(1), ['k']),
                            gen_candidate!(["nn"], false, Some(1)),
                            gen_candidate!(["xn"], false, Some(0))
                        ],
                        gen_candidate!(["n"], true, None, ['k'])
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    ),],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "き",
                        vec![gen_candidate!(["ki"], true, Some(2)),],
                        gen_candidate!(["ki"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(6, 0), 'i'.try_into().unwrap(), true)
                    ],
                ),
            ],
        }
    );

    assert!(pci.is_finished());
}

#[test]
fn stroke_key_3() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "か",
            vec![
                gen_candidate!(["ka"], true, None),
                gen_candidate!(["ca"], true, None),
            ],
            gen_candidate!(["ka"], true, None)
        ),
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None, ['k']),
                gen_candidate!(["nn"], true, None),
                gen_candidate!(["xn"], true, None)
            ],
            gen_candidate!(["n"], true, None, ['k'])
        ),
        gen_chunk!(
            "き",
            vec![gen_candidate!(["ki"], true, None),],
            gen_candidate!(["ki"], true, None)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, None),
                        gen_candidate!(["ca"], true, None),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['k']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                ),
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, None),],
                    gen_candidate!(["ki"], true, None)
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
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['k']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                ),
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, None),],
                    gen_candidate!(["ki"], true, None)
                ),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(0)),
                        gen_candidate!(["ca"], true, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                )
                .into()
            ),
            confirmed_chunks: vec![],
        }
    );

    // 3. 「k」と入力
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['k']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                ),
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, None),],
                    gen_candidate!(["ki"], true, None)
                ),
            ]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(1)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'k'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![],
        }
    );

    // 3. 「a」と入力
    pci.stroke_key('a'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "き",
                vec![gen_candidate!(["ki"], true, None),],
                gen_candidate!(["ki"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(0), ['k']),
                        gen_candidate!(["nn"], true, Some(0)),
                        gen_candidate!(["xn"], true, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                )
                .into()
            ),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(2)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ],
            )],
        }
    );

    // 4. 「n」と入力
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "き",
                vec![gen_candidate!(["ki"], true, None),],
                gen_candidate!(["ki"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['k']),
                        gen_candidate!(["nn"], true, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                )
                .into(),
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(2)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ],
            ),],
        }
    );

    // 5. 「j」と入力（ミスタイプ）
    // 遅延確定候補が確定していないのでミスタイプはどのチャンクにも属さない
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(4, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "き",
                vec![gen_candidate!(["ki"], true, None),],
                gen_candidate!(["ki"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['k']),
                        gen_candidate!(["nn"], true, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['k'])
                )
                .into(),
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                vec![ActualKeyStroke::new(
                    Duration::new(4, 0),
                    'j'.try_into().unwrap(),
                    false
                )]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, Some(2)),
                        gen_candidate!(["ca"], false, Some(0)),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ],
            ),],
        }
    );

    // 6. 「n」と入力
    // 遅延確定候補でない候補で確定したのでミスタイプはそのチャンクに属する
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(5, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, Some(0)),],
                    gen_candidate!(["ki"], true, None)
                )
                .into()
            ),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "か",
                        vec![
                            gen_candidate!(["ka"], true, Some(2)),
                            gen_candidate!(["ca"], false, Some(0)),
                        ],
                        gen_candidate!(["ka"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![
                            gen_candidate!(["n"], false, Some(1), ['k']),
                            gen_candidate!(["nn"], true, Some(2)),
                            gen_candidate!(["xn"], false, Some(0))
                        ],
                        gen_candidate!(["n"], true, None, ['k'])
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'n'.try_into().unwrap(), true),
                    ],
                )
            ],
        }
    );

    // 7. 「k」と入力
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(6, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "き",
                    vec![gen_candidate!(["ki"], true, Some(1)),],
                    gen_candidate!(["ki"], true, None)
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(6, 0),
                    'k'.try_into().unwrap(),
                    true
                ),],
                vec![]
            )),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "か",
                        vec![
                            gen_candidate!(["ka"], true, Some(2)),
                            gen_candidate!(["ca"], false, Some(0)),
                        ],
                        gen_candidate!(["ka"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![
                            gen_candidate!(["n"], false, Some(1), ['k']),
                            gen_candidate!(["nn"], true, Some(2)),
                            gen_candidate!(["xn"], false, Some(0))
                        ],
                        gen_candidate!(["n"], true, None, ['k'])
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'n'.try_into().unwrap(), true),
                    ],
                )
            ],
        }
    );

    // 8. 「i」と入力
    pci.stroke_key('i'.try_into().unwrap(), Duration::new(7, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: None,
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "か",
                        vec![
                            gen_candidate!(["ka"], true, Some(2)),
                            gen_candidate!(["ca"], false, Some(0)),
                        ],
                        gen_candidate!(["ka"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![
                            gen_candidate!(["n"], false, Some(1), ['k']),
                            gen_candidate!(["nn"], true, Some(2)),
                            gen_candidate!(["xn"], false, Some(0))
                        ],
                        gen_candidate!(["n"], true, None, ['k'])
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'n'.try_into().unwrap(), true),
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "き",
                        vec![gen_candidate!(["ki"], true, Some(2)),],
                        gen_candidate!(["ki"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(6, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(7, 0), 'i'.try_into().unwrap(), true)
                    ],
                ),
            ],
        }
    );

    assert!(pci.is_finished());
}

#[test]
fn stroke_key_4() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None, ['p']),
                gen_candidate!(["nn"], true, None),
                gen_candidate!(["xn"], true, None)
            ],
            gen_candidate!(["n"], true, None, ['p'])
        ),
        gen_chunk!(
            "ぴ",
            vec![gen_candidate!(["p"], true, None),],
            gen_candidate!(["p"], true, None)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['p']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['p'])
                ),
                gen_chunk!(
                    "ぴ",
                    vec![gen_candidate!(["p"], true, None),],
                    gen_candidate!(["p"], true, None)
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
            unprocessed_chunks: vec![gen_chunk!(
                "ぴ",
                vec![gen_candidate!(["p"], true, None),],
                gen_candidate!(["p"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(0), ['p']),
                        gen_candidate!(["nn"], true, Some(0)),
                        gen_candidate!(["xn"], true, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['p'])
                )
                .into()
            ),
            confirmed_chunks: vec![],
        }
    );

    // 3. 「n」と入力
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "ぴ",
                vec![gen_candidate!(["p"], true, None),],
                gen_candidate!(["p"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['p']),
                        gen_candidate!(["nn"], true, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['p'])
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![],
        }
    );

    // 3. 「p」と入力
    pci.stroke_key('p'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: None,
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![
                            gen_candidate!(["n"], true, Some(1), ['p']),
                            gen_candidate!(["nn"], false, Some(1)),
                            gen_candidate!(["xn"], false, Some(0))
                        ],
                        gen_candidate!(["n"], true, None, ['p'])
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'n'.try_into().unwrap(),
                        true
                    ),],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ぴ",
                        vec![gen_candidate!(["p"], true, Some(1)),],
                        gen_candidate!(["p"], true, None)
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'p'.try_into().unwrap(),
                        true
                    ),],
                )
            ],
        }
    );

    assert!(pci.is_finished());
}

#[test]
fn construct_display_info_1() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"], true, None),
                gen_candidate!(["ki", "lyo"], true, None),
                gen_candidate!(["ki", "xyo"], true, None)
            ],
            gen_candidate!(["kyo"], true, None)
        ),
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"], true, None),
                gen_candidate!(["ki", "lyo"], true, None),
                gen_candidate!(["ki", "xyo"], true, None)
            ],
            gen_candidate!(["kyo"], true, None)
        ),
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"], true, None),
                gen_candidate!(["ki", "lyo"], true, None),
                gen_candidate!(["ki", "xyo"], true, None)
            ],
            gen_candidate!(["kyo"], true, None)
        ),
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["ky"], true, None),
                gen_candidate!(["ki"], true, None),
            ],
            gen_candidate!(["ky"], true, None)
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
            unprocessed_chunks: vec![gen_chunk!(
                "きょ",
                vec![
                    gen_candidate!(["ky"], true, None),
                    gen_candidate!(["ki"], true, None),
                ],
                gen_candidate!(["ky"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "きょ",
                    vec![
                        gen_candidate!(["kyo"], true, Some(1)),
                        gen_candidate!(["ki", "lyo"], true, Some(1)),
                        gen_candidate!(["ki", "xyo"], true, Some(1)),
                    ],
                    gen_candidate!(["kyo"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(11, 0), 'c'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(12, 0), 'k'.try_into().unwrap(), true),
                ],
                vec![]
            )),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!(
                        "きょ",
                        vec![
                            gen_candidate!(["kyo"], true, Some(3)),
                            gen_candidate!(["ki", "lyo"], false, Some(1)),
                            gen_candidate!(["ki", "xyo"], false, Some(1)),
                        ],
                        gen_candidate!(["kyo"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'u'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(3, 0), 'y'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'o'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "きょ",
                        vec![
                            gen_candidate!(["kyo"], false, Some(1)),
                            gen_candidate!(["ki", "lyo"], false, Some(2)),
                            gen_candidate!(["ki", "xyo"], true, Some(5)),
                        ],
                        gen_candidate!(["kyo"], true, None)
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(5, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(6, 0), 'i'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(7, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(8, 0), 'x'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(9, 0), 'y'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(10, 0), 'o'.try_into().unwrap(), true)
                    ],
                ),
            ],
        }
    );

    let (sdi, ksdi) =
        pci.construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()));

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "きょきょきょきょ".to_string(),
            vec![4, 5],
            vec![0, 1, 3, 4, 5],
            7,
            OnTypingStatisticsTarget::new(4, 8, 1, 5, None, None, vec![1, 2, 3, 3, 5, 6])
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "kyokixyokyoky".to_string(),
            9,
            vec![1, 5, 8],
            OnTypingStatisticsTarget::new(
                9,
                13,
                6,
                3,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![
                    Duration::new(3, 0),
                    Duration::new(5, 0),
                    Duration::new(8, 0),
                    Duration::new(10, 0)
                ]),
                vec![1, 3, 5, 7, 9, 11],
            ),
            OnTypingStatisticsTarget::new(7, 11, 4, 3, None, None, vec![1, 3, 4, 5, 7, 9])
        )
    );

    let (_, ksdi) =
        pci.construct_display_info(LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()));

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "kyokixyokyoky".to_string(),
            9,
            vec![1, 5, 8],
            OnTypingStatisticsTarget::new(9, 13, 6, 3, None, None, vec![1, 4, 7, 9, 11]),
            OnTypingStatisticsTarget::new(
                7,
                11,
                4,
                3,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![
                    Duration::new(3, 0),
                    Duration::new(5, 0),
                    Duration::new(10, 0)
                ]),
                vec![1, 3, 5, 7, 9]
            )
        )
    );

    let (sdi, ksdi) = pci.construct_display_info(LapRequest::Spell(NonZeroUsize::new(1).unwrap()));

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "きょきょきょきょ".to_string(),
            vec![4, 5],
            vec![0, 1, 3, 4, 5],
            7,
            OnTypingStatisticsTarget::new(
                4,
                8,
                1,
                5,
                Some(NonZeroUsize::new(1).unwrap()),
                Some(vec![
                    Duration::new(4, 0),
                    Duration::new(4, 0),
                    Duration::new(6, 0),
                    Duration::new(10, 0)
                ]),
                vec![0, 1, 2, 3, 4, 5, 6, 7]
            )
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "kyokixyokyoky".to_string(),
            9,
            vec![1, 5, 8],
            OnTypingStatisticsTarget::new(9, 13, 6, 3, None, None, vec![0, 2, 4, 7, 8, 10, 11, 12]),
            OnTypingStatisticsTarget::new(7, 11, 4, 3, None, None, vec![0, 2, 3, 5, 6, 8, 9, 10])
        )
    );
}

#[test]
fn construct_display_info_2() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None, ['z', 'j']),
                gen_candidate!(["nn"], true, None),
                gen_candidate!(["xn"], true, None)
            ],
            gen_candidate!(["n"], true, None, ['z', 'j'])
        ),
        gen_chunk!(
            "じ",
            vec![
                gen_candidate!(["zi"], true, None),
                gen_candidate!(["ji"], true, None),
            ],
            gen_candidate!(["zi"], true, None)
        ),
    ]);

    // 2. タイピング開始
    pci.move_next_chunk();

    // 3. n -> m(ミスタイプ) と入力
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "じ",
                vec![
                    gen_candidate!(["zi"], true, None),
                    gen_candidate!(["ji"], true, None),
                ],
                gen_candidate!(["zi"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['z', 'j']),
                        gen_candidate!(["nn"], true, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['z', 'j'])
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                vec![ActualKeyStroke::new(
                    Duration::new(2, 0),
                    'm'.try_into().unwrap(),
                    false
                ),]
            )),
            confirmed_chunks: vec![],
        }
    );

    let (sdi, ksdi) =
        pci.construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()));

    // 入力を終えた遅延確定候補は表示の上では確定したとみなす
    // pendingにあるミスタイプは表示状は次のチャンクに帰属させる
    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![1],
            1,
            OnTypingStatisticsTarget::new(1, 2, 1, 1, None, None, vec![1])
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nzi".to_string(),
            1,
            vec![1],
            OnTypingStatisticsTarget::new(
                1,
                3,
                1,
                1,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![]),
                vec![1],
            ),
            OnTypingStatisticsTarget::new(1, 3, 1, 1, None, None, vec![1])
        )
    );

    let (_, ksdi) =
        pci.construct_display_info(LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()));

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nzi".to_string(),
            1,
            vec![1],
            OnTypingStatisticsTarget::new(1, 3, 1, 1, None, None, vec![1]),
            OnTypingStatisticsTarget::new(
                1,
                3,
                1,
                1,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![]),
                vec![1]
            )
        )
    );

    let (sdi, ksdi) = pci.construct_display_info(LapRequest::Spell(NonZeroUsize::new(1).unwrap()));

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![1],
            1,
            OnTypingStatisticsTarget::new(
                1,
                2,
                1,
                1,
                Some(NonZeroUsize::new(1).unwrap()),
                Some(vec![Duration::new(1, 0)]),
                vec![0, 1]
            )
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nzi".to_string(),
            1,
            vec![1],
            OnTypingStatisticsTarget::new(1, 3, 1, 1, None, None, vec![0, 2]),
            OnTypingStatisticsTarget::new(1, 3, 1, 1, None, None, vec![0, 2])
        )
    );

    // 4. jと入力
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(3, 0));

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "じ",
                    vec![
                        gen_candidate!(["zi"], false, Some(0)),
                        gen_candidate!(["ji"], true, Some(1)),
                    ],
                    gen_candidate!(["zi"], true, None)
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), true),
                ],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['z', 'j']),
                        gen_candidate!(["nn"], false, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['z', 'j'])
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
            )],
        }
    );

    let (sdi, ksdi) =
        pci.construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()));

    // 遅延確定候補で確定したのでミスタイプは引き続き次のチャンクに属する
    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![1],
            1,
            OnTypingStatisticsTarget::new(1, 2, 1, 1, None, None, vec![1])
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nji".to_string(),
            2,
            vec![1],
            OnTypingStatisticsTarget::new(
                2,
                3,
                1,
                1,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![Duration::new(3, 0)]),
                vec![1],
            ),
            OnTypingStatisticsTarget::new(2, 3, 1, 1, None, None, vec![1])
        )
    );

    let (_, ksdi) =
        pci.construct_display_info(LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()));

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nji".to_string(),
            2,
            vec![1],
            OnTypingStatisticsTarget::new(2, 3, 1, 1, None, None, vec![1]),
            OnTypingStatisticsTarget::new(
                2,
                3,
                1,
                1,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![Duration::new(3, 0)]),
                vec![1]
            )
        )
    );

    let (sdi, ksdi) = pci.construct_display_info(LapRequest::Spell(NonZeroUsize::new(1).unwrap()));

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![1],
            1,
            OnTypingStatisticsTarget::new(
                1,
                2,
                1,
                1,
                Some(NonZeroUsize::new(1).unwrap()),
                Some(vec![Duration::new(1, 0)]),
                vec![0, 1]
            )
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nji".to_string(),
            2,
            vec![1],
            OnTypingStatisticsTarget::new(2, 3, 1, 1, None, None, vec![0, 2]),
            OnTypingStatisticsTarget::new(2, 3, 1, 1, None, None, vec![0, 2])
        )
    );
}

#[test]
fn construct_display_info_3() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None, ['z', 'j']),
                gen_candidate!(["nn"], true, None),
                gen_candidate!(["xn"], true, None)
            ],
            gen_candidate!(["n"], true, None, ['z', 'j'])
        ),
        gen_chunk!(
            "じ",
            vec![
                gen_candidate!(["zi"], true, None),
                gen_candidate!(["ji"], true, None),
            ],
            gen_candidate!(["zi"], true, None)
        ),
    ]);

    // 2. タイピング開始
    pci.move_next_chunk();

    // 3. n -> m(ミスタイプ) と入力
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk!(
                "じ",
                vec![
                    gen_candidate!(["zi"], true, None),
                    gen_candidate!(["ji"], true, None),
                ],
                gen_candidate!(["zi"], true, None)
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, Some(1), ['z', 'j']),
                        gen_candidate!(["nn"], true, Some(1)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['z', 'j'])
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
                vec![ActualKeyStroke::new(
                    Duration::new(2, 0),
                    'm'.try_into().unwrap(),
                    false
                ),]
            )),
            confirmed_chunks: vec![],
        }
    );

    let (sdi, ksdi) =
        pci.construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()));

    // 入力を終えた遅延確定候補は表示の上では確定したとみなす
    // pendingにあるミスタイプは表示状は次のチャンクに帰属させる
    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![1],
            1,
            OnTypingStatisticsTarget::new(1, 2, 1, 1, None, None, vec![1])
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nzi".to_string(),
            1,
            vec![1],
            OnTypingStatisticsTarget::new(
                1,
                3,
                1,
                1,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![]),
                vec![1],
            ),
            OnTypingStatisticsTarget::new(1, 3, 1, 1, None, None, vec![1])
        )
    );

    let (sdi, ksdi) = pci.construct_display_info(LapRequest::Spell(NonZeroUsize::new(1).unwrap()));

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![1],
            1,
            OnTypingStatisticsTarget::new(
                1,
                2,
                1,
                1,
                Some(NonZeroUsize::new(1).unwrap()),
                Some(vec![Duration::new(1, 0)]),
                vec![0, 1]
            )
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nzi".to_string(),
            1,
            vec![1],
            OnTypingStatisticsTarget::new(1, 3, 1, 1, None, None, vec![0, 2]),
            OnTypingStatisticsTarget::new(1, 3, 1, 1, None, None, vec![0, 2])
        )
    );

    // 4. nと入力
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "じ",
                    vec![
                        gen_candidate!(["zi"], true, Some(0)),
                        gen_candidate!(["ji"], true, Some(0)),
                    ],
                    gen_candidate!(["zi"], true, None)
                ),
                vec![],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], false, Some(1), ['z', 'j']),
                        gen_candidate!(["nn"], true, Some(2)),
                        gen_candidate!(["xn"], false, Some(0))
                    ],
                    gen_candidate!(["n"], true, None, ['z', 'j'])
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'n'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                ],
            )],
        }
    );

    let (sdi, ksdi) =
        pci.construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()));

    // 遅延確定候補ではない候補で確定したのでミスタイプはその候補に属する
    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![0],
            1,
            OnTypingStatisticsTarget::new(1, 2, 0, 1, None, None, vec![0, 1])
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nnzi".to_string(),
            2,
            vec![1],
            OnTypingStatisticsTarget::new(
                2,
                4,
                1,
                1,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![Duration::new(3, 0)]),
                vec![1, 3],
            ),
            OnTypingStatisticsTarget::new(1, 3, 0, 1, None, None, vec![0, 2])
        )
    );

    let (_, ksdi) =
        pci.construct_display_info(LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()));

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nnzi".to_string(),
            2,
            vec![1],
            OnTypingStatisticsTarget::new(2, 4, 1, 1, None, None, vec![2]),
            OnTypingStatisticsTarget::new(
                1,
                3,
                0,
                1,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![]),
                vec![1]
            )
        )
    );

    let (sdi, ksdi) = pci.construct_display_info(LapRequest::Spell(NonZeroUsize::new(1).unwrap()));

    // 遅延確定候補ではない候補で確定したのでミスタイプはその候補に属する
    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "んじ".to_string(),
            vec![1],
            vec![0],
            1,
            OnTypingStatisticsTarget::new(
                1,
                2,
                0,
                1,
                Some(NonZeroUsize::new(1).unwrap()),
                Some(vec![Duration::new(3, 0)]),
                vec![0, 1]
            )
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nnzi".to_string(),
            2,
            vec![1],
            OnTypingStatisticsTarget::new(2, 4, 1, 1, None, None, vec![1, 3]),
            OnTypingStatisticsTarget::new(1, 3, 0, 1, None, None, vec![0, 2])
        )
    );
}

#[test]
fn construct_display_info_4() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "あ",
            vec![gen_candidate!(["a"], true, None)],
            gen_candidate!(["a"], true, None)
        ),
        gen_chunk!(
            "っ",
            vec![
                gen_candidate!(["k"], true, None, 'k', ['k']),
                gen_candidate!(["c"], true, None, 'c', ['c']),
                gen_candidate!(["ltu"], true, None),
                gen_candidate!(["xtu"], true, None),
                gen_candidate!(["ltsu"], true, None)
            ],
            gen_candidate!(["k"], true, None, 'k', ['k'])
        ),
        gen_chunk!(
            "か",
            vec![
                gen_candidate!(["ka"], true, None),
                gen_candidate!(["ca"], true, None)
            ],
            gen_candidate!(["ka"], true, None)
        ),
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["nn"], true, None),
                gen_candidate!(["xn"], true, None)
            ],
            gen_candidate!(["nn"], true, None)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "あ",
                    vec![gen_candidate!(["a"], true, None)],
                    gen_candidate!(["a"], true, None)
                ),
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["k"], true, None, 'k', ['k']),
                        gen_candidate!(["c"], true, None, 'c', ['c']),
                        gen_candidate!(["ltu"], true, None),
                        gen_candidate!(["xtu"], true, None),
                        gen_candidate!(["ltsu"], true, None)
                    ],
                    gen_candidate!(["k"], true, None, 'k', ['k'])
                ),
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, None),
                        gen_candidate!(["ca"], true, None)
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["nn"], true, None)
                ),
            ]
            .into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
        }
    );

    // 2. タイピング開始
    pci.move_next_chunk();

    // 3. a と入力
    pci.stroke_key('a'.try_into().unwrap(), Duration::new(1, 0));

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, None),
                        gen_candidate!(["ca"], true, None)
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["nn"], true, None)
                ),
            ]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["k"], true, Some(0), 'k', ['k']),
                        gen_candidate!(["c"], true, Some(0), 'c', ['c']),
                        gen_candidate!(["ltu"], true, Some(0)),
                        gen_candidate!(["xtu"], true, Some(0)),
                        gen_candidate!(["ltsu"], true, Some(0))
                    ],
                    gen_candidate!(["k"], true, None, 'k', ['k'])
                ),
                vec![],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "あ",
                    vec![gen_candidate!(["a"], true, Some(1))],
                    gen_candidate!(["a"], true, None)
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'a'.try_into().unwrap(),
                    true
                ),],
            )],
        }
    );

    let (sdi, ksdi) =
        pci.construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()));

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "あっかん".to_string(),
            vec![1],
            vec![],
            3,
            OnTypingStatisticsTarget::new(1, 4, 1, 0, None, None, vec![1, 2, 3])
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "akkann".to_string(),
            1,
            vec![],
            OnTypingStatisticsTarget::new(
                1,
                6,
                1,
                0,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![]),
                vec![1, 3, 5],
            ),
            OnTypingStatisticsTarget::new(1, 6, 1, 0, None, None, vec![1, 3, 5])
        )
    );

    let (_, ksdi) =
        pci.construct_display_info(LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()));

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "akkann".to_string(),
            1,
            vec![],
            OnTypingStatisticsTarget::new(1, 6, 1, 0, None, None, vec![1, 3, 5]),
            OnTypingStatisticsTarget::new(
                1,
                6,
                1,
                0,
                Some(NonZeroUsize::new(2).unwrap()),
                Some(vec![]),
                vec![1, 3, 5]
            )
        )
    );

    let (sdi, ksdi) = pci.construct_display_info(LapRequest::Spell(NonZeroUsize::new(1).unwrap()));

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "あっかん".to_string(),
            vec![1],
            vec![],
            3,
            OnTypingStatisticsTarget::new(
                1,
                4,
                1,
                0,
                Some(NonZeroUsize::new(1).unwrap()),
                Some(vec![Duration::new(1, 0)]),
                vec![0, 1, 2, 3]
            )
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "akkann".to_string(),
            1,
            vec![],
            OnTypingStatisticsTarget::new(1, 6, 1, 0, None, None, vec![0, 1, 3, 5]),
            OnTypingStatisticsTarget::new(1, 6, 1, 0, None, None, vec![0, 1, 3, 5])
        )
    );
}
