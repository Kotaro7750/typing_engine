use super::*;
use crate::{gen_candidate, gen_chunk};

#[test]
fn stroke_key_1() {
    let mut typed_chunk = TypedChunk {
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
            ],
            gen_candidate!(["jo"])
        ),
        cursor_positions_of_candidates: vec![0; 7],
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
                    gen_candidate!(["jo"]),
                    gen_candidate!(["jyo"]),
                    gen_candidate!(["ji", "lyo"]),
                    gen_candidate!(["ji", "xyo"]),
                ],
                gen_candidate!(["jo"])
            ),
            cursor_positions_of_candidates: vec![1; 4],
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
                    gen_candidate!(["jo"]),
                    gen_candidate!(["jyo"]),
                    gen_candidate!(["ji", "lyo"]),
                    gen_candidate!(["ji", "xyo"]),
                ],
                gen_candidate!(["jo"])
            ),
            cursor_positions_of_candidates: vec![1; 4],
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
            chunk: gen_chunk!("じょ", vec![gen_candidate!(["jo"])], gen_candidate!(["jo"])),
            cursor_positions_of_candidates: vec![2],
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
                gen_candidate!(["n"], ['j']),
                gen_candidate!(["nn"]),
                gen_candidate!(["xn"]),
            ],
            gen_candidate!(["n"], ['j'])
        ),
        cursor_positions_of_candidates: vec![0; 3],
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
                vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                gen_candidate!(["n"], ['j'])
            ),
            cursor_positions_of_candidates: vec![1, 1],
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
                vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                gen_candidate!(["n"], ['j'])
            ),
            cursor_positions_of_candidates: vec![1, 1],
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
                vec![gen_candidate!(["nn"])],
                gen_candidate!(["n"], ['j'])
            ),
            cursor_positions_of_candidates: vec![2],
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
                gen_candidate!(["n"], ['j']),
                gen_candidate!(["nn"]),
                gen_candidate!(["xn"]),
            ],
            gen_candidate!(["n"], ['j'])
        ),
        cursor_positions_of_candidates: vec![0; 3],
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
                vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                gen_candidate!(["n"], ['j'])
            ),
            cursor_positions_of_candidates: vec![1, 1],
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
                vec![gen_candidate!(["n"], ['j']), gen_candidate!(["nn"])],
                gen_candidate!(["n"], ['j'])
            ),
            cursor_positions_of_candidates: vec![1, 1],
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
                vec![gen_candidate!(["n"], ['j'])],
                gen_candidate!(["n"], ['j'])
            ),
            cursor_positions_of_candidates: vec![1],
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
