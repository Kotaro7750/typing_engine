use super::*;
use crate::{gen_candidate, gen_chunk};

#[test]
fn stroke_key_1() {
    let mut typed_chunk = TypedChunk {
        chunk: gen_chunk!(
            "じょ",
            vec![
                gen_candidate!(["jo"], true, Some(0)),
                gen_candidate!(["zyo"], true, Some(0)),
                gen_candidate!(["jyo"], true, Some(0)),
                gen_candidate!(["zi", "lyo"], true, Some(0)),
                gen_candidate!(["zi", "xyo"], true, Some(0)),
                gen_candidate!(["ji", "lyo"], true, Some(0)),
                gen_candidate!(["ji", "xyo"], true, Some(0)),
            ],
            gen_candidate!(["jo"], true, None)
        ),
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
                    gen_candidate!(["jo"], true, Some(1)),
                    gen_candidate!(["zyo"], false, Some(0)),
                    gen_candidate!(["jyo"], true, Some(1)),
                    gen_candidate!(["zi", "lyo"], false, Some(0)),
                    gen_candidate!(["zi", "xyo"], false, Some(0)),
                    gen_candidate!(["ji", "lyo"], true, Some(1)),
                    gen_candidate!(["ji", "xyo"], true, Some(1)),
                ],
                gen_candidate!(["jo"], true, None)
            ),
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
                    gen_candidate!(["jo"], true, Some(1)),
                    gen_candidate!(["zyo"], false, Some(0)),
                    gen_candidate!(["jyo"], true, Some(1)),
                    gen_candidate!(["zi", "lyo"], false, Some(0)),
                    gen_candidate!(["zi", "xyo"], false, Some(0)),
                    gen_candidate!(["ji", "lyo"], true, Some(1)),
                    gen_candidate!(["ji", "xyo"], true, Some(1)),
                ],
                gen_candidate!(["jo"], true, None)
            ),
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
            chunk: gen_chunk!(
                "じょ",
                vec![
                    gen_candidate!(["jo"], true, Some(2)),
                    gen_candidate!(["zyo"], false, Some(0)),
                    gen_candidate!(["jyo"], false, Some(1)),
                    gen_candidate!(["zi", "lyo"], false, Some(0)),
                    gen_candidate!(["zi", "xyo"], false, Some(0)),
                    gen_candidate!(["ji", "lyo"], false, Some(1)),
                    gen_candidate!(["ji", "xyo"], false, Some(1)),
                ],
                gen_candidate!(["jo"], true, None)
            ),
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
                gen_candidate!(["n"], true, Some(0), ['j']),
                gen_candidate!(["nn"], true, Some(0)),
                gen_candidate!(["xn"], true, Some(0)),
            ],
            gen_candidate!(["n"], true, None, ['j'])
        ),
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
                vec![
                    gen_candidate!(["n"], true, Some(1), ['j']),
                    gen_candidate!(["nn"], true, Some(1)),
                    gen_candidate!(["xn"], false, Some(0)),
                ],
                gen_candidate!(["n"], true, None, ['j'])
            ),
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
                vec![
                    gen_candidate!(["n"], true, Some(1), ['j']),
                    gen_candidate!(["nn"], true, Some(1)),
                    gen_candidate!(["xn"], false, Some(0)),
                ],
                gen_candidate!(["n"], true, None, ['j'])
            ),
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
                vec![
                    gen_candidate!(["n"], false, Some(1), ['j']),
                    gen_candidate!(["nn"], true, Some(2)),
                    gen_candidate!(["xn"], false, Some(0)),
                ],
                gen_candidate!(["n"], true, None, ['j'])
            ),
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
                gen_candidate!(["n"], true, Some(0), ['j']),
                gen_candidate!(["nn"], true, Some(0)),
                gen_candidate!(["xn"], true, Some(0)),
            ],
            gen_candidate!(["n"], true, None, ['j'])
        ),
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
                vec![
                    gen_candidate!(["n"], true, Some(1), ['j']),
                    gen_candidate!(["nn"], true, Some(1)),
                    gen_candidate!(["xn"], false, Some(0)),
                ],
                gen_candidate!(["n"], true, None, ['j'])
            ),
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
                vec![
                    gen_candidate!(["n"], true, Some(1), ['j']),
                    gen_candidate!(["nn"], true, Some(1)),
                    gen_candidate!(["xn"], false, Some(0)),
                ],
                gen_candidate!(["n"], true, None, ['j'])
            ),
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
                vec![
                    gen_candidate!(["n"], true, Some(1), ['j']),
                    gen_candidate!(["nn"], false, Some(1)),
                    gen_candidate!(["xn"], false, Some(0)),
                ],
                gen_candidate!(["n"], true, None, ['j'])
            ),
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
