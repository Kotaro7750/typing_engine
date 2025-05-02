use super::*;

use std::time::Duration;

use crate::statistics::statistical_event::{
    ChunkAddedContext, ChunkConfirmedContext, IdealKeyStrokeDeemedFinishedContext,
    SpellFinishedContext,
};
use crate::typing_primitive_types::chunk::inflight::ChunkSpellCursorPosition;
use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;
use crate::typing_primitive_types::chunk::ChunkSpell;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
use crate::{gen_candidate, gen_candidate_key_stroke, gen_chunk_inflight, gen_chunk_unprocessed};

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
            KeyStrokeCorrectContext::new(
                'k'.try_into().unwrap(),
                Duration::from_secs(2),
                vec!['j'.try_into().unwrap()]
            )
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
                Duration::from_secs(1),
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
                Duration::from_secs(1),
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
fn correct_stroke_to_processed_chunk_info_with_delayed_confirmable_inflight_chunk_confirms_delayed_confirmed_candidate(
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
                Duration::from_secs(2),
                vec!['j'.try_into().unwrap()]
            ))
        ]
    );
}

#[test]
fn correct_stroke_to_processed_chunk_info_with_delayed_confirmable_inflight_chunk_confirms_not_delayed_confirmed_candidate(
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
                Duration::from_secs(2),
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
fn correct_stroke_to_processed_chunk_info_with_delayed_confirmable_inflight_chunk_confirms_delayed_confirmed_candidate_and_next_chunk(
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
                Duration::from_secs(1),
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
fn snapshot_processed_chunk_info_with_unprocessed_chunk() {
    let (pci, _) = ProcessedChunkInfo::new(vec![gen_chunk_unprocessed!(
        "きょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("kyo")),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!("kyo"))
    )]);

    let events = pci.snapshot();

    assert_eq!(
        events,
        vec![
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_unstarted(
                &'k'.try_into().unwrap()
            )),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_unstarted(
                &'y'.try_into().unwrap()
            )),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_unstarted(
                &'o'.try_into().unwrap()
            )),
        ]
    );
}

#[test]
fn snapshot_processed_chunk_info_with_inflight_chunk_with_double_splitted() {
    let pci = ProcessedChunkInfo {
        unprocessed_chunks: vec![].into(),
        inflight_chunk: Some(gen_chunk_inflight!(
            "きょ",
            vec![gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),],
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'i'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(3, 0), 'x'.try_into().unwrap(), true)
            ],
            3,
            []
        )),
        confirmed_chunks: vec![],
        elapsed_time_of_last_correct_key_stroke: Duration::from_secs(3),
    };

    let events = pci.snapshot();

    assert_eq!(
        events,
        vec![
            StatisticalEvent::IdealKeyStrokeDeemedFinished(
                IdealKeyStrokeDeemedFinishedContext::new(0)
            ),
            StatisticalEvent::IdealKeyStrokeDeemedFinished(
                IdealKeyStrokeDeemedFinishedContext::new(0)
            ),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_started(
                &'y'.try_into().unwrap(),
                vec![]
            )),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_unstarted(
                &'o'.try_into().unwrap()
            )),
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("ょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleSecond,
                vec![]
            )),
        ]
    );
}

#[test]
fn snapshot_processed_chunk_info_with_inflight_chunk_without_wrong_key_stroke() {
    let pci = ProcessedChunkInfo {
        unprocessed_chunks: vec![].into(),
        inflight_chunk: Some(gen_chunk_inflight!(
            "きょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"])),
            ],
            vec![],
            gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'k'.try_into().unwrap(),
                true
            )],
            1,
            []
        )),
        confirmed_chunks: vec![],
        elapsed_time_of_last_correct_key_stroke: Duration::from_secs(1),
    };

    let events = pci.snapshot();

    assert_eq!(
        events,
        vec![
            StatisticalEvent::IdealKeyStrokeDeemedFinished(
                IdealKeyStrokeDeemedFinishedContext::new(0)
            ),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_started(
                &'y'.try_into().unwrap(),
                vec![]
            )),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_unstarted(
                &'o'.try_into().unwrap()
            )),
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleCombined,
                vec![]
            )),
        ]
    );
}

#[test]
fn snapshot_processed_chunk_info_with_inflight_chunk_with_wrong_key_stroke() {
    let pci = ProcessedChunkInfo {
        unprocessed_chunks: vec![].into(),
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
                ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(2, 0), 'k'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(3, 0), 't'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(4, 0), 'u'.try_into().unwrap(), false)
            ],
            1,
            []
        )),
        confirmed_chunks: vec![],
        elapsed_time_of_last_correct_key_stroke: Duration::from_secs(4),
    };

    let events = pci.snapshot();

    assert_eq!(
        events,
        vec![
            StatisticalEvent::IdealKeyStrokeDeemedFinished(
                IdealKeyStrokeDeemedFinishedContext::new(1)
            ),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_started(
                &'y'.try_into().unwrap(),
                vec!['t'.try_into().unwrap(), 'u'.try_into().unwrap()]
            )),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_unstarted(
                &'o'.try_into().unwrap()
            )),
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("きょ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::DoubleCombined,
                vec![
                    'h'.try_into().unwrap(),
                    't'.try_into().unwrap(),
                    'u'.try_into().unwrap()
                ]
            )),
        ]
    );
}

#[test]
fn snapshot_processed_chunk_info_with_inflight_chunk_with_delayed_confirmable_candidate() {
    let pci = ProcessedChunkInfo {
        unprocessed_chunks: vec![gen_chunk_unprocessed!(
            "じ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("zi")),
                gen_candidate!(gen_candidate_key_stroke!("ji")),
            ],
            gen_candidate!(gen_candidate_key_stroke!("zi"))
        )]
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
        elapsed_time_of_last_correct_key_stroke: Duration::from_secs(2),
    };

    let events = pci.snapshot();

    assert_eq!(
        events,
        vec![
            StatisticalEvent::IdealKeyStrokeDeemedFinished(
                IdealKeyStrokeDeemedFinishedContext::new(0)
            ),
            StatisticalEvent::SpellDeemedFinished(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                0
            )),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_started(
                &'z'.try_into().unwrap(),
                vec!['m'.try_into().unwrap()]
            )),
            StatisticalEvent::InflightSpellSnapshotted(InflightSpellSnapshottedContext::new(
                ChunkSpell::new("じ".to_string().try_into().unwrap()),
                ChunkSpellCursorPosition::Single,
                vec!['m'.try_into().unwrap()]
            )),
            StatisticalEvent::KeyStrokeSnapshotted(KeyStrokeSnapshottedContext::new_unstarted(
                &'i'.try_into().unwrap(),
            )),
        ]
    );
}
