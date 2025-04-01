use std::num::NonZeroUsize;

use typing_engine::{
    LapRequest, QueryRequest, TypingEngine, VocabularyEntry, VocabularyOrder, VocabularyQuantifier,
    VocabularySeparator, VocabularySpellElement,
};

#[test]
fn construct_display_info_output_correct_display_string() {
    let mut engine = TypingEngine::new();
    let vocabularies = vec![VocabularyEntry::new(
        "ピョピョン吉".to_string(),
        vec![
            VocabularySpellElement::Normal("ぴ".to_string().try_into().unwrap()),
            VocabularySpellElement::Normal("ょ".to_string().try_into().unwrap()),
            VocabularySpellElement::Normal("ぴ".to_string().try_into().unwrap()),
            VocabularySpellElement::Normal("ょ".to_string().try_into().unwrap()),
            VocabularySpellElement::Normal("ん".to_string().try_into().unwrap()),
            VocabularySpellElement::Normal("きち".to_string().try_into().unwrap()),
        ],
    )
    .unwrap()];
    let vocabularies_slice: Vec<&VocabularyEntry> = vocabularies.iter().map(|v| v).collect();
    let query_request = QueryRequest::new(
        &vocabularies_slice,
        VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
        VocabularySeparator::None,
        VocabularyOrder::InOrder,
    );
    engine.init(query_request);
    engine
        .start()
        .expect("start() should be finished without errors");
    engine.stroke_key('p'.try_into().unwrap()).unwrap();
    engine.stroke_key('i'.try_into().unwrap()).unwrap();
    engine.stroke_key('x'.try_into().unwrap()).unwrap();
    engine.stroke_key('y'.try_into().unwrap()).unwrap();
    engine.stroke_key('o'.try_into().unwrap()).unwrap();
    engine.stroke_key('p'.try_into().unwrap()).unwrap();
    engine.stroke_key('t'.try_into().unwrap()).unwrap();
    engine.stroke_key('y'.try_into().unwrap()).unwrap();
    engine.stroke_key('o'.try_into().unwrap()).unwrap();
    engine.stroke_key('n'.try_into().unwrap()).unwrap();
    engine.stroke_key('j'.try_into().unwrap()).unwrap();

    let display_info = engine
        .construct_display_info(LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
        .unwrap();

    // Assertion for view
    assert_eq!(display_info.view_info().view(), "ピョピョン吉");
    assert_eq!(
        display_info.view_info().current_cursor_positions(),
        &vec![5]
    );
    assert_eq!(display_info.view_info().missed_positions(), &vec![2, 3, 5]);
    assert_eq!(display_info.view_info().last_position(), 5);
    // Assertion for spell
    assert_eq!(display_info.spell_info().spell(), "ぴょぴょんきち");
    assert_eq!(display_info.spell_info().last_position(), 6);
    assert_eq!(
        display_info.spell_info().current_cursor_positions(),
        &vec![5]
    );
    assert_eq!(display_info.view_info().missed_positions(), &vec![2, 3, 5]);
    // Assertion for key stroke
    assert_eq!(display_info.key_stroke_info().key_stroke(), "pixyopyonkiti");
    assert_eq!(display_info.key_stroke_info().current_cursor_position(), 9);
    assert_eq!(
        display_info.key_stroke_info().missed_positions(),
        &vec![6, 9]
    );
}
