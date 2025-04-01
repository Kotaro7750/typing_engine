use super::*;
use crate::typing_primitive_types::key_stroke::ActualKeyStroke;
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

