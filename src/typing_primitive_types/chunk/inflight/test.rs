use super::*;
use crate::{gen_candidate, gen_candidate_key_stroke, gen_chunk_inflight};

#[test]
fn wrong_stroke_to_inflight_chunk() {
    let mut typed_chunk = generate_test_inflight_chunk();

    let stroke_result = typed_chunk.stroke_key('h'.try_into().unwrap(), Duration::new(1, 0));

    assertion_stroke_wrong(&stroke_result);
    assert_eq!(
        typed_chunk,
        generate_test_inflight_chunk_with_a_wrong_stroke()
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_without_wrong_stroke() {
    let mut typed_chunk = generate_test_inflight_chunk();

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(2, 0));

    assertion_stroke_correct(&stroke_result, vec![]);
    assertion_not_finish_spell(&stroke_result);
    assertion_not_chunk_confirm(&stroke_result, &typed_chunk);
    assert_eq!(
        typed_chunk,
        generate_test_inflight_chunk_with_a_correct_stroke()
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_with_wrong_stroke() {
    let mut typed_chunk = generate_test_inflight_chunk_with_a_wrong_stroke();

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(2, 0));

    assertion_stroke_correct(
        &stroke_result,
        vec![ActualKeyStroke::new(
            Duration::new(1, 0),
            'h'.try_into().unwrap(),
            false,
        )],
    );
    assertion_not_finish_spell(&stroke_result);
    assertion_not_chunk_confirm(&stroke_result, &typed_chunk);
    assert_eq!(
        typed_chunk,
        generate_test_inflight_chunk_with_wrong_stroke_and_correct_stroke()
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_finishes_spell() {
    let mut typed_chunk = generate_test_inflight_chunk_with_wrong_stroke_and_correct_stroke();

    let stroke_result = typed_chunk.stroke_key('i'.try_into().unwrap(), Duration::new(3, 0));

    assertion_stroke_correct(&stroke_result, vec![]);
    assertion_not_chunk_confirm(&stroke_result, &typed_chunk);
    assertion_finish_spell(&stroke_result, "じ", 1);
    assert_eq!(
        typed_chunk,
        gen_chunk_inflight!(
            "じょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),
            ],
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["jo"])),
                gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(3, 0), 'i'.try_into().unwrap(), true)
            ],
            2,
            []
        )
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_confirms_chunk() {
    let mut typed_chunk = generate_test_inflight_chunk_with_wrong_stroke_and_correct_stroke();

    let stroke_result = typed_chunk.stroke_key('o'.try_into().unwrap(), Duration::new(3, 0));

    assertion_stroke_correct(&stroke_result, vec![]);
    assertion_chunk_confirm(&stroke_result, &typed_chunk, "じょ", 1, vec![].as_slice());
    assert_eq!(
        typed_chunk,
        generate_test_inflight_chunk_with_wrong_stroke_and_correct_stroke_and_confirmed()
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_finishes_spell_and_confirms_split_chunk() {
    let mut typed_chunk = gen_chunk_inflight!(
        "じょ",
        vec![gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),],
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!(["jo"])),
        [
            ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(3, 0), 'i'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(4, 0), 'x'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(5, 0), 'y'.try_into().unwrap(), true)
        ],
        4,
        []
    );

    let stroke_result = typed_chunk.stroke_key('o'.try_into().unwrap(), Duration::new(6, 0));

    assertion_stroke_correct(&stroke_result, vec![]);
    assertion_chunk_confirm(&stroke_result, &typed_chunk, "ょ", 0, vec![].as_slice());
    assert_eq!(
        typed_chunk,
        gen_chunk_inflight!(
            "じょ",
            vec![gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),],
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["jo"])),
                gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
                gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(3, 0), 'i'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(4, 0), 'x'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(5, 0), 'y'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(6, 0), 'o'.try_into().unwrap(), true)
            ],
            5,
            []
        )
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_that_becomes_delayed_confirmable() {
    let mut typed_chunk = gen_chunk_inflight!(
        "ん",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            gen_candidate!(gen_candidate_key_stroke!("nn")),
            gen_candidate!(gen_candidate_key_stroke!("xn")),
        ],
        vec![],
        gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
        [],
        0,
        []
    );

    let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(1, 0));

    assertion_stroke_correct(&stroke_result, vec![]);
    assertion_not_finish_spell(&stroke_result);
    assertion_not_chunk_confirm(&stroke_result, &typed_chunk);
    assert!(typed_chunk.delayed_confirmable_candidate_index().is_some());
    assert_eq!(
        typed_chunk.delayed_confirmable_candidate_index().unwrap(),
        0
    );
    assert_eq!(
        typed_chunk,
        gen_chunk_inflight!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
            ],
            vec![gen_candidate!(gen_candidate_key_stroke!("xn")),],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )],
            1,
            []
        ),
    );
}

#[test]
fn wrong_stroke_to_inflight_chunk_that_is_delayed_confirmable() {
    let mut typed_chunk = gen_chunk_inflight!(
        "ん",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            gen_candidate!(gen_candidate_key_stroke!("nn")),
        ],
        vec![gen_candidate!(gen_candidate_key_stroke!("xn")),],
        gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
        [ActualKeyStroke::new(
            Duration::new(1, 0),
            'n'.try_into().unwrap(),
            true
        )],
        1,
        []
    );

    let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));

    assertion_stroke_wrong(&stroke_result);
    assert_eq!(
        typed_chunk.pending_key_strokes(),
        vec![ActualKeyStroke::new(
            Duration::new(2, 0),
            'm'.try_into().unwrap(),
            false
        )]
    );
    assert_eq!(
        typed_chunk,
        gen_chunk_inflight!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
            ],
            vec![gen_candidate!(gen_candidate_key_stroke!("xn")),],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
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
        )
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_confirms_delayed_confirm_candidate() {
    let mut typed_chunk = gen_chunk_inflight!(
        "ん",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            gen_candidate!(gen_candidate_key_stroke!("nn")),
        ],
        vec![gen_candidate!(gen_candidate_key_stroke!("xn")),],
        gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
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
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(3, 0));

    assertion_stroke_correct_delayed_confirm(&stroke_result);
    assertion_chunk_confirm(
        &stroke_result,
        &typed_chunk,
        "ん",
        0,
        vec![
            ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), true),
        ]
        .as_slice(),
    );
    assert!(stroke_result.wrong_key_strokes().is_none());

    assert_eq!(
        typed_chunk,
        gen_chunk_inflight!(
            "ん",
            vec![gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),],
            vec![
                gen_candidate!(gen_candidate_key_stroke!("xn")),
                gen_candidate!(gen_candidate_key_stroke!("nn")),
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            [ActualKeyStroke::new(
                Duration::new(1, 0),
                'n'.try_into().unwrap(),
                true
            )],
            1,
            []
        )
    );
}

#[test]
fn correct_stroke_to_inflight_chunk_confirms_not_delayed_confirm_candidate() {
    let mut typed_chunk = gen_chunk_inflight!(
        "ん",
        vec![
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            gen_candidate!(gen_candidate_key_stroke!("nn")),
        ],
        vec![gen_candidate!(gen_candidate_key_stroke!("xn")),],
        gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
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
    );

    let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));

    assertion_stroke_correct(
        &stroke_result,
        vec![ActualKeyStroke::new(
            Duration::new(2, 0),
            'm'.try_into().unwrap(),
            false,
        )],
    );
    assertion_chunk_confirm(&stroke_result, &typed_chunk, "ん", 1, vec![].as_slice());
    assert_eq!(
        typed_chunk,
        gen_chunk_inflight!(
            "ん",
            vec![gen_candidate!(gen_candidate_key_stroke!("nn")),],
            vec![
                gen_candidate!(gen_candidate_key_stroke!("xn")),
                gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), ['j']),
            [
                ActualKeyStroke::new(Duration::new(1, 0), 'n'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'n'.try_into().unwrap(), true)
            ],
            2,
            []
        )
    );
}

fn generate_test_inflight_chunk() -> ChunkInflight {
    gen_chunk_inflight!(
        "じょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),
        ],
        vec![],
        gen_candidate!(gen_candidate_key_stroke!(["jo"])),
        [],
        0,
        []
    )
}

fn generate_test_inflight_chunk_with_a_wrong_stroke() -> ChunkInflight {
    gen_chunk_inflight!(
        "じょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),
        ],
        vec![],
        gen_candidate!(gen_candidate_key_stroke!(["jo"])),
        [ActualKeyStroke::new(
            Duration::new(1, 0),
            'h'.try_into().unwrap(),
            false
        )],
        0,
        []
    )
}

fn generate_test_inflight_chunk_with_a_correct_stroke() -> ChunkInflight {
    gen_chunk_inflight!(
        "じょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),
        ],
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!(["jo"])),
        [ActualKeyStroke::new(
            Duration::new(2, 0),
            'j'.try_into().unwrap(),
            true
        )],
        1,
        []
    )
}

fn generate_test_inflight_chunk_with_wrong_stroke_and_correct_stroke() -> ChunkInflight {
    gen_chunk_inflight!(
        "じょ",
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["jo"])),
            gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),
        ],
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!(["jo"])),
        [
            ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), true)
        ],
        1,
        []
    )
}

fn generate_test_inflight_chunk_with_wrong_stroke_and_correct_stroke_and_confirmed() -> ChunkInflight
{
    gen_chunk_inflight!(
        "じょ",
        vec![gen_candidate!(gen_candidate_key_stroke!(["jo"])),],
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["zyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["jyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!(["jo"])),
        [
            ActualKeyStroke::new(Duration::new(1, 0), 'h'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(3, 0), 'o'.try_into().unwrap(), true)
        ],
        2,
        []
    )
}

fn assertion_stroke_wrong(stroke_result: &KeyStrokeResult) {
    assert!(!stroke_result.is_correct());
    assert!(stroke_result.correct_context().is_none());
    assert!(stroke_result.wrong_key_strokes().is_none());
}

fn assertion_stroke_correct(
    stroke_result: &KeyStrokeResult,
    wrong_key_strokes: Vec<ActualKeyStroke>,
) {
    assert!(stroke_result.is_correct());
    assert!(stroke_result.correct_context().is_some());
    assert!(stroke_result.wrong_key_strokes().is_some());
    assert_eq!(
        stroke_result.wrong_key_strokes().unwrap(),
        wrong_key_strokes.as_slice()
    );
}

fn assertion_stroke_correct_delayed_confirm(stroke_result: &KeyStrokeResult) {
    assert!(stroke_result.is_correct());
    assert!(stroke_result.correct_context().is_some());
    assert!(stroke_result.wrong_key_strokes().is_none());
}

fn assertion_not_finish_spell(stroke_result: &KeyStrokeResult) {
    assert!(stroke_result
        .correct_context()
        .unwrap()
        .spell_finished_context()
        .is_none());
}

fn assertion_finish_spell(
    stroke_result: &KeyStrokeResult,
    spell: &str,
    wrong_key_strokes_count: usize,
) {
    assert!(stroke_result
        .correct_context()
        .unwrap()
        .spell_finished_context()
        .is_some());
    assert_eq!(
        stroke_result
            .correct_context()
            .unwrap()
            .spell_finished_context()
            .as_ref()
            .unwrap()
            .clone(),
        SpellFinishedContext::new(
            ChunkSpell::new(spell.to_string().try_into().unwrap()),
            wrong_key_strokes_count
        )
    );
}

fn assertion_not_chunk_confirm(stroke_result: &KeyStrokeResult, typed_chunk: &ChunkInflight) {
    assert!(stroke_result
        .correct_context()
        .unwrap()
        .chunk_confirmation()
        .is_none());
    assert!(!typed_chunk.is_confirmed());
}

fn assertion_chunk_confirm(
    stroke_result: &KeyStrokeResult,
    typed_chunk: &ChunkInflight,
    spell: &str,
    wrong_key_strokes_count: usize,
    pending_key_strokes_for_next_chunk: &[ActualKeyStroke],
) {
    assertion_finish_spell(stroke_result, spell, wrong_key_strokes_count);
    assert!(stroke_result
        .correct_context()
        .unwrap()
        .chunk_confirmation()
        .is_some());
    assert_eq!(
        stroke_result
            .correct_context()
            .unwrap()
            .chunk_confirmation()
            .as_ref()
            .unwrap()
            .as_slice(),
        pending_key_strokes_for_next_chunk
    );
    assert!(typed_chunk.is_confirmed());
}
