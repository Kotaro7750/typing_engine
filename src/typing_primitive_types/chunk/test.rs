use super::*;

use crate::{gen_candidate, gen_chunk};

#[test]
fn strict_key_stroke_count_1() {
    let mut chunk = gen_chunk!(
        "じょ",
        vec![
            gen_candidate!(["jo"], true, None),
            gen_candidate!(["zyo"], true, None),
            gen_candidate!(["jyo"], true, None),
            gen_candidate!(["zi", "lyo"], true, None),
            gen_candidate!(["zi", "xyo"], true, None),
            gen_candidate!(["ji", "lyo"], true, None),
            gen_candidate!(["ji", "xyo"], true, None),
        ],
        ChunkState::Unprocessed,
        gen_candidate!(["jo"], true, None)
    );

    chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

    assert_eq!(
        chunk,
        gen_chunk!(
            "じょ",
            vec![
                gen_candidate!(["j"], true, None),
                gen_candidate!(["z"], true, None),
            ],
            ChunkState::Unprocessed,
            gen_candidate!(["j"], true, None)
        )
    )
}

#[test]
fn strict_key_stroke_count_2() {
    let mut chunk = gen_chunk!(
        "ん",
        vec![
            gen_candidate!(["n"], true, None, ['j', 'z']),
            gen_candidate!(["nn"], true, None),
            gen_candidate!(["xn"], true, None),
        ],
        ChunkState::Unprocessed,
        gen_candidate!(["n"], true, None, ['j', 'z'])
    );

    chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

    assert_eq!(
        chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None),
                gen_candidate!(["x"], true, None)
            ],
            ChunkState::Unprocessed,
            gen_candidate!(["n"], true, None)
        )
    )
}

#[test]
fn is_element_end_at_key_stroke_index_1() {
    let c = gen_candidate!(["ki", "xyo"], true, None);

    assert!(!c.is_element_end_at_key_stroke_index(0));
    assert!(c.is_element_end_at_key_stroke_index(1));
    assert!(!c.is_element_end_at_key_stroke_index(2));
    assert!(!c.is_element_end_at_key_stroke_index(3));
    assert!(c.is_element_end_at_key_stroke_index(4));
}

#[test]
fn stroke_key_1() {
    let mut typed_chunk = gen_chunk!(
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
        ChunkState::Inflight,
        gen_candidate!(["jo"], true, None),
        []
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Correct);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
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
            ChunkState::Inflight,
            gen_candidate!(["jo"], true, None),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'j'.try_into().unwrap(),
                true
            )]
        )
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Wrong);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
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
            ChunkState::Inflight,
            gen_candidate!(["jo"], true, None),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false)
            ]
        )
    );

    let stroke_result = typed_chunk.stroke_key('o'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Correct);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
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
            ChunkState::Inflight,
            gen_candidate!(["jo"], true, None),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'o'.try_into().unwrap(), true)
            ]
        )
    );
}

#[test]
fn stroke_key_2() {
    let mut typed_chunk = gen_chunk!(
        "ん",
        vec![
            gen_candidate!(["n"], true, Some(0), ['j']),
            gen_candidate!(["nn"], true, Some(0)),
            gen_candidate!(["xn"], true, Some(0)),
        ],
        ChunkState::Inflight,
        gen_candidate!(["n"], true, None, ['j']),
        []
    );

    let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Correct);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, Some(1), ['j']),
                gen_candidate!(["nn"], true, Some(1)),
                gen_candidate!(["xn"], false, Some(0)),
            ],
            ChunkState::Inflight,
            gen_candidate!(["n"], true, None, ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )]
        ),
    );

    assert!(!typed_chunk.is_confirmed());
    assert!(typed_chunk.is_delayed_confirmable());

    let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Wrong);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, Some(1), ['j']),
                gen_candidate!(["nn"], true, Some(1)),
                gen_candidate!(["xn"], false, Some(0)),
            ],
            ChunkState::Inflight,
            gen_candidate!(["n"], true, None, ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )]
        )
    );

    let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Correct);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], false, Some(1), ['j']),
                gen_candidate!(["nn"], true, Some(2)),
                gen_candidate!(["xn"], false, Some(0)),
            ],
            ChunkState::Inflight,
            gen_candidate!(["n"], true, None, ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )]
        )
    );

    assert!(typed_chunk.is_confirmed());
}

#[test]
fn stroke_key_3() {
    let mut typed_chunk = gen_chunk!(
        "ん",
        vec![
            gen_candidate!(["n"], true, Some(0), ['j']),
            gen_candidate!(["nn"], true, Some(0)),
            gen_candidate!(["xn"], true, Some(0)),
        ],
        ChunkState::Inflight,
        gen_candidate!(["n"], true, None, ['j']),
        []
    );

    let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Correct);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, Some(1), ['j']),
                gen_candidate!(["nn"], true, Some(1)),
                gen_candidate!(["xn"], false, Some(0)),
            ],
            ChunkState::Inflight,
            gen_candidate!(["n"], true, None, ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )]
        )
    );

    assert!(!typed_chunk.is_confirmed());
    assert!(typed_chunk.is_delayed_confirmable());

    let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Wrong);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, Some(1), ['j']),
                gen_candidate!(["nn"], true, Some(1)),
                gen_candidate!(["xn"], false, Some(0)),
            ],
            ChunkState::Inflight,
            gen_candidate!(["n"], true, None, ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )]
        )
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(3, 0));
    assert_eq!(stroke_result, KeyStrokeResult::Correct);

    assert_eq!(
        typed_chunk,
        gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], true, Some(1), ['j']),
                gen_candidate!(["nn"], false, Some(1)),
                gen_candidate!(["xn"], false, Some(0)),
            ],
            ChunkState::Inflight,
            gen_candidate!(["n"], true, None, ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )]
        )
    );

    assert!(typed_chunk.is_confirmed());
}

#[test]
fn construct_spell_end_vector_1() {
    let cc = gen_chunk!(
        "きょ",
        vec![
            gen_candidate!(["kyo"], false, Some(1)),
            gen_candidate!(["ki", "lyo"], false, Some(2)),
            gen_candidate!(["ki", "xyo"], true, Some(5))
        ],
        ChunkState::Confirmed,
        gen_candidate!(["kyo"], true, None),
        [
            ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(2, 0), 'i'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(4, 0), 'x'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(5, 0), 'y'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(6, 0), 'o'.try_into().unwrap(), true)
        ]
    );

    let spell_end_vector = cc.construct_spell_end_vector();

    assert_eq!(
        spell_end_vector,
        vec![None, Some(1), None, None, None, Some(1)]
    );
}

#[test]
fn construct_spell_end_vector_2() {
    let cc = gen_chunk!(
        "きょ",
        vec![
            gen_candidate!(["kyo"], true, Some(3)),
            gen_candidate!(["ki", "lyo"], false, Some(1)),
            gen_candidate!(["ki", "xyo"], false, Some(1))
        ],
        ChunkState::Confirmed,
        gen_candidate!(["kyo"], true, None),
        [
            ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(3, 0), 'y'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(4, 0), 'o'.try_into().unwrap(), true)
        ]
    );

    let spell_end_vector = cc.construct_spell_end_vector();

    assert_eq!(spell_end_vector, vec![None, None, None, Some(2)]);
}
