use super::*;

use std::num::NonZeroUsize;
use std::time::Duration;

use crate::statistics::statistical_event::{ChunkAddedContext, ChunkConfirmationInfo};
use crate::statistics::statistics_counter::{PrimitiveStatisticsCounter, StatisticsCounter};
use crate::statistics::OnTypingStatisticsTarget;
use crate::typing_engine::processed_chunk_info::KeyStrokeDisplayInfo;
use crate::typing_engine::processed_chunk_info::SpellDisplayInfo;
use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::{
    gen_candidate, gen_candidate_key_stroke, gen_chunk_confirmed, gen_chunk_inflight,
    gen_chunk_unprocessed,
};

#[test]
fn stroke_key_1() {
    // 1. 初期化
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "う",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("u"), false),
                gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                gen_candidate!(gen_candidate_key_stroke!("whu"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("u"), false)
        ),
        gen_chunk_unprocessed!(
            "っ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w')
        ),
        gen_chunk_unprocessed!(
            "う",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("u"), false),
                gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                gen_candidate!(gen_candidate_key_stroke!("whu"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("wu"), false)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "う",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("u"), false),
                        gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("u"), false)
                ),
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w')
                ),
                gen_chunk_unprocessed!(
                    "う",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("u"), false),
                        gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false)
                ),
            ]
            .into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w')
                ),
                gen_chunk_unprocessed!(
                    "う",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("u"), false),
                        gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false)
                ),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "う",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("u"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("wu"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), true, 0)
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「u」と入力
    let (result, events) = pci.stroke_key('u'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "う",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                ],
                gen_candidate!(gen_candidate_key_stroke!("wu"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "っ",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("w"), true, 0, 'w'),
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), true, 0)
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "う",
                gen_candidate!(gen_candidate_key_stroke!("u"), false),
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                ],
                gen_candidate!(gen_candidate_key_stroke!("u"), false),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'u'.try_into().unwrap(),
                    true
                )]
            )],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「w」と入力
    let (result, events) = pci.stroke_key('w'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "う",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("wu"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), true, 0)
                    ],
                    vec![gen_candidate!(gen_candidate_key_stroke!("u"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "う",
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    [ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "っ",
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    [ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    // 4. 「w」と入力
    let (result, events) = pci.stroke_key('w'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "う",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("wu"), true, 1),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), true, 1)
                    ],
                    vec![gen_candidate!(gen_candidate_key_stroke!("u"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    [ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'w'.try_into().unwrap(),
                        true
                    )]
                )
                .into(),
            ),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "う",
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    [ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "っ",
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    [ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    // 5. 「u」と入力
    let (result, events) = pci.stroke_key('u'.try_into().unwrap(), Duration::new(4, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: None,
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "う",
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    [ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "っ",
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    [ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "う",
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("u"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    [
                        ActualKeyStroke::new(Duration::new(3, 0), 'w'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'u'.try_into().unwrap(), true)
                    ]
                )
            ],
            pending_key_strokes: vec![],
        }
    );

    assert!(pci.is_finished());

    let statistical_events = pci.append_chunks(vec![gen_chunk_unprocessed!(
        "う",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("u"), false),
            gen_candidate!(gen_candidate_key_stroke!("wu"), false),
            gen_candidate!(gen_candidate_key_stroke!("whu"), false)
        ],
        gen_candidate!(gen_candidate_key_stroke!("u"), false)
    )]);

    assert_eq!(
        statistical_events,
        vec![StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
            1,
            KeyStrokeElementCount::Sigle(1)
        )),],
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "う",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("u"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("wu"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), true, 0)
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "う",
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("u"), false),
                    [ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'u'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "っ",
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("w"), false, 'w'),
                    [ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'w'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "う",
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("u"), false),
                        gen_candidate!(gen_candidate_key_stroke!("whu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("wu"), false),
                    [
                        ActualKeyStroke::new(Duration::new(3, 0), 'w'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'u'.try_into().unwrap(), true)
                    ]
                )
            ],
            pending_key_strokes: vec![],
        }
    );
}

#[test]
fn stroke_key_2() {
    // 1. 初期化
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "か",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                gen_candidate!(gen_candidate_key_stroke!("ca"), false),
            ],
            gen_candidate!(gen_candidate_key_stroke!("ka"), false)
        ),
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                gen_candidate!(gen_candidate_key_stroke!("xn"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
        ),
        gen_chunk_unprocessed!(
            "き",
            vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
            gen_candidate!(gen_candidate_key_stroke!("ki"), false)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ca"), false),
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                ),
                gen_chunk_unprocessed!(
                    "き",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false)
                ),
            ]
            .into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                ),
                gen_chunk_unprocessed!(
                    "き",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false)
                ),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("ca"), true, 0),
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「k」と入力
    let (result, events) = pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                ),
                gen_chunk_unprocessed!(
                    "き",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false)
                ),
            ]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "か",
                vec![gen_candidate!(gen_candidate_key_stroke!("ka"), true, 1),],
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'k'.try_into().unwrap(),
                    true
                )]
            )),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「a」と入力
    let (result, events) = pci.stroke_key('a'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, 0, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, 0)
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "か",
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ]
            ),],
            pending_key_strokes: vec![],
        }
    );

    // 4. 「n」と入力
    let (result, events) = pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, 1, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, 1),
                    ],
                    vec![gen_candidate!(gen_candidate_key_stroke!("xn"), false)],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    )]
                )
                .into()
            ),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "か",
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ]
            )],
            pending_key_strokes: vec![],
        }
    );

    // 5. 「j」と入力（ミスタイプ）
    // 遅延確定候補が確定していないのでミスタイプはどのチャンクにも属さない
    let (result, events) = pci.stroke_key('j'.try_into().unwrap(), Duration::new(4, 0));
    assert_eq!(result, KeyStrokeResult::Wrong);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, 1, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, 1),
                    ],
                    vec![gen_candidate!(gen_candidate_key_stroke!("xn"), false)],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    )]
                )
                .into(),
            ),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "か",
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ]
            )],
            pending_key_strokes: vec![ActualKeyStroke::new(
                Duration::new(4, 0),
                'j'.try_into().unwrap(),
                false
            )],
        }
    );

    // 6. 「k」と入力
    // 遅延確定候補が確定したのでミスタイプは次のチャンクに属する
    let (result, events) = pci.stroke_key('k'.try_into().unwrap(), Duration::new(5, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), true, 1),],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false),
                [
                    ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(5, 0), 'k'.try_into().unwrap(), true)
                ]
            ),),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "か",
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    [
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "ん",
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    )]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    // 7. 「i」と入力
    let (result, events) = pci.stroke_key('i'.try_into().unwrap(), Duration::new(6, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(false, None), (true, None), (true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: None,
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "か",
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    [
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "ん",
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "き",
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false),
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false),
                    [
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(6, 0), 'i'.try_into().unwrap(), true)
                    ]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    assert!(pci.is_finished());
}

#[test]
fn stroke_key_3() {
    // 1. 初期化
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "か",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                gen_candidate!(gen_candidate_key_stroke!("ca"), false),
            ],
            gen_candidate!(gen_candidate_key_stroke!("ka"), false)
        ),
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                gen_candidate!(gen_candidate_key_stroke!("xn"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
        ),
        gen_chunk_unprocessed!(
            "き",
            vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
            gen_candidate!(gen_candidate_key_stroke!("ki"), false)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ca"), false),
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                ),
                gen_chunk_unprocessed!(
                    "き",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false)
                ),
            ]
            .into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                ),
                gen_chunk_unprocessed!(
                    "き",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false)
                ),
            ]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("ca"), true, 0),
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「k」と入力
    let (result, events) = pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                ),
                gen_chunk_unprocessed!(
                    "き",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false)
                ),
            ]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "か",
                vec![gen_candidate!(gen_candidate_key_stroke!("ka"), true, 1),],
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'k'.try_into().unwrap(),
                    true
                )]
            )),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「a」と入力
    let (result, events) = pci.stroke_key('a'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, 0, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, 0)
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "か",
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ]
            ),],
            pending_key_strokes: vec![],
        }
    );

    // 4. 「n」と入力
    let (result, events) = pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, 1, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, 1),
                    ],
                    vec![gen_candidate!(gen_candidate_key_stroke!("xn"), false)],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    )]
                )
                .into(),
            ),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "か",
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ]
            )],
            pending_key_strokes: vec![],
        }
    );

    // 5. 「j」と入力（ミスタイプ）
    // 遅延確定候補が確定していないのでミスタイプはどのチャンクにも属さない
    let (result, events) = pci.stroke_key('j'.try_into().unwrap(), Duration::new(4, 0));
    assert_eq!(result, KeyStrokeResult::Wrong);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, 1, ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, 1),
                    ],
                    vec![gen_candidate!(gen_candidate_key_stroke!("xn"), false)],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [ActualKeyStroke::new(
                        Duration::new(3, 0),
                        'n'.try_into().unwrap(),
                        true
                    )]
                )
                .into(),
            ),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "か",
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                ]
            )],
            pending_key_strokes: vec![ActualKeyStroke::new(
                Duration::new(4, 0),
                'j'.try_into().unwrap(),
                false
            )],
        }
    );

    // 6. 「n」と入力
    // 遅延確定候補でない候補で確定したのでミスタイプはそのチャンクに属する
    let (result, events) = pci.stroke_key('n'.try_into().unwrap(), Duration::new(5, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(1),
                1,
                2,
                1,
                1,
                vec![(true, None), (false, None), (true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "き",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ki"), true, 0),],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "か",
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    [
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "ん",
                    gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [
                        ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'n'.try_into().unwrap(), true)
                    ]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    // 7. 「k」と入力
    let (result, events) = pci.stroke_key('k'.try_into().unwrap(), Duration::new(6, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "き",
                vec![gen_candidate!(gen_candidate_key_stroke!("ki"), true, 1),],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("ki"), false),
                [ActualKeyStroke::new(
                    Duration::new(6, 0),
                    'k'.try_into().unwrap(),
                    true
                )]
            ),),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "か",
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    [
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "ん",
                    gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [
                        ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'n'.try_into().unwrap(), true)
                    ]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    // 8. 「i」と入力
    let (result, events) = pci.stroke_key('i'.try_into().unwrap(), Duration::new(7, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::ChunkConfirmed(
            ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(2),
                KeyStrokeElementCount::Sigle(2),
                1,
                2,
                2,
                1,
                vec![(true, None), (true, Some(1))]
            )
        )]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: None,
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "か",
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    vec![gen_candidate!(gen_candidate_key_stroke!("ca"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                    [
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'a'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "ん",
                    gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k'])
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['k']),
                    [
                        ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(5, 0), 'n'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "き",
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false),
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("ki"), false),
                    [
                        ActualKeyStroke::new(Duration::new(6, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(7, 0), 'i'.try_into().unwrap(), true)
                    ]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    assert!(pci.is_finished());
}

#[test]
fn stroke_key_4() {
    // 1. 初期化
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p']),
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                gen_candidate!(gen_candidate_key_stroke!("xn"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p'])
        ),
        gen_chunk_unprocessed!(
            "ぴ",
            vec![gen_candidate!(gen_candidate_key_stroke!("p"), false),],
            gen_candidate!(gen_candidate_key_stroke!("p"), false)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p'])
                ),
                gen_chunk_unprocessed!(
                    "ぴ",
                    vec![gen_candidate!(gen_candidate_key_stroke!("p"), false),],
                    gen_candidate!(gen_candidate_key_stroke!("p"), false)
                ),
            ]
            .into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);
    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "ぴ",
                vec![gen_candidate!(gen_candidate_key_stroke!("p"), false),],
                gen_candidate!(gen_candidate_key_stroke!("p"), false)
            ),]
            .into(),
            inflight_chunk: Some(
                gen_chunk_inflight!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, 0, ['p']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, 0),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, 0)
                    ],
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p']),
                    []
                )
                .into()
            ),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「n」と入力
    let (result, events) = pci.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(events, vec![]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "ぴ",
                vec![gen_candidate!(gen_candidate_key_stroke!("p"), false),],
                gen_candidate!(gen_candidate_key_stroke!("p"), false)
            ),]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "ん",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("n"), true, 1, ['p']),
                    gen_candidate!(gen_candidate_key_stroke!("nn"), true, 1),
                ],
                vec![gen_candidate!(gen_candidate_key_stroke!("xn"), false)],
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p']),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )]
            )),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    // 3. 「p」と入力
    let (result, events) = pci.stroke_key('p'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(result, KeyStrokeResult::Correct);
    assert_eq!(
        events,
        vec![
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))]
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmationInfo::new(
                KeyStrokeElementCount::Sigle(1),
                KeyStrokeElementCount::Sigle(1),
                1,
                1,
                1,
                1,
                vec![(true, Some(1))]
            ))
        ]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: None,
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "ん",
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p']),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['p']),
                    [ActualKeyStroke::new(
                        Duration::new(1, 0),
                        'n'.try_into().unwrap(),
                        true
                    )]
                ),
                gen_chunk_confirmed!(
                    "ぴ",
                    gen_candidate!(gen_candidate_key_stroke!("p"), false),
                    vec![],
                    gen_candidate!(gen_candidate_key_stroke!("p"), false),
                    [ActualKeyStroke::new(
                        Duration::new(2, 0),
                        'p'.try_into().unwrap(),
                        true
                    )]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    assert!(pci.is_finished());
}

#[test]
fn construct_display_info_1() {
    // 1. 初期化
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false)
        ),
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false)
        ),
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false)
        ),
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["ky"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ki"]), false),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["ky"]), false)
        ),
    ]);

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                2,
                KeyStrokeElementCount::Sigle(2)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);

    // 3. k -> u(ミスタイプ) -> y -> o -> k -> i -> j(ミスタイプ) -> x -> y -> o -> c(ミスタイプ) -> k という順で入力
    let mut results = vec![];
    results.push(pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0)));
    results.push(pci.stroke_key('u'.try_into().unwrap(), Duration::new(2, 0)));
    results.push(pci.stroke_key('y'.try_into().unwrap(), Duration::new(3, 0)));
    results.push(pci.stroke_key('o'.try_into().unwrap(), Duration::new(4, 0)));
    results.push(pci.stroke_key('k'.try_into().unwrap(), Duration::new(5, 0)));
    results.push(pci.stroke_key('i'.try_into().unwrap(), Duration::new(6, 0)));
    results.push(pci.stroke_key('j'.try_into().unwrap(), Duration::new(7, 0)));
    results.push(pci.stroke_key('x'.try_into().unwrap(), Duration::new(8, 0)));
    results.push(pci.stroke_key('y'.try_into().unwrap(), Duration::new(9, 0)));
    results.push(pci.stroke_key('o'.try_into().unwrap(), Duration::new(10, 0)));
    results.push(pci.stroke_key('c'.try_into().unwrap(), Duration::new(11, 0)));
    results.push(pci.stroke_key('k'.try_into().unwrap(), Duration::new(12, 0)));

    assert_eq!(
        results,
        vec![
            (KeyStrokeResult::Correct, vec![]),
            (KeyStrokeResult::Wrong, vec![]),
            (KeyStrokeResult::Correct, vec![]),
            (
                KeyStrokeResult::Correct,
                vec![StatisticalEvent::ChunkConfirmed(
                    ChunkConfirmationInfo::new(
                        KeyStrokeElementCount::Sigle(3),
                        KeyStrokeElementCount::Sigle(3),
                        2,
                        3,
                        3,
                        2,
                        vec![(true, None), (false, None), (true, None), (true, Some(2))]
                    )
                )]
            ),
            (KeyStrokeResult::Correct, vec![]),
            (KeyStrokeResult::Correct, vec![]),
            (KeyStrokeResult::Wrong, vec![]),
            (KeyStrokeResult::Correct, vec![]),
            (KeyStrokeResult::Correct, vec![]),
            (
                KeyStrokeResult::Correct,
                vec![StatisticalEvent::ChunkConfirmed(
                    ChunkConfirmationInfo::new(
                        KeyStrokeElementCount::Double((2, 3)),
                        KeyStrokeElementCount::Sigle(3),
                        2,
                        5,
                        3,
                        1,
                        vec![
                            (true, None),
                            (true, Some(1)),
                            (false, None),
                            (true, None),
                            (true, None),
                            (true, Some(1))
                        ]
                    )
                )]
            ),
            (KeyStrokeResult::Wrong, vec![]),
            (KeyStrokeResult::Correct, vec![]),
        ]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "きょ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!(["ky"]), false),
                    gen_candidate!(gen_candidate_key_stroke!(["ki"]), false),
                ],
                gen_candidate!(gen_candidate_key_stroke!(["ky"]), false)
            ),]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "きょ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"]), true, 1),
                    gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"]), true, 1),
                    gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]), true, 1),
                ],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                [
                    ActualKeyStroke::new(Duration::new(11, 0), 'c'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(12, 0), 'k'.try_into().unwrap(), true)
                ]
            ),),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "きょ",
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"]), false),
                        gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]), false),
                    ],
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                    [
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'u'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(3, 0), 'y'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'o'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "きょ",
                    gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]), false),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                        gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"]), false),
                    ],
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"]), false),
                    [
                        ActualKeyStroke::new(Duration::new(5, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(6, 0), 'i'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(7, 0), 'j'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(8, 0), 'x'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(9, 0), 'y'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(10, 0), 'o'.try_into().unwrap(), true)
                    ]
                ),
            ],
            pending_key_strokes: vec![],
        }
    );

    let confirmed_only_statistics_counter = StatisticsCounter::new_with_values(
        PrimitiveStatisticsCounter::new(8, 8, 6, 2),
        PrimitiveStatisticsCounter::new(6, 6, 4, 2),
        PrimitiveStatisticsCounter::new(4, 4, 1, 3),
        PrimitiveStatisticsCounter::new(2, 2, 0, 2),
        false,
        false,
        false,
        false,
        None,
        None,
        0,
    );

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (_, ksdi) = pci.construct_display_info(
        LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::Spell(NonZeroUsize::new(1).unwrap()),
        &confirmed_only_statistics_counter,
    );

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
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j']),
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                gen_candidate!(gen_candidate_key_stroke!("xn"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j'])
        ),
        gen_chunk_unprocessed!(
            "じ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("zi"), false),
                gen_candidate!(gen_candidate_key_stroke!("ji"), false),
            ],
            gen_candidate!(gen_candidate_key_stroke!("zi"), false)
        ),
    ]);

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);

    // 3. n -> m(ミスタイプ) と入力
    let mut results = vec![];

    results.push(pci.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0)));
    results.push(pci.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0)));

    assert_eq!(
        results,
        vec![
            (KeyStrokeResult::Correct, vec![]),
            (KeyStrokeResult::Wrong, vec![]),
        ]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "じ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("zi"), false),
                    gen_candidate!(gen_candidate_key_stroke!("ji"), false),
                ],
                gen_candidate!(gen_candidate_key_stroke!("zi"), false)
            ),]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "ん",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("n"), true, 1, ['z', 'j']),
                    gen_candidate!(gen_candidate_key_stroke!("nn"), true, 1),
                ],
                vec![gen_candidate!(gen_candidate_key_stroke!("xn"), false)],
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j']),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )]
            )),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![ActualKeyStroke::new(
                Duration::new(2, 0),
                'm'.try_into().unwrap(),
                false
            ),],
        }
    );

    let confirmed_only_statistics_counter = StatisticsCounter::new();

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (_, ksdi) = pci.construct_display_info(
        LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::Spell(NonZeroUsize::new(1).unwrap()),
        &confirmed_only_statistics_counter,
    );

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
            inflight_chunk: Some(gen_chunk_inflight!(
                "じ",
                vec![gen_candidate!(gen_candidate_key_stroke!("ji"), true, 1),],
                vec![gen_candidate!(gen_candidate_key_stroke!("zi"), false),],
                gen_candidate!(gen_candidate_key_stroke!("zi"), false),
                [
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), true)
                ]
            )),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "ん",
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j']),
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                    gen_candidate!(gen_candidate_key_stroke!("nn"), false)
                ],
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j']),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )]
            ),],
            pending_key_strokes: vec![],
        }
    );

    let confirmed_only_statistics_counter = StatisticsCounter::new_with_values(
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        false,
        false,
        false,
        false,
        None,
        None,
        0,
    );

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (_, ksdi) = pci.construct_display_info(
        LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::Spell(NonZeroUsize::new(1).unwrap()),
        &confirmed_only_statistics_counter,
    );

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
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j']),
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                gen_candidate!(gen_candidate_key_stroke!("xn"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j'])
        ),
        gen_chunk_unprocessed!(
            "じ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("zi"), false),
                gen_candidate!(gen_candidate_key_stroke!("ji"), false),
            ],
            gen_candidate!(gen_candidate_key_stroke!("zi"), false)
        ),
    ]);

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);

    // 3. n -> m(ミスタイプ) と入力
    let mut results = vec![];

    results.push(pci.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0)));
    results.push(pci.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0)));

    assert_eq!(
        results,
        vec![
            (KeyStrokeResult::Correct, vec![]),
            (KeyStrokeResult::Wrong, vec![]),
        ]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "じ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("zi"), false),
                    gen_candidate!(gen_candidate_key_stroke!("ji"), false),
                ],
                gen_candidate!(gen_candidate_key_stroke!("zi"), false)
            ),]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "ん",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("n"), true, 1, ['z', 'j']),
                    gen_candidate!(gen_candidate_key_stroke!("nn"), true, 1),
                ],
                vec![gen_candidate!(gen_candidate_key_stroke!("xn"), false)],
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j']),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )]
            )),
            confirmed_chunks: vec![],
            pending_key_strokes: vec![ActualKeyStroke::new(
                Duration::new(2, 0),
                'm'.try_into().unwrap(),
                false
            ),],
        }
    );

    let confirmed_only_statistics_counter = StatisticsCounter::new();

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::Spell(NonZeroUsize::new(1).unwrap()),
        &confirmed_only_statistics_counter,
    );

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
    assert_eq!(
        pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0)),
        (
            KeyStrokeResult::Correct,
            vec![StatisticalEvent::ChunkConfirmed(
                ChunkConfirmationInfo::new(
                    KeyStrokeElementCount::Sigle(2),
                    KeyStrokeElementCount::Sigle(1),
                    1,
                    2,
                    1,
                    1,
                    vec![(true, None), (false, None), (true, Some(1))]
                )
            )]
        )
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "じ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("zi"), true, 0),
                    gen_candidate!(gen_candidate_key_stroke!("ji"), true, 0),
                ],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("zi"), false),
                []
            ),),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "ん",
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("xn"), false),
                    gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j'])
                ],
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['z', 'j']),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'n'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true)
                ]
            )],
            pending_key_strokes: vec![],
        }
    );

    let confirmed_only_statistics_counter = StatisticsCounter::new_with_values(
        PrimitiveStatisticsCounter::new(2, 2, 1, 1),
        PrimitiveStatisticsCounter::new(1, 1, 0, 1),
        PrimitiveStatisticsCounter::new(1, 1, 0, 1),
        PrimitiveStatisticsCounter::new(1, 1, 0, 1),
        false,
        false,
        false,
        false,
        None,
        None,
        0,
    );

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (_, ksdi) = pci.construct_display_info(
        LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::Spell(NonZeroUsize::new(1).unwrap()),
        &confirmed_only_statistics_counter,
    );

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
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "あ",
            vec![gen_candidate!(gen_candidate_key_stroke!("a"), false)],
            gen_candidate!(gen_candidate_key_stroke!("a"), false)
        ),
        gen_chunk_unprocessed!(
            "っ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("k"), false, 'k', ['k']),
                gen_candidate!(gen_candidate_key_stroke!("c"), false, 'c', ['c']),
                gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("k"), false, 'k', ['k'])
        ),
        gen_chunk_unprocessed!(
            "か",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                gen_candidate!(gen_candidate_key_stroke!("ca"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("ka"), false)
        ),
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                gen_candidate!(gen_candidate_key_stroke!("xn"), false)
            ],
            gen_candidate!(gen_candidate_key_stroke!("nn"), false)
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "あ",
                    vec![gen_candidate!(gen_candidate_key_stroke!("a"), false)],
                    gen_candidate!(gen_candidate_key_stroke!("a"), false)
                ),
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("k"), false, 'k', ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("c"), false, 'c', ['c']),
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("k"), false, 'k', ['k'])
                ),
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ca"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("nn"), false)
                ),
            ]
            .into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
            pending_key_strokes: vec![],
        }
    );

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                1,
                KeyStrokeElementCount::Sigle(2)
            ))
        ],
    );

    // 2. タイピング開始
    assert_eq!(pci.move_next_chunk(), None);

    // 3. a と入力
    assert_eq!(
        pci.stroke_key('a'.try_into().unwrap(), Duration::new(1, 0)),
        (
            KeyStrokeResult::Correct,
            vec![StatisticalEvent::ChunkConfirmed(
                ChunkConfirmationInfo::new(
                    KeyStrokeElementCount::Sigle(1),
                    KeyStrokeElementCount::Sigle(1),
                    1,
                    1,
                    1,
                    1,
                    vec![(true, Some(1))]
                )
            )]
        )
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka"), false),
                        gen_candidate!(gen_candidate_key_stroke!("ca"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), false)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), false)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("nn"), false)
                ),
            ]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "っ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("k"), true, 0, 'k', ['k']),
                    gen_candidate!(gen_candidate_key_stroke!("c"), true, 0, 'c', ['c']),
                    gen_candidate!(gen_candidate_key_stroke!("ltu"), true, 0),
                    gen_candidate!(gen_candidate_key_stroke!("xtu"), true, 0),
                    gen_candidate!(gen_candidate_key_stroke!("ltsu"), true, 0)
                ],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("k"), false, 'k', ['k']),
                []
            ),),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "あ",
                gen_candidate!(gen_candidate_key_stroke!("a"), false),
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("a"), false),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'a'.try_into().unwrap(),
                    true
                )]
            ),],
            pending_key_strokes: vec![],
        }
    );

    let confirmed_only_statistics_counter = StatisticsCounter::new_with_values(
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        PrimitiveStatisticsCounter::new(1, 1, 1, 0),
        false,
        false,
        false,
        false,
        None,
        None,
        0,
    );

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (_, ksdi) = pci.construct_display_info(
        LapRequest::IdealKeyStroke(NonZeroUsize::new(2).unwrap()),
        &confirmed_only_statistics_counter,
    );

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

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::Spell(NonZeroUsize::new(1).unwrap()),
        &confirmed_only_statistics_counter,
    );

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
