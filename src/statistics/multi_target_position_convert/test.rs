use super::*;

#[test]
fn convert_between_key_stroke_delta_1() {
    // position conversion of key stroke to ideal key stroke for double spell like "きょ"
    let from = KeyStrokeElementCount::new(&vec![2, 3]);
    let to = KeyStrokeElementCount::new(&vec![3]);
    let spell_count = 2;

    assert_eq!(
        convert_between_key_stroke_delta(&from, &to, spell_count, 1),
        1
    );
    assert_eq!(
        convert_between_key_stroke_delta(&from, &to, spell_count, 2),
        1
    );
    assert_eq!(
        convert_between_key_stroke_delta(&from, &to, spell_count, 3),
        2
    );
    assert_eq!(
        convert_between_key_stroke_delta(&from, &to, spell_count, 4),
        3
    );
    assert_eq!(
        convert_between_key_stroke_delta(&from, &to, spell_count, 5),
        3
    );
}

#[test]
fn convert_between_key_stroke_delta_2() {
    // position conversion of key stroke to ideal key stroke for single spell like "ん"
    let from = KeyStrokeElementCount::new(&vec![2]);
    let to = KeyStrokeElementCount::new(&vec![1]);
    let spell_count = 1;

    assert_eq!(
        convert_between_key_stroke_delta(&from, &to, spell_count, 1),
        1
    );
    assert_eq!(
        convert_between_key_stroke_delta(&from, &to, spell_count, 2),
        1
    );
}

#[test]
fn multi_target_delta_converter_1() {
    let m = MultiTargetDeltaConverter::new(
        2,
        KeyStrokeElementCount::new(&vec![3]),
        KeyStrokeElementCount::new(&vec![2, 3]),
        BaseTarget::Chunk,
    );

    assert_eq!(m.chunk_delta(&vec![1]), vec![1]);
    assert_eq!(m.spell_delta(&vec![1]), vec![2]);
    assert_eq!(m.ideal_key_stroke_delta(&vec![1]), vec![3]);
    assert_eq!(m.key_stroke_delta(&vec![1]), vec![5]);
}

#[test]
fn multi_target_delta_converter_2() {
    let m = MultiTargetDeltaConverter::new(
        2,
        KeyStrokeElementCount::new(&vec![3]),
        KeyStrokeElementCount::new(&vec![2, 3]),
        BaseTarget::Spell,
    );

    assert_eq!(m.chunk_delta(&vec![1, 2]), vec![1, 1]);

    assert_eq!(m.spell_delta(&vec![1, 2]), vec![1, 2]);
    assert_eq!(m.ideal_key_stroke_delta(&vec![1, 2]), vec![1, 3]);
    assert_eq!(m.key_stroke_delta(&vec![1, 2]), vec![2, 5]);
}

#[test]
fn multi_target_delta_converter_3() {
    let m = MultiTargetDeltaConverter::new(
        2,
        KeyStrokeElementCount::new(&vec![3]),
        KeyStrokeElementCount::new(&vec![2, 3]),
        BaseTarget::IdealKeyStroke,
    );

    assert_eq!(m.chunk_delta(&vec![1, 2, 3]), vec![1, 1, 1]);
    assert_eq!(m.spell_delta(&vec![1, 2, 3]), vec![1, 2, 2]);
    assert_eq!(m.ideal_key_stroke_delta(&vec![1, 2, 3]), vec![1, 2, 3]);
    assert_eq!(m.key_stroke_delta(&vec![1, 2, 3]), vec![2, 4, 5]);
}

#[test]
fn multi_target_delta_converter_4() {
    let m = MultiTargetDeltaConverter::new(
        2,
        KeyStrokeElementCount::new(&vec![3]),
        KeyStrokeElementCount::new(&vec![2, 3]),
        BaseTarget::KeyStroke,
    );

    assert_eq!(m.chunk_delta(&vec![1, 2, 3, 4, 5]), vec![1, 1, 1, 1, 1]);
    assert_eq!(m.spell_delta(&vec![1, 2, 3, 4, 5]), vec![1, 1, 2, 2, 2]);
    assert_eq!(
        m.ideal_key_stroke_delta(&vec![1, 2, 3, 4, 5]),
        vec![1, 1, 2, 3, 3]
    );
    assert_eq!(
        m.key_stroke_delta(&vec![1, 2, 3, 4, 5]),
        vec![1, 2, 3, 4, 5]
    );
}

#[test]
fn multi_target_delta_converter_5() {
    let m = MultiTargetDeltaConverter::new(
        1,
        KeyStrokeElementCount::new(&vec![1]),
        KeyStrokeElementCount::new(&vec![2]),
        BaseTarget::Chunk,
    );

    assert_eq!(m.chunk_delta(&vec![1]), vec![1]);
    assert_eq!(m.spell_delta(&vec![1]), vec![1]);
    assert_eq!(m.ideal_key_stroke_delta(&vec![1]), vec![1]);
    assert_eq!(m.key_stroke_delta(&vec![1]), vec![2]);
}

#[test]
fn multi_target_delta_converter_6() {
    let m = MultiTargetDeltaConverter::new(
        1,
        KeyStrokeElementCount::new(&vec![1]),
        KeyStrokeElementCount::new(&vec![2]),
        BaseTarget::Spell,
    );

    assert_eq!(m.chunk_delta(&vec![1]), vec![1]);
    assert_eq!(m.spell_delta(&vec![1]), vec![1]);
    assert_eq!(m.ideal_key_stroke_delta(&vec![1]), vec![1]);
    assert_eq!(m.key_stroke_delta(&vec![1]), vec![2]);
}

#[test]
fn multi_target_delta_converter_7() {
    let m = MultiTargetDeltaConverter::new(
        1,
        KeyStrokeElementCount::new(&vec![1]),
        KeyStrokeElementCount::new(&vec![2]),
        BaseTarget::IdealKeyStroke,
    );

    assert_eq!(m.chunk_delta(&vec![1]), vec![1]);
    assert_eq!(m.spell_delta(&vec![1]), vec![1]);
    assert_eq!(m.ideal_key_stroke_delta(&vec![1]), vec![1]);
    assert_eq!(m.key_stroke_delta(&vec![1]), vec![2]);
}

#[test]
fn multi_target_delta_converter_8() {
    let m = MultiTargetDeltaConverter::new(
        1,
        KeyStrokeElementCount::new(&vec![1]),
        KeyStrokeElementCount::new(&vec![2]),
        BaseTarget::KeyStroke,
    );

    assert_eq!(m.chunk_delta(&vec![1, 2]), vec![1, 1]);
    assert_eq!(m.spell_delta(&vec![1, 2]), vec![1, 1]);
    assert_eq!(m.ideal_key_stroke_delta(&vec![1, 2]), vec![1, 1]);
    assert_eq!(m.key_stroke_delta(&vec![1, 2]), vec![1, 2]);
}
