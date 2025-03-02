use inflight::KeyStrokeCorrectContext;

use super::*;
use crate::{
    statistics::statistical_event::SpellFinishedContext,
    typing_primitive_types::key_stroke::ActualKeyStroke,
};
use std::time::Duration;

use crate::{gen_candidate, gen_candidate_key_stroke, gen_chunk_inflight};

#[test]
fn is_element_end_at_key_stroke_index_1() {
    let c = gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]));

    assert_eq!(c.is_element_end_at_key_stroke_index(0), Some(false));
    assert_eq!(c.is_element_end_at_key_stroke_index(1), Some(true));
    assert_eq!(c.is_element_end_at_key_stroke_index(2), Some(false));
    assert_eq!(c.is_element_end_at_key_stroke_index(3), Some(false));
    assert_eq!(c.is_element_end_at_key_stroke_index(4), Some(true));
    assert_eq!(c.is_element_end_at_key_stroke_index(5), None);
}

#[test]
fn stroke_key_1() {
    let mut typed_chunk = gen_chunk_inflight!(
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
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(None, None)
    );

    assert_eq!(
        typed_chunk,
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
                Duration::new(1, 0),
                'j'.try_into().unwrap(),
                true
            )],
            1,
            []
        )
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(2, 0));
    assert!(!stroke_result.is_correct());

    assert_eq!(
        typed_chunk,
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
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false)
            ],
            1,
            []
        )
    );

    let stroke_result = typed_chunk.stroke_key('o'.try_into().unwrap(), Duration::new(3, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(
            Some(SpellFinishedContext::new(
                ChunkSpell::new("じょ".to_string().try_into().unwrap()),
                1
            )),
            Some(vec![])
        )
    );

    assert_eq!(
        typed_chunk,
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
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'o'.try_into().unwrap(), true)
            ],
            2,
            []
        )
    );
}

#[test]
fn stroke_key_2() {
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
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(None, None)
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

    assert!(!typed_chunk.is_confirmed());
    assert!(typed_chunk.delayed_confirmable_candidate_index().is_some());

    let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
    assert!(!stroke_result.is_correct());

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

    let stroke_result = typed_chunk.stroke_key('n'.try_into().unwrap(), Duration::new(3, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(
            Some(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                1
            )),
            Some(vec![])
        )
    );

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

    assert!(typed_chunk.is_confirmed());
}

#[test]
fn stroke_key_3() {
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
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(None, None)
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
        )
    );

    assert!(!typed_chunk.is_confirmed());
    assert!(typed_chunk.delayed_confirmable_candidate_index().is_some());

    let stroke_result = typed_chunk.stroke_key('m'.try_into().unwrap(), Duration::new(2, 0));
    assert!(!stroke_result.is_correct());

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

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(3, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(
            Some(SpellFinishedContext::new(
                ChunkSpell::new("ん".to_string().try_into().unwrap()),
                0
            )),
            Some(vec![
                ActualKeyStroke::new(Duration::new(2, 0), 'm'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), true)
            ])
        )
    );

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

    assert!(typed_chunk.is_confirmed());
}

#[test]
fn stroke_key_4() {
    let mut typed_chunk = gen_chunk_inflight!(
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
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(1, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(None, None)
    );

    assert_eq!(
        typed_chunk,
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
                Duration::new(1, 0),
                'j'.try_into().unwrap(),
                true
            )],
            1,
            []
        )
    );

    let stroke_result = typed_chunk.stroke_key('j'.try_into().unwrap(), Duration::new(2, 0));
    assert!(!stroke_result.is_correct());

    assert_eq!(
        typed_chunk,
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
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false)
            ],
            1,
            []
        )
    );

    let stroke_result = typed_chunk.stroke_key('i'.try_into().unwrap(), Duration::new(3, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(
            Some(SpellFinishedContext::new(
                ChunkSpell::new("じ".to_string().try_into().unwrap()),
                1
            )),
            None
        )
    );

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
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'i'.try_into().unwrap(), true)
            ],
            2,
            []
        )
    );

    let stroke_result = typed_chunk.stroke_key('x'.try_into().unwrap(), Duration::new(4, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(None, None)
    );

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
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'i'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(4, 0), 'x'.try_into().unwrap(), true)
            ],
            3,
            []
        )
    );

    let stroke_result = typed_chunk.stroke_key('y'.try_into().unwrap(), Duration::new(5, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(None, None)
    );

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
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
                ActualKeyStroke::new(Duration::new(3, 0), 'i'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(4, 0), 'x'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(5, 0), 'y'.try_into().unwrap(), true)
            ],
            4,
            []
        )
    );

    let stroke_result = typed_chunk.stroke_key('o'.try_into().unwrap(), Duration::new(6, 0));
    assert!(stroke_result.is_correct());
    assert_eq!(
        stroke_result.correct_context().unwrap().clone(),
        KeyStrokeCorrectContext::new(
            Some(SpellFinishedContext::new(
                ChunkSpell::new("ょ".to_string().try_into().unwrap()),
                0
            )),
            Some(vec![])
        )
    );

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
                ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
                ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
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
fn construct_spell_end_vector_1() {
    let cc = gen_chunk_inflight!(
        "きょ",
        vec![gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]))],
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
        ],
        gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
        [
            ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(2, 0), 'i'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(4, 0), 'x'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(5, 0), 'y'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(6, 0), 'o'.try_into().unwrap(), true)
        ],
        5,
        []
    );

    let spell_end_vector = cc.construct_spell_end_vector();

    assert_eq!(
        spell_end_vector,
        vec![None, Some(1), None, None, None, Some(1)]
    );
}

#[test]
fn construct_spell_end_vector_2() {
    let cc = gen_chunk_inflight!(
        "きょ",
        vec![gen_candidate!(gen_candidate_key_stroke!(["kyo"]))],
        vec![
            gen_candidate!(gen_candidate_key_stroke!(["ki", "lyo"])),
            gen_candidate!(gen_candidate_key_stroke!(["ki", "xyo"]))
        ],
        gen_candidate!(gen_candidate_key_stroke!(["kyo"])),
        [
            ActualKeyStroke::new(Duration::new(1, 0), 'k'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(3, 0), 'y'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(4, 0), 'o'.try_into().unwrap(), true)
        ],
        3,
        []
    );

    let spell_end_vector = cc.construct_spell_end_vector();

    assert_eq!(spell_end_vector, vec![None, None, None, Some(2)]);
}

#[test]
fn wrong_positions_1() {
    let typed_chunk = gen_chunk_inflight!(
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
            ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(5, 0), 'i'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(6, 0), 'x'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(7, 0), 'x'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(8, 0), 'y'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(9, 0), 'o'.try_into().unwrap(), true)
        ],
        5,
        []
    );

    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_key_stroke_index(),
        vec![1, 2, 0, 1, 0]
    );

    assert_eq!(typed_chunk.wrong_key_stroke_positions(0), vec![0, 1, 3]);

    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_element_index(ChunkElementIndex::OnlyFirst),
        0
    );
    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_element_index(ChunkElementIndex::DoubleFirst),
        3
    );
    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_element_index(ChunkElementIndex::DoubleSecond),
        1
    );

    assert_eq!(typed_chunk.wrong_spell_element_positions(0), vec![0, 1]);
    assert_eq!(typed_chunk.wrong_spell_positions(0), vec![0, 1]);
}

#[test]
fn wrong_positions_2() {
    let typed_chunk = gen_chunk_inflight!(
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
            ActualKeyStroke::new(Duration::new(1, 0), 'j'.try_into().unwrap(), true),
            ActualKeyStroke::new(Duration::new(2, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(3, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(4, 0), 'j'.try_into().unwrap(), false),
            ActualKeyStroke::new(Duration::new(5, 0), 'o'.try_into().unwrap(), true)
        ],
        2,
        []
    );

    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_key_stroke_index(),
        vec![0, 3]
    );

    assert_eq!(typed_chunk.wrong_key_stroke_positions(0), vec![1]);

    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_element_index(ChunkElementIndex::OnlyFirst),
        3
    );
    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_element_index(ChunkElementIndex::DoubleFirst),
        0
    );
    assert_eq!(
        typed_chunk.wrong_key_stroke_count_of_element_index(ChunkElementIndex::DoubleSecond),
        0
    );

    assert_eq!(typed_chunk.wrong_spell_element_positions(0), vec![0]);
    assert_eq!(typed_chunk.wrong_spell_positions(0), vec![0, 1]);
}
