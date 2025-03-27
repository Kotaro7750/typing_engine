use super::*;

use std::num::NonZeroUsize;
use std::time::Duration;

use crate::statistics::statistical_event::{
    ChunkAddedContext, ChunkConfirmedContext, SpellFinishedContext,
};
use crate::statistics::statistics_counter::PrimitiveStatisticsCounter;
use crate::statistics::OnTypingStatisticsTarget;
use crate::typing_engine::processed_chunk_info::KeyStrokeDisplayInfo;
use crate::typing_engine::processed_chunk_info::SpellDisplayInfo;
use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;
use crate::typing_primitive_types::chunk::ChunkSpell;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::{
    gen_candidate, gen_candidate_key_stroke, gen_chunk_confirmed, gen_chunk_inflight,
    gen_chunk_unprocessed,
};

#[test]
fn empty_processed_chunk_info() {
    let (pci, events) = ProcessedChunkInfo::new(vec![]);
    assert!(pci.is_finished());
    assert!(events.is_empty());
}

#[test]
fn create_processed_chunk_info_returns_chunk_added_events() {
    let (_, events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("kyo")),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!("kyo"))
        ),
        gen_chunk_unprocessed!(
            "う",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("u")),
                gen_candidate!(gen_candidate_key_stroke!("wu")),
                gen_candidate!(gen_candidate_key_stroke!("whu"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("u"))
        ),
    ]);

    assert_eq!(
        events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("う".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(1)
            )),
        ]
    );
}

#[test]
fn append_chunks_to_processed_chunk_info_returns_chunk_added_events() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![]);

    let events = pci.append_chunks(vec![
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("kyo")),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!("kyo"))
        ),
        gen_chunk_unprocessed!(
            "う",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("u")),
                gen_candidate!(gen_candidate_key_stroke!("wu")),
                gen_candidate!(gen_candidate_key_stroke!("whu"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("u"))
        ),
    ]);

    assert_eq!(
        events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("う".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(1)
            )),
        ]
    );
}

#[test]
fn append_chunks_to_finished_processed_chunk_info_become_unfinished() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![]);
    let original_pci = pci.clone();

    pci.append_chunks(vec![gen_chunk_unprocessed!(
        "きょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("kyo")),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!("kyo"))
    )]);

    assert!(original_pci.is_finished());
    assert!(!pci.is_finished());
}

#[test]
fn wrong_stroke_to_processed_chunk_info_returns_no_event() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![gen_chunk_unprocessed!(
        "きょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("kyo")),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!("kyo"))
    )]);

    pci.move_next_chunk();
    let (hit_miss, events) = pci.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Wrong);
    assert_eq!(events, vec![]);
}

#[test]
fn correct_stroke_to_processed_chunk_info_returns_key_stroke_correct_event() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![gen_chunk_unprocessed!(
        "きょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("kyo")),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!("kyo"))
    )]);

    pci.move_next_chunk();
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
    let (hit_miss, events) = pci.stroke_key('k'.try_into().unwrap(), Duration::new(2, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Correct);
    assert_eq!(
        events,
        vec![StatisticalEvent::KeyStrokeCorrect(
            KeyStrokeCorrectContext::new('k'.try_into().unwrap(), vec!['j'.try_into().unwrap()])
        ),]
    );
}

#[test]
fn correct_stroke_to_processed_chunk_info_returns_spell_finished_event() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![gen_chunk_unprocessed!(
        "きょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("kyo")),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!("kyo"))
    )]);
    pci.move_next_chunk();
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(2, 0));

    let (hit_miss, events) = pci.stroke_key('i'.try_into().unwrap(), Duration::new(3, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Correct);
    assert_eq!(
        events,
        vec![
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'i'.try_into().unwrap(),
                vec![]
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("き".to_string().try_into().unwrap()),
                1
            ))
        ]
    );
}

#[test]
fn correct_stroke_to_processed_chunk_info_returns_chunk_confirmed_event() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![gen_chunk_unprocessed!(
        "きょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("kyo")),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!("kyo"))
    )]);
    pci.move_next_chunk();
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(2, 0));
    pci.stroke_key('i'.try_into().unwrap(), Duration::new(3, 0));
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(4, 0));
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(5, 0));
    pci.stroke_key('x'.try_into().unwrap(), Duration::new(6, 0));
    pci.stroke_key('y'.try_into().unwrap(), Duration::new(7, 0));

    let (hit_miss, events) = pci.stroke_key('o'.try_into().unwrap(), Duration::new(8, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Correct);
    assert_eq!(
        events,
        vec![
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'o'.try_into().unwrap(),
                vec![]
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ょ".to_string().try_into().unwrap()),
                2
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(3, vec![1, 0, 2, 0, 0]))
        ]
    );
}

#[test]
fn correct_stroke_to_processed_chunk_info_with_delayed_confirmabed_inflight_chunk_confirms_delayed_confirmed_candidate(
) {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "か",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("ka")),
                gen_candidate!(gen_candidate_key_stroke!("ca")),
            ],
            gen_candidate!(gen_candidate_key_stroke!("ka"))
        ),
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), ['k']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                gen_candidate!(gen_candidate_key_stroke!("xn"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['k'])
        ),
        gen_chunk_unprocessed!(
            "き",
            vec![gen_candidate!(gen_candidate_key_stroke!("ki")),],
            gen_candidate!(gen_candidate_key_stroke!("ki"))
        ),
    ]);
    pci.move_next_chunk();
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('a'.try_into().unwrap(), Duration::new(2, 0));
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(4, 0)); // wrong

    let (hit_miss, events) = pci.stroke_key('k'.try_into().unwrap(), Duration::new(5, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Correct);
    assert_eq!(
        events,
        vec![
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                0
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                vec!['j'.try_into().unwrap()]
            ))
        ]
    );
}

#[test]
fn correct_stroke_to_processed_chunk_info_with_delayed_confirmabed_inflight_chunk_confirms_not_delayed_confirmed_candidate(
) {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "か",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("ka")),
                gen_candidate!(gen_candidate_key_stroke!("ca")),
            ],
            gen_candidate!(gen_candidate_key_stroke!("ka"))
        ),
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), ['k']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                gen_candidate!(gen_candidate_key_stroke!("xn"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['k'])
        ),
        gen_chunk_unprocessed!(
            "き",
            vec![gen_candidate!(gen_candidate_key_stroke!("ki")),],
            gen_candidate!(gen_candidate_key_stroke!("ki"))
        ),
    ]);
    pci.move_next_chunk();
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('a'.try_into().unwrap(), Duration::new(2, 0));
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    pci.stroke_key('j'.try_into().unwrap(), Duration::new(4, 0)); // wrong

    let (hit_miss, events) = pci.stroke_key('n'.try_into().unwrap(), Duration::new(5, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Correct);
    assert_eq!(
        events,
        vec![
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'n'.try_into().unwrap(),
                vec!['j'.try_into().unwrap()]
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                1
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0, 1])),
        ]
    );
}

#[test]
fn correct_stroke_to_processed_chunk_info_with_delayed_confirmabed_inflight_chunk_confirms_delayed_confirmed_candidate_and_next_chunk(
) {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), ['p']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                gen_candidate!(gen_candidate_key_stroke!("xn"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['p'])
        ),
        gen_chunk_unprocessed!(
            "ぴ",
            vec![gen_candidate!(gen_candidate_key_stroke!("p")),],
            gen_candidate!(gen_candidate_key_stroke!("p"))
        ),
    ]);
    pci.move_next_chunk();
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));

    let (hit_miss, events) = pci.stroke_key('p'.try_into().unwrap(), Duration::new(2, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Correct);
    assert_eq!(
        events,
        vec![
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                0
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
            StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                'p'.try_into().unwrap(),
                vec![]
            )),
            StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                ChunkSpell::new("ぴ".to_string().try_into().unwrap()),
                0
            )),
            StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0])),
        ]
    );
}

#[test]
fn candidate_restriction_during_stroke_to_processed_chunk_info() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "う",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("u")),
                gen_candidate!(gen_candidate_key_stroke!("wu")),
                gen_candidate!(gen_candidate_key_stroke!("whu"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("u"))
        ),
        gen_chunk_unprocessed!(
            "っ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("w"), 'w'),
                gen_candidate!(gen_candidate_key_stroke!("ltu")),
                gen_candidate!(gen_candidate_key_stroke!("xtu")),
                gen_candidate!(gen_candidate_key_stroke!("ltsu"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("w"), 'w')
        ),
        gen_chunk_unprocessed!(
            "う",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("u")),
                gen_candidate!(gen_candidate_key_stroke!("wu")),
                gen_candidate!(gen_candidate_key_stroke!("whu"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("wu"))
        ),
    ]);
    pci.move_next_chunk();
    pci.stroke_key('u'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('w'.try_into().unwrap(), Duration::new(2, 0));

    let (hit_miss, events) = pci.stroke_key('u'.try_into().unwrap(), Duration::new(3, 0));

    assert_eq!(hit_miss, KeyStrokeHitMiss::Wrong);
    assert_eq!(events, vec![]);
}

#[test]
fn after_last_chunk_confirmation_processed_chunk_info_is_finished() {
    let (mut pci, _) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "か",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("ka")),
                gen_candidate!(gen_candidate_key_stroke!("ca")),
            ],
            gen_candidate!(gen_candidate_key_stroke!("ka"))
        ),
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), ['k']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                gen_candidate!(gen_candidate_key_stroke!("xn"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['k'])
        ),
        gen_chunk_unprocessed!(
            "き",
            vec![gen_candidate!(gen_candidate_key_stroke!("ki")),],
            gen_candidate!(gen_candidate_key_stroke!("ki"))
        ),
    ]);

    pci.move_next_chunk();
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(1, 0));
    pci.stroke_key('a'.try_into().unwrap(), Duration::new(2, 0));
    pci.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    pci.stroke_key('k'.try_into().unwrap(), Duration::new(4, 0));
    pci.stroke_key('i'.try_into().unwrap(), Duration::new(5, 0));

    assert!(pci.is_finished());
}

#[test]
fn construct_display_info_1() {
    // 1. 初期化
    let (mut pci, statistical_events) = ProcessedChunkInfo::new(vec![
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]))
            ],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"]))
        ),
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]))
            ],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"]))
        ),
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]))
            ],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"]))
        ),
        gen_chunk_unprocessed!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["ky"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["ky"]))
        ),
    ]);

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(3)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
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
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new('k'.try_into().unwrap(), vec![])
                ),]
            ),
            (KeyStrokeHitMiss::Wrong, vec![]),
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new(
                        'y'.try_into().unwrap(),
                        vec!['u'.try_into().unwrap()]
                    )
                ),]
            ),
            (
                KeyStrokeHitMiss::Correct,
                vec![
                    StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                        'o'.try_into().unwrap(),
                        vec![]
                    )),
                    StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                        ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                        1
                    )),
                    StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(3, vec![0, 1, 0])),
                ]
            ),
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new('k'.try_into().unwrap(), vec![])
                ),]
            ),
            (
                KeyStrokeHitMiss::Correct,
                vec![
                    StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                        'i'.try_into().unwrap(),
                        vec![]
                    )),
                    StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                        ChunkSpell::new("き".to_string().try_into().unwrap()),
                        0
                    )),
                ]
            ),
            (KeyStrokeHitMiss::Wrong, vec![]),
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new(
                        'x'.try_into().unwrap(),
                        vec!['j'.try_into().unwrap()]
                    )
                ),]
            ),
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new('y'.try_into().unwrap(), vec![])
                ),]
            ),
            (
                KeyStrokeHitMiss::Correct,
                vec![
                    StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                        'o'.try_into().unwrap(),
                        vec![]
                    )),
                    StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                        ChunkSpell::new("ょ".to_string().try_into().unwrap()),
                        1
                    )),
                    StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(
                        3,
                        vec![0, 0, 1, 0, 0],
                    )),
                ]
            ),
            (KeyStrokeHitMiss::Wrong, vec![]),
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new(
                        'k'.try_into().unwrap(),
                        vec!['c'.try_into().unwrap()]
                    )
                ),]
            ),
        ]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "きょ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!(["ky"])),
                    gen_candidate!(gen_candidate_key_stroke!(["ki"])),
                ],
                gen_candidate!(gen_candidate_key_stroke!(["ky"]))
            ),]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "きょ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                    gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                    gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
                ],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                [
                    ActualKeyStroke::new(Duration::new(11, 0), 'c'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(12, 0), 'k'.try_into().unwrap(), true)
                ],
                1,
                []
            ),),
            confirmed_chunks: vec![
                gen_chunk_confirmed!(
                    "きょ",
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                        gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
                    ],
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                    [
                        ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(2, 0), 'u'.try_into().unwrap(), false),
                        ActualKeyStroke::new(Duration::new(3, 0), 'y'.try_into().unwrap(), true),
                        ActualKeyStroke::new(Duration::new(4, 0), 'o'.try_into().unwrap(), true)
                    ]
                ),
                gen_chunk_confirmed!(
                    "きょ",
                    gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                        gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                    ],
                    gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
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
        }
    );

    let key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(9, 9, 6, 3);
    let ideal_key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(6, 11, 4, 3);
    let spell_statistics_counter = PrimitiveStatisticsCounter::new(4, 8, 1, 3);

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
                gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                gen_candidate!(gen_candidate_key_stroke!("xn"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j'])
        ),
        gen_chunk_unprocessed!(
            "じ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("zi")),
                gen_candidate!(gen_candidate_key_stroke!("ji")),
            ],
            gen_candidate!(gen_candidate_key_stroke!("zi"))
        ),
    ]);

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("じ".to_string().try_into().unwrap()),
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
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new('n'.try_into().unwrap(), vec![])
                ),]
            ),
            (KeyStrokeHitMiss::Wrong, vec![]),
        ]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "じ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("zi")),
                    gen_candidate!(gen_candidate_key_stroke!("ji")),
                ],
                gen_candidate!(gen_candidate_key_stroke!("zi"))
            ),]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "ん",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                    gen_candidate!(gen_candidate_key_stroke!("nn")),
                ],
                vec![gen_candidate!(gen_candidate_key_stroke!("xn"))],
                gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                1,
                [ActualKeyStroke::new(
                    Duration::new(2, 0),
                    'm'.try_into().unwrap(),
                    false
                )]
            )),
            confirmed_chunks: vec![],
        }
    );

    let key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(1, 1, 1, 0);
    let ideal_key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(0, 3, 0, 0);
    let spell_statistics_counter = PrimitiveStatisticsCounter::new(0, 2, 0, 0);

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
                vec![gen_candidate!(gen_candidate_key_stroke!("ji")),],
                vec![gen_candidate!(gen_candidate_key_stroke!("zi")),],
                gen_candidate!(gen_candidate_key_stroke!("zi")),
                [
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), true)
                ],
                1,
                []
            )),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "ん",
                gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("xn")),
                    gen_candidate!(gen_candidate_key_stroke!("nn"))
                ],
                gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )]
            ),],
        }
    );

    let key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(2, 2, 1, 1);
    let ideal_key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(1, 3, 1, 1);
    let spell_statistics_counter = PrimitiveStatisticsCounter::new(1, 2, 1, 0);

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
                gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                gen_candidate!(gen_candidate_key_stroke!("xn"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j'])
        ),
        gen_chunk_unprocessed!(
            "じ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("zi")),
                gen_candidate!(gen_candidate_key_stroke!("ji")),
            ],
            gen_candidate!(gen_candidate_key_stroke!("zi"))
        ),
    ]);

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("じ".to_string().try_into().unwrap()),
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
            (
                KeyStrokeHitMiss::Correct,
                vec![StatisticalEvent::KeyStrokeCorrect(
                    KeyStrokeCorrectContext::new('n'.try_into().unwrap(), vec![])
                ),]
            ),
            (KeyStrokeHitMiss::Wrong, vec![]),
        ]
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![gen_chunk_unprocessed!(
                "じ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("zi")),
                    gen_candidate!(gen_candidate_key_stroke!("ji")),
                ],
                gen_candidate!(gen_candidate_key_stroke!("zi"))
            ),]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "ん",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                    gen_candidate!(gen_candidate_key_stroke!("nn")),
                ],
                vec![gen_candidate!(gen_candidate_key_stroke!("xn"))],
                gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'n'.try_into().unwrap(),
                    true
                )],
                1,
                [ActualKeyStroke::new(
                    Duration::new(2, 0),
                    'm'.try_into().unwrap(),
                    false
                )]
            )),
            confirmed_chunks: vec![],
        }
    );

    let key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(1, 1, 1, 0);
    let ideal_key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(0, 3, 0, 0);
    let spell_statistics_counter = PrimitiveStatisticsCounter::new(0, 2, 0, 0);

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
            KeyStrokeHitMiss::Correct,
            vec![
                StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                    'n'.try_into().unwrap(),
                    vec!['m'.try_into().unwrap()]
                )),
                StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                    ChunkSpell::new("ん".to_string().try_into().unwrap()),
                    1
                )),
                StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0, 1]))
            ]
        )
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![].into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "じ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("zi")),
                    gen_candidate!(gen_candidate_key_stroke!("ji")),
                ],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("zi")),
                [],
                0,
                []
            ),),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "ん",
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("xn")),
                    gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j'])
                ],
                gen_candidate!(gen_candidate_key_stroke!("n"), ['z', 'j']),
                [
                    ActualKeyStroke::new(Duration::new(1, 0), 'n'.try_into().unwrap(), true),
                    ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                    ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true)
                ]
            )],
        }
    );

    let key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(2, 2, 1, 1);
    let ideal_key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(1, 3, 0, 1);
    let spell_statistics_counter = PrimitiveStatisticsCounter::new(1, 2, 0, 1);

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
            vec![gen_candidate!(gen_candidate_key_stroke!("a"))],
            gen_candidate!(gen_candidate_key_stroke!("a"))
        ),
        gen_chunk_unprocessed!(
            "っ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("k"), 'k', ['k']),
                gen_candidate!(gen_candidate_key_stroke!("c"), 'c', ['c']),
                gen_candidate!(gen_candidate_key_stroke!("ltu")),
                gen_candidate!(gen_candidate_key_stroke!("xtu")),
                gen_candidate!(gen_candidate_key_stroke!("ltsu"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("k"), 'k', ['k'])
        ),
        gen_chunk_unprocessed!(
            "か",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("ka")),
                gen_candidate!(gen_candidate_key_stroke!("ca"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("ka"))
        ),
        gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("nn")),
                gen_candidate!(gen_candidate_key_stroke!("xn"))
            ],
            gen_candidate!(gen_candidate_key_stroke!("nn"))
        ),
    ]);

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "あ",
                    vec![gen_candidate!(gen_candidate_key_stroke!("a"))],
                    gen_candidate!(gen_candidate_key_stroke!("a"))
                ),
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("k"), 'k', ['k']),
                        gen_candidate!(gen_candidate_key_stroke!("c"), 'c', ['c']),
                        gen_candidate!(gen_candidate_key_stroke!("ltu")),
                        gen_candidate!(gen_candidate_key_stroke!("xtu")),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"))
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("k"), 'k', ['k'])
                ),
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka")),
                        gen_candidate!(gen_candidate_key_stroke!("ca"))
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("ka"))
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("nn")),
                        gen_candidate!(gen_candidate_key_stroke!("xn"))
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("nn"))
                ),
            ]
            .into(),
            inflight_chunk: None,
            confirmed_chunks: vec![],
        }
    );

    assert_eq!(
        statistical_events,
        vec![
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("あ".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("っ".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(1)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("か".to_string().try_into().unwrap()),
                KeyStrokeElementCount::Sigle(2)
            )),
            StatisticalEvent::ChunkAdded(ChunkAddedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
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
            KeyStrokeHitMiss::Correct,
            vec![
                StatisticalEvent::KeyStrokeCorrect(KeyStrokeCorrectContext::new(
                    'a'.try_into().unwrap(),
                    vec![]
                )),
                StatisticalEvent::SpellFinished(SpellFinishedContext::new(
                    ChunkSpell::new("あ".to_string().try_into().unwrap()),
                    0
                )),
                StatisticalEvent::ChunkConfirmed(ChunkConfirmedContext::new(1, vec![0]))
            ]
        )
    );

    assert_eq!(
        pci,
        ProcessedChunkInfo {
            unprocessed_chunks: vec![
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka")),
                        gen_candidate!(gen_candidate_key_stroke!("ca"))
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("ka"))
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("nn")),
                        gen_candidate!(gen_candidate_key_stroke!("xn"))
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("nn"))
                ),
            ]
            .into(),
            inflight_chunk: Some(gen_chunk_inflight!(
                "っ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("k"), 'k', ['k']),
                    gen_candidate!(gen_candidate_key_stroke!("c"), 'c', ['c']),
                    gen_candidate!(gen_candidate_key_stroke!("ltu")),
                    gen_candidate!(gen_candidate_key_stroke!("xtu")),
                    gen_candidate!(gen_candidate_key_stroke!("ltsu"))
                ],
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("k"), 'k', ['k']),
                [],
                0,
                []
            ),),
            confirmed_chunks: vec![gen_chunk_confirmed!(
                "あ",
                gen_candidate!(gen_candidate_key_stroke!("a")),
                vec![],
                gen_candidate!(gen_candidate_key_stroke!("a")),
                [ActualKeyStroke::new(
                    Duration::new(1, 0),
                    'a'.try_into().unwrap(),
                    true
                )]
            ),],
        }
    );

    let key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(1, 1, 1, 0);
    let ideal_key_stroke_statistics_counter = PrimitiveStatisticsCounter::new(1, 6, 1, 0);
    let spell_statistics_counter = PrimitiveStatisticsCounter::new(1, 4, 1, 0);

    let (sdi, ksdi) = pci.construct_display_info(
        LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()),
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
        &key_stroke_statistics_counter,
        &ideal_key_stroke_statistics_counter,
        &spell_statistics_counter,
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
