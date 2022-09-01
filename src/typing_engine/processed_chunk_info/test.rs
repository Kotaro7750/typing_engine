use super::*;

use std::time::Duration;

use crate::key_stroke::ActualKeyStroke;
use crate::statistics::OnTypingStatisticsDynamicTarget;
use crate::typing_engine::processed_chunk_info::ConfirmedChunk;
use crate::typing_engine::processed_chunk_info::KeyStrokeDisplayInfo;
use crate::typing_engine::processed_chunk_info::SpellDisplayInfo;
use crate::typing_engine::processed_chunk_info::TypedChunk;
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
            ],
            gen_candidate!(["u"])
        ),
        gen_chunk!(
            "っ",
            vec![
                gen_candidate!(["w"], 'w'),
                gen_candidate!(["ltu"]),
                gen_candidate!(["xtu"]),
                gen_candidate!(["ltsu"])
            ],
            gen_candidate!(["w"], 'w')
        ),
        gen_chunk!(
            "う",
            vec![
                gen_candidate!(["u"]),
                gen_candidate!(["wu"]),
                gen_candidate!(["whu"])
            ],
            gen_candidate!(["wu"])
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
                    ],
                    gen_candidate!(["u"])
                ),
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["w"], 'w'),
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"])
                    ],
                    gen_candidate!(["w"], 'w')
                ),
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"]),
                        gen_candidate!(["wu"]),
                        gen_candidate!(["whu"])
                    ],
                    gen_candidate!(["wu"])
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
                    ],
                    gen_candidate!(["w"], 'w')
                ),
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"]),
                        gen_candidate!(["wu"]),
                        gen_candidate!(["whu"])
                    ],
                    gen_candidate!(["wu"])
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
                    ],
                    gen_candidate!(["u"])
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
                ],
                gen_candidate!(["wu"])
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
                    ],
                    gen_candidate!(["w"], 'w')
                )
                .into()
            ),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!("う", vec![gen_candidate!(["u"]),], gen_candidate!(["u"])),
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
                    vec![gen_candidate!(["wu"]), gen_candidate!(["whu"])],
                    gen_candidate!(["wu"])
                )
                .into()
            ),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!("う", vec![gen_candidate!(["u"]),], gen_candidate!(["u"])),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![gen_candidate!(["w"], 'w'),],
                        gen_candidate!(["w"], 'w')
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
                    vec![gen_candidate!(["wu"]), gen_candidate!(["whu"])],
                    gen_candidate!(["wu"])
                )
                .into(),
                vec![1, 1],
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'w'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!("う", vec![gen_candidate!(["u"]),], gen_candidate!(["u"])),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![gen_candidate!(["w"], 'w'),],
                        gen_candidate!(["w"], 'w')
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
                    gen_chunk!("う", vec![gen_candidate!(["u"]),], gen_candidate!(["u"])),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![gen_candidate!(["w"], 'w'),],
                        gen_candidate!(["w"], 'w')
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!("う", vec![gen_candidate!(["wu"])], gen_candidate!(["wu"])),
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
            gen_candidate!(["u"]),
            gen_candidate!(["wu"]),
            gen_candidate!(["whu"])
        ],
        gen_candidate!(["u"])
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
                    ],
                    gen_candidate!(["u"])
                )
                .into()
            ),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!("う", vec![gen_candidate!(["u"]),], gen_candidate!(["u"])),
                    vec![ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "っ",
                        vec![gen_candidate!(["w"], 'w'),],
                        gen_candidate!(["w"], 'w')
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )],
                ),
                ConfirmedChunk::new(
                    gen_chunk!("う", vec![gen_candidate!(["wu"])], gen_candidate!(["wu"])),
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
            vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
            gen_candidate!(["ka"])
        ),
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], ['k']),
                gen_candidate!(["nn"]),
                gen_candidate!(["xn"])
            ],
            gen_candidate!(["n"], ['k'])
        ),
        gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "か",
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
                    gen_candidate!(["ka"])
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                ),
                gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
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
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                ),
                gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "か",
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
                    gen_candidate!(["ka"])
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
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                ),
                gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
            ]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!("か", vec![gen_candidate!(["ka"])], gen_candidate!(["ka"])),
                vec![1],
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
                vec![gen_candidate!(["ki"])],
                gen_candidate!(["ki"])
            )]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                )
                .into()
            ),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
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
                vec![gen_candidate!(["ki"])],
                gen_candidate!(["ki"])
            )]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['k']), gen_candidate!(["nn"]),],
                    gen_candidate!(["n"], ['k'])
                )
                .into(),
                vec![1, 1],
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
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
                vec![gen_candidate!(["ki"])],
                gen_candidate!(["ki"])
            )]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['k']), gen_candidate!(["nn"]),],
                    gen_candidate!(["n"], ['k'])
                )
                .into(),
                vec![1, 1],
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
                gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
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
                gen_chunk!("き", vec![gen_candidate!(["ki"])], gen_candidate!(["ki"])),
                vec![1],
                vec![
                    ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(5, 0), 'k'.try_into().unwrap(), true)
                ],
                vec![]
            )),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![gen_candidate!(["n"], ['k']),],
                        gen_candidate!(["n"], ['k'])
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
                    gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![gen_candidate!(["n"], ['k']),],
                        gen_candidate!(["n"], ['k'])
                    ),
                    vec![ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    ),],
                ),
                ConfirmedChunk::new(
                    gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
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
            vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
            gen_candidate!(["ka"])
        ),
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], ['k']),
                gen_candidate!(["nn"]),
                gen_candidate!(["xn"])
            ],
            gen_candidate!(["n"], ['k'])
        ),
        gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!(
                    "か",
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
                    gen_candidate!(["ka"])
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                ),
                gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
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
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                ),
                gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "か",
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
                    gen_candidate!(["ka"])
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
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                ),
                gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
            ]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!("か", vec![gen_candidate!(["ka"])], gen_candidate!(["ka"])),
                vec![1],
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
                vec![gen_candidate!(["ki"])],
                gen_candidate!(["ki"])
            )]
            .into(),
            inflight_chunk: Some(
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], ['k']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['k'])
                )
                .into()
            ),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
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
                vec![gen_candidate!(["ki"])],
                gen_candidate!(["ki"])
            )]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['k']), gen_candidate!(["nn"]),],
                    gen_candidate!(["n"], ['k'])
                )
                .into(),
                vec![1, 1],
                vec![ActualKeyStroke::new(
                    Duration::new(3, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
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
                vec![gen_candidate!(["ki"])],
                gen_candidate!(["ki"])
            )]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['k']), gen_candidate!(["nn"]),],
                    gen_candidate!(["n"], ['k'])
                )
                .into(),
                vec![1, 1],
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
                gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
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
                gen_chunk!("き", vec![gen_candidate!(["ki"])], gen_candidate!(["ki"])).into()
            ),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![gen_candidate!(["nn"]),],
                        gen_candidate!(["n"], ['k'])
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
                gen_chunk!("き", vec![gen_candidate!(["ki"])], gen_candidate!(["ki"])),
                vec![1],
                vec![ActualKeyStroke::new(
                    Duration::new(6, 0),
                    'k'.try_into().unwrap(),
                    true
                ),],
                vec![]
            )),
            confirmed_chunks: vec![
                ConfirmedChunk::new(
                    gen_chunk!("か", vec![gen_candidate!(["ka"]),], gen_candidate!(["ka"])),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![gen_candidate!(["nn"]),],
                        gen_candidate!(["n"], ['k'])
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
                    gen_chunk!("か", vec![gen_candidate!(["ka"])], gen_candidate!(["ka"])),
                    vec![
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!(
                        "ん",
                        vec![gen_candidate!(["nn"])],
                        gen_candidate!(["n"], ['k'])
                    ),
                    vec![
                        ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'n'.try_into().unwrap(), true),
                    ],
                ),
                ConfirmedChunk::new(
                    gen_chunk!("き", vec![gen_candidate!(["ki"]),], gen_candidate!(["ki"])),
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
fn construct_display_info_1() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"]),
                gen_candidate!(["ki", "lyo"]),
                gen_candidate!(["ki", "xyo"])
            ],
            gen_candidate!(["kyo"])
        ),
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"]),
                gen_candidate!(["ki", "lyo"]),
                gen_candidate!(["ki", "xyo"])
            ],
            gen_candidate!(["kyo"])
        ),
        gen_chunk!(
            "きょ",
            vec![
                gen_candidate!(["kyo"]),
                gen_candidate!(["ki", "lyo"]),
                gen_candidate!(["ki", "xyo"])
            ],
            gen_candidate!(["kyo"])
        ),
        gen_chunk!(
            "きょ",
            vec![gen_candidate!(["ky"]), gen_candidate!(["ki"]),],
            gen_candidate!(["ky"])
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
                vec![gen_candidate!(["ky"]), gen_candidate!(["ki"]),],
                gen_candidate!(["ky"])
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "きょ",
                    vec![
                        gen_candidate!(["kyo"]),
                        gen_candidate!(["ki", "lyo"]),
                        gen_candidate!(["ki", "xyo"]),
                    ],
                    gen_candidate!(["kyo"])
                ),
                vec![1, 1, 1],
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
                        vec![gen_candidate!(["kyo"]),],
                        gen_candidate!(["kyo"])
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
                        vec![gen_candidate!(["ki", "xyo"]),],
                        gen_candidate!(["kyo"])
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

    let (sdi, ksdi) = pci.construct_display_info();

    assert_eq!(
        sdi,
        SpellDisplayInfo::new(
            "きょきょきょきょ".to_string(),
            vec![4, 5],
            vec![0, 1, 3, 4, 5],
            7
        )
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "kyokixyokyoky".to_string(),
            9,
            vec![1, 5, 8],
            OnTypingStatisticsDynamicTarget::new(9, 11, 13, 6, 3)
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
                gen_candidate!(["n"], ['z', 'j']),
                gen_candidate!(["nn"]),
                gen_candidate!(["xn"])
            ],
            gen_candidate!(["n"], ['z', 'j'])
        ),
        gen_chunk!(
            "じ",
            vec![gen_candidate!(["zi"]), gen_candidate!(["ji"]),],
            gen_candidate!(["zi"])
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
                vec![gen_candidate!(["zi"]), gen_candidate!(["ji"]),],
                gen_candidate!(["zi"])
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['z', 'j']), gen_candidate!(["nn"]),],
                    gen_candidate!(["n"], ['z', 'j'])
                ),
                vec![1, 1],
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

    let (sdi, ksdi) = pci.construct_display_info();

    // 入力を終えた遅延確定候補は表示の上では確定したとみなす
    // pendingにあるミスタイプは表示状は次のチャンクに帰属させる
    assert_eq!(
        sdi,
        SpellDisplayInfo::new("んじ".to_string(), vec![1], vec![1], 1)
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nzi".to_string(),
            1,
            vec![1],
            OnTypingStatisticsDynamicTarget::new(1, 3, 3, 1, 1)
        )
    );

    // 4. jと入力
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(3, 0));

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!("じ", vec![gen_candidate!(["ji"]),], gen_candidate!(["zi"])),
                vec![1],
                vec![
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), true),
                ],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['z', 'j'])],
                    gen_candidate!(["n"], ['z', 'j'])
                ),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                ),],
            )],
        }
    );

    let (sdi, ksdi) = pci.construct_display_info();

    // 遅延確定候補で確定したのでミスタイプは引き続き次のチャンクに属する
    assert_eq!(
        sdi,
        SpellDisplayInfo::new("んじ".to_string(), vec![1], vec![1], 1)
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nji".to_string(),
            2,
            vec![1],
            OnTypingStatisticsDynamicTarget::new(2, 3, 3, 1, 1)
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
                gen_candidate!(["n"], ['z', 'j']),
                gen_candidate!(["nn"]),
                gen_candidate!(["xn"])
            ],
            gen_candidate!(["n"], ['z', 'j'])
        ),
        gen_chunk!(
            "じ",
            vec![gen_candidate!(["zi"]), gen_candidate!(["ji"]),],
            gen_candidate!(["zi"])
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
                vec![gen_candidate!(["zi"]), gen_candidate!(["ji"]),],
                gen_candidate!(["zi"])
            ),]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["n"], ['z', 'j']), gen_candidate!(["nn"]),],
                    gen_candidate!(["n"], ['z', 'j'])
                ),
                vec![1, 1],
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

    let (sdi, ksdi) = pci.construct_display_info();

    // 入力を終えた遅延確定候補は表示の上では確定したとみなす
    // pendingにあるミスタイプは表示状は次のチャンクに帰属させる
    assert_eq!(
        sdi,
        SpellDisplayInfo::new("んじ".to_string(), vec![1], vec![1], 1)
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nzi".to_string(),
            1,
            vec![1],
            OnTypingStatisticsDynamicTarget::new(1, 3, 3, 1, 1)
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
                    vec![gen_candidate!(["zi"]), gen_candidate!(["ji"])],
                    gen_candidate!(["zi"])
                ),
                vec![0, 0],
                vec![],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["nn"])],
                    gen_candidate!(["n"], ['z', 'j'])
                ),
                vec![
                    ActualKeyStroke::new(Duration::new(1, 0), 'n'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                ],
            )],
        }
    );

    let (sdi, ksdi) = pci.construct_display_info();

    // 遅延確定候補ではない候補で確定したのでミスタイプはその候補に属する
    assert_eq!(
        sdi,
        SpellDisplayInfo::new("んじ".to_string(), vec![1], vec![0], 1)
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "nnzi".to_string(),
            2,
            vec![1],
            OnTypingStatisticsDynamicTarget::new(2, 4, 3, 0, 1)
        )
    );
}

#[test]
fn construct_display_info_4() {
    // 1. 初期化
    let mut pci = ProcessedChunkInfo::new(vec![
        gen_chunk!("あ", vec![gen_candidate!(["a"])], gen_candidate!(["a"])),
        gen_chunk!(
            "っ",
            vec![
                gen_candidate!(["k"], 'k', ['k']),
                gen_candidate!(["c"], 'c', ['c']),
                gen_candidate!(["ltu"]),
                gen_candidate!(["xtu"]),
                gen_candidate!(["ltsu"])
            ],
            gen_candidate!(["k"], 'k', ['k'])
        ),
        gen_chunk!(
            "か",
            vec![gen_candidate!(["ka"]), gen_candidate!(["ca"])],
            gen_candidate!(["ka"])
        ),
        gen_chunk!(
            "ん",
            vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])],
            gen_candidate!(["nn"])
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk!("あ", vec![gen_candidate!(["a"])], gen_candidate!(["a"])),
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["k"], 'k', ['k']),
                        gen_candidate!(["c"], 'c', ['c']),
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"])
                    ],
                    gen_candidate!(["k"], 'k', ['k'])
                ),
                gen_chunk!(
                    "か",
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"])],
                    gen_candidate!(["ka"])
                ),
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])],
                    gen_candidate!(["nn"])
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
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"])],
                    gen_candidate!(["ka"])
                ),
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])],
                    gen_candidate!(["nn"])
                ),
            ]
            .into(),
            inflight_chunk: Some(TypedChunk::new(
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["k"], 'k', ['k']),
                        gen_candidate!(["c"], 'c', ['c']),
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"])
                    ],
                    gen_candidate!(["k"], 'k', ['k'])
                ),
                vec![0, 0, 0, 0, 0],
                vec![],
                vec![]
            )),
            confirmed_chunks: vec![ConfirmedChunk::new(
                gen_chunk!("あ", vec![gen_candidate!(["a"])], gen_candidate!(["a"])),
                vec![ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'a'.try_into().unwrap(),
                    true
                ),],
            )],
        }
    );

    let (sdi, ksdi) = pci.construct_display_info();

    assert_eq!(
        sdi,
        SpellDisplayInfo::new("あっかん".to_string(), vec![1], vec![], 3)
    );

    assert_eq!(
        ksdi,
        KeyStrokeDisplayInfo::new(
            "akkann".to_string(),
            1,
            vec![],
            OnTypingStatisticsDynamicTarget::new(1, 6, 6, 1, 0)
        )
    );
}
