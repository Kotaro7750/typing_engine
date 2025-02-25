use super::*;

use crate::{
    gen_candidate, gen_candidate_key_stroke, gen_chunk_unprocessed, gen_view_position,
    gen_vocabulary_entry, gen_vocabulary_info,
};

#[test]
fn construct_query_1() {
    let vocabularies = vec![gen_vocabulary_entry!("イオン", [("い"), ("お"), ("ん")])];

    let qr = QueryRequest::new(
        vocabularies
            .iter()
            .map(|ve| ve)
            .collect::<Vec<&VocabularyEntry>>()
            .as_slice(),
        VocabularyQuantifier::KeyStroke(NonZeroUsize::new(5).unwrap()),
        VocabularySeparator::WhiteSpace,
        VocabularyOrder::InOrder,
    );

    let query = qr.construct_query();

    assert_eq!(
        query,
        Query::new(
            vec![
                gen_vocabulary_info!(
                    "イオン",
                    "いおん",
                    vec![
                        gen_view_position!(0),
                        gen_view_position!(1),
                        gen_view_position!(2)
                    ],
                    3
                ),
                gen_vocabulary_info!(" ", " ", vec![gen_view_position!(0)], 1)
            ],
            vec![
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("yi"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "お",
                    vec![gen_candidate!(gen_candidate_key_stroke!("o"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("o"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("nn"), true, None)
                ),
                gen_chunk_unprocessed!(
                    " ",
                    vec![gen_candidate!(gen_candidate_key_stroke!(" "), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!(" "), true, None)
                ),
            ]
        )
    );
}

#[test]
fn construct_query_2() {
    let vocabularies = vec![gen_vocabulary_entry!("イオン", [("い"), ("お"), ("ん")])];

    let qr = QueryRequest::new(
        vocabularies
            .iter()
            .map(|ve| ve)
            .collect::<Vec<&VocabularyEntry>>()
            .as_slice(),
        VocabularyQuantifier::KeyStroke(NonZeroUsize::new(5).unwrap()),
        VocabularySeparator::None,
        VocabularyOrder::InOrder,
    );

    let query = qr.construct_query();

    assert_eq!(
        query,
        Query::new(
            vec![
                gen_vocabulary_info!(
                    "イオン",
                    "いおん",
                    vec![
                        gen_view_position!(0),
                        gen_view_position!(1),
                        gen_view_position!(2)
                    ],
                    3
                ),
                gen_vocabulary_info!(
                    "イオン",
                    "いおん",
                    vec![
                        gen_view_position!(0),
                        gen_view_position!(1),
                        gen_view_position!(2)
                    ],
                    1
                )
            ],
            vec![
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("yi"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "お",
                    vec![gen_candidate!(gen_candidate_key_stroke!("o"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("o"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("nn"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("y"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
            ]
        )
    );
}

#[test]
fn construct_query_3() {
    let vocabularies = vec![
        gen_vocabulary_entry!("イオン", [("い"), ("お"), ("ん")]),
        gen_vocabulary_entry!("買っ", [("か"), ("っ")]),
        gen_vocabulary_entry!("た", [("た")]),
    ];

    let qr = QueryRequest::new(
        vocabularies
            .iter()
            .map(|ve| ve)
            .collect::<Vec<&VocabularyEntry>>()
            .as_slice(),
        VocabularyQuantifier::KeyStroke(NonZeroUsize::new(10).unwrap()),
        VocabularySeparator::None,
        VocabularyOrder::InOrder,
    );

    let query = qr.construct_query();

    assert_eq!(
        query,
        Query::new(
            vec![
                gen_vocabulary_info!(
                    "イオン",
                    "いおん",
                    vec![
                        gen_view_position!(0),
                        gen_view_position!(1),
                        gen_view_position!(2)
                    ],
                    3
                ),
                gen_vocabulary_info!(
                    "買っ",
                    "かっ",
                    vec![gen_view_position!(0), gen_view_position!(1)],
                    2
                ),
                gen_vocabulary_info!("た", "た", vec![gen_view_position!(0)], 1),
                gen_vocabulary_info!(
                    "イオン",
                    "いおん",
                    vec![
                        gen_view_position!(0),
                        gen_view_position!(1),
                        gen_view_position!(2)
                    ],
                    2
                ),
            ],
            vec![
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("yi"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "お",
                    vec![gen_candidate!(gen_candidate_key_stroke!("o"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("o"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, None, ['k', 'c']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), true, None, ['k', 'c'])
                ),
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("ka"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("ca"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("ka"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("t"), true, None, 't'),
                        gen_candidate!(gen_candidate_key_stroke!("ltu"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("xtu"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("ltsu"), true, None),
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("t"), true, None, 't')
                ),
                gen_chunk_unprocessed!(
                    "た",
                    vec![gen_candidate!(gen_candidate_key_stroke!("ta"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("ta"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("yi"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "お",
                    vec![gen_candidate!(gen_candidate_key_stroke!("o"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("o"), true, None)
                ),
            ]
        )
    );
}

#[test]
fn construct_query_4() {
    let vocabularies = vec![
        gen_vocabulary_entry!("1", [("1")]),
        gen_vocabulary_entry!("2", [("2")]),
    ];

    let qr = QueryRequest::new(
        vocabularies
            .iter()
            .map(|ve| ve)
            .collect::<Vec<&VocabularyEntry>>()
            .as_slice(),
        VocabularyQuantifier::KeyStroke(NonZeroUsize::new(3).unwrap()),
        VocabularySeparator::WhiteSpace,
        VocabularyOrder::Arbitrary(Box::new(|prev_vocabulary_index, vocabulary_entries| {
            if prev_vocabulary_index.is_none() {
                vocabulary_entries.len() - 1
            } else if prev_vocabulary_index.is_some()
                && *prev_vocabulary_index.as_ref().unwrap() == 0
            {
                vocabulary_entries.len() - 1
            } else {
                prev_vocabulary_index.as_ref().unwrap() - 1
            }
        })),
    );

    let query = qr.construct_query();

    assert_eq!(
        query,
        Query::new(
            vec![
                gen_vocabulary_info!("2", "2", vec![gen_view_position!(0)], 1),
                gen_vocabulary_info!(" ", " ", vec![gen_view_position!(0)], 1),
                gen_vocabulary_info!("1", "1", vec![gen_view_position!(0)], 1),
            ],
            vec![
                gen_chunk_unprocessed!(
                    "2",
                    vec![gen_candidate!(gen_candidate_key_stroke!("2"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("2"), true, None)
                ),
                gen_chunk_unprocessed!(
                    " ",
                    vec![gen_candidate!(gen_candidate_key_stroke!(" "), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!(" "), true, None)
                ),
                gen_chunk_unprocessed!(
                    "1",
                    vec![gen_candidate!(gen_candidate_key_stroke!("1"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("1"), true, None)
                ),
            ]
        )
    );
}

#[test]
fn construct_query_5() {
    let vocabularies = vec![gen_vocabulary_entry!("イオン", [("い"), ("お"), ("ん")])];

    let qr = QueryRequest::new(
        vocabularies
            .iter()
            .map(|ve| ve)
            .collect::<Vec<&VocabularyEntry>>()
            .as_slice(),
        VocabularyQuantifier::Vocabulary(NonZeroUsize::new(2).unwrap()),
        VocabularySeparator::WhiteSpace,
        VocabularyOrder::InOrder,
    );

    let query = qr.construct_query();

    assert_eq!(
        query,
        Query::new(
            vec![
                gen_vocabulary_info!(
                    "イオン",
                    "いおん",
                    vec![
                        gen_view_position!(0),
                        gen_view_position!(1),
                        gen_view_position!(2)
                    ],
                    3
                ),
                gen_vocabulary_info!(" ", " ", vec![gen_view_position!(0)], 1)
            ],
            vec![
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("yi"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "お",
                    vec![gen_candidate!(gen_candidate_key_stroke!("o"), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!("o"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("nn"), true, None)
                ),
                gen_chunk_unprocessed!(
                    " ",
                    vec![gen_candidate!(gen_candidate_key_stroke!(" "), true, None)],
                    gen_candidate!(gen_candidate_key_stroke!(" "), true, None)
                ),
            ]
        )
    );
}

#[test]
fn construct_query_6() {
    let vocabularies = vec![gen_vocabulary_entry!("印字", [("いん"), ("じ")])];

    let qr = QueryRequest::new(
        vocabularies
            .iter()
            .map(|ve| ve)
            .collect::<Vec<&VocabularyEntry>>()
            .as_slice(),
        VocabularyQuantifier::KeyStroke(NonZeroUsize::new(3).unwrap()),
        VocabularySeparator::WhiteSpace,
        VocabularyOrder::InOrder,
    );

    let query = qr.construct_query();

    assert_eq!(
        query,
        Query::new(
            vec![gen_vocabulary_info!(
                "印字",
                "いんじ",
                vec![
                    gen_view_position!(0),
                    gen_view_position!(0),
                    gen_view_position!(1)
                ],
                3
            ),],
            vec![
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("yi"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, None, ['z', 'j']),
                        gen_candidate!(gen_candidate_key_stroke!("nn"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("xn"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), true, None, ['z', 'j'])
                ),
                gen_chunk_unprocessed!(
                    "じ",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("z"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("j"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("z"), true, None)
                ),
            ]
        )
    );
}

#[test]
fn construct_query_7() {
    let vocabularies = vec![gen_vocabulary_entry!("印字", [("いん"), ("じ")])];

    let qr = QueryRequest::new(
        vocabularies
            .iter()
            .map(|ve| ve)
            .collect::<Vec<&VocabularyEntry>>()
            .as_slice(),
        VocabularyQuantifier::KeyStroke(NonZeroUsize::new(2).unwrap()),
        VocabularySeparator::WhiteSpace,
        VocabularyOrder::InOrder,
    );

    let query = qr.construct_query();

    assert_eq!(
        query,
        Query::new(
            vec![gen_vocabulary_info!(
                "印字",
                "いんじ",
                vec![
                    gen_view_position!(0),
                    gen_view_position!(0),
                    gen_view_position!(1)
                ],
                2
            ),],
            vec![
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("i"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("yi"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("i"), true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(gen_candidate_key_stroke!("n"), true, None),
                        gen_candidate!(gen_candidate_key_stroke!("x"), true, None)
                    ],
                    gen_candidate!(gen_candidate_key_stroke!("n"), true, None)
                ),
            ]
        )
    );
}
