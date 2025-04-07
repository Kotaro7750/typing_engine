use std::num::NonZeroUsize;

use std::time::Duration;
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
    engine
        .stroke_key_with_elapsed_time('p'.try_into().unwrap(), Duration::from_millis(100))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('i'.try_into().unwrap(), Duration::from_millis(200))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('x'.try_into().unwrap(), Duration::from_millis(300))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('y'.try_into().unwrap(), Duration::from_millis(400))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('o'.try_into().unwrap(), Duration::from_millis(500))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('p'.try_into().unwrap(), Duration::from_millis(600))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('t'.try_into().unwrap(), Duration::from_millis(700))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('y'.try_into().unwrap(), Duration::from_millis(800))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('o'.try_into().unwrap(), Duration::from_millis(900))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('n'.try_into().unwrap(), Duration::from_millis(1000))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('j'.try_into().unwrap(), Duration::from_millis(1100))
        .unwrap();

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

#[test]
fn construct_display_info_when_finished_output_correct_display_string() {
    let mut engine = TypingEngine::new();
    let vocabularies = vec![VocabularyEntry::new(
        "阿".to_string(),
        vec![VocabularySpellElement::Normal(
            "あ".to_string().try_into().unwrap(),
        )],
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
    engine.stroke_key('a'.try_into().unwrap()).unwrap();

    let display_info = engine
        .construct_display_info(LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
        .unwrap();

    // Assertion for view
    assert_eq!(display_info.view_info().view(), "阿");
    assert_eq!(
        display_info.view_info().current_cursor_positions(),
        &vec![0]
    );
    assert_eq!(display_info.view_info().missed_positions(), &vec![]);
    assert_eq!(display_info.view_info().last_position(), 0);
    // Assertion for spell
    assert_eq!(display_info.spell_info().spell(), "あ");
    assert_eq!(display_info.spell_info().last_position(), 0);
    assert_eq!(
        display_info.spell_info().current_cursor_positions(),
        &vec![1]
    );
    assert_eq!(display_info.view_info().missed_positions(), &vec![]);
    // Assertion for key stroke
    assert_eq!(display_info.key_stroke_info().key_stroke(), "a");
    assert_eq!(display_info.key_stroke_info().current_cursor_position(), 1);
    assert_eq!(display_info.key_stroke_info().missed_positions(), &vec![]);
}

#[test]
fn construct_result_output_correct_result() {
    let mut engine = TypingEngine::new();
    let vocabularies = vec![VocabularyEntry::new(
        "阿".to_string(),
        vec![VocabularySpellElement::Normal(
            "あ".to_string().try_into().unwrap(),
        )],
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
    engine
        .stroke_key_with_elapsed_time('a'.try_into().unwrap(), Duration::from_millis(100))
        .unwrap();

    let result = engine
        .construst_result_statistics(LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
        .expect("Failed to get result");

    assert_eq!(result.key_stroke().whole_count(), 1);
    assert_eq!(result.key_stroke().completely_correct_count(), 1);
    assert_eq!(result.key_stroke().missed_count(), 0);

    assert_eq!(result.ideal_key_stroke().whole_count(), 1);
    assert_eq!(result.ideal_key_stroke().completely_correct_count(), 1);
    assert_eq!(result.ideal_key_stroke().missed_count(), 0);

    assert_eq!(result.total_time(), std::time::Duration::from_millis(100));
}
