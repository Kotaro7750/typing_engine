use std::num::NonZeroUsize;

use std::time::Duration;
use typing_engine::{
    KeyStrokeChar, LapRequest, QueryRequest, TypingEngine, VocabularyEntry, VocabularyOrder,
    VocabularyQuantifier, VocabularySeparator, VocabularySpellElement,
};

fn init_engine_and_type_halfway() -> TypingEngine {
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

    engine
}

fn init_engine_and_finish_typing() -> TypingEngine {
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

    engine
}

#[test]
fn construct_display_info_output_correct_display_string() {
    let engine = init_engine_and_type_halfway();

    let display_info = engine
        .construct_display_info(LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
        .unwrap();

    // Assertion for view
    assert_eq!(display_info.view_info().view(), "ピョピョン吉");
    assert_eq!(
        display_info.view_info().current_cursor_positions(),
        &vec![5]
    );
    assert_eq!(display_info.view_info().wrong_positions(), &vec![2, 3, 5]);
    assert_eq!(display_info.view_info().last_position(), 5);
    // Assertion for spell
    assert_eq!(display_info.spell_info().spell(), "ぴょぴょんきち");
    assert_eq!(display_info.spell_info().last_position(), 6);
    assert_eq!(
        display_info.spell_info().current_cursor_positions(),
        &vec![5]
    );
    assert_eq!(display_info.view_info().wrong_positions(), &vec![2, 3, 5]);
    // Assertion for key stroke
    assert_eq!(display_info.key_stroke_info().key_stroke(), "pixyopyonkiti");
    assert_eq!(display_info.key_stroke_info().current_cursor_position(), 9);
    assert_eq!(
        display_info.key_stroke_info().wrong_positions(),
        &vec![6, 9]
    );
}

#[test]
fn construct_display_info_output_correct_summary_statistics() {
    let engine = init_engine_and_type_halfway();

    let display_info = engine
        .construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()))
        .unwrap();
    let key_stroke_summary = display_info.key_stroke_info().summary_statistics();
    let ideal_key_stroke_summary = display_info.ideal_key_stroke_info().summary_statistics();
    let spell_summary = display_info.spell_info().summary_statistics();

    assert_eq!(key_stroke_summary.whole_count(), 13);
    assert_eq!(key_stroke_summary.finished_count(), 9);
    assert_eq!(key_stroke_summary.wrong_count(), 2);
    assert_eq!(key_stroke_summary.completely_correct_count(), 8);
    assert_eq!(ideal_key_stroke_summary.whole_count(), 11);
    assert_eq!(ideal_key_stroke_summary.finished_count(), 7);
    assert_eq!(ideal_key_stroke_summary.wrong_count(), 2);
    assert_eq!(ideal_key_stroke_summary.completely_correct_count(), 6);
    assert_eq!(spell_summary.whole_count(), 7);
    assert_eq!(spell_summary.finished_count(), 5);
    assert_eq!(spell_summary.wrong_count(), 3);
    assert_eq!(spell_summary.completely_correct_count(), 3);
}

#[test]
fn construct_display_info_output_correct_lap_info() {
    let engine = init_engine_and_type_halfway();

    let display_info = engine
        .construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(2).unwrap()))
        .unwrap();
    let lap_info = display_info.lap_info();

    assert_eq!(
        lap_info.elapsed_times(),
        vec![
            Duration::from_millis(200),
            Duration::from_millis(400),
            Duration::from_millis(600),
            Duration::from_millis(900),
        ]
    );
    assert_eq!(
        lap_info.lap_times(),
        vec![
            Duration::from_millis(200),
            Duration::from_millis(200),
            Duration::from_millis(200),
            Duration::from_millis(300),
        ]
    );
    assert_eq!(
        lap_info.key_stroke_lap_end_positions(),
        vec![1, 3, 5, 7, 9, 11]
    );
    assert_eq!(lap_info.spell_lap_end_positions(), vec![0, 1, 2, 3, 5, 6]);
    assert_eq!(lap_info.chunk_lap_end_positions(), vec![0, 0, 1, 1, 3, 4]);
    assert_eq!(lap_info.view_lap_end_positions(), vec![0, 1, 2, 3, 5, 5]);
}

#[test]
fn construct_display_info_when_finished_output_correct_display_string() {
    let engine = init_engine_and_finish_typing();

    let display_info = engine
        .construct_display_info(LapRequest::Spell(NonZeroUsize::new(3).unwrap()))
        .unwrap();

    // Assertion for view
    assert_eq!(display_info.view_info().view(), "阿");
    assert_eq!(
        display_info.view_info().current_cursor_positions(),
        &vec![0]
    );
    assert_eq!(display_info.view_info().wrong_positions(), &vec![]);
    assert_eq!(display_info.view_info().last_position(), 0);
    // Assertion for spell
    assert_eq!(display_info.spell_info().spell(), "あ");
    assert_eq!(display_info.spell_info().last_position(), 0);
    assert_eq!(
        display_info.spell_info().current_cursor_positions(),
        &vec![1]
    );
    assert_eq!(display_info.view_info().wrong_positions(), &vec![]);
    // Assertion for key stroke
    assert_eq!(display_info.key_stroke_info().key_stroke(), "a");
    assert_eq!(display_info.key_stroke_info().current_cursor_position(), 1);
    assert_eq!(display_info.key_stroke_info().wrong_positions(), &vec![]);
}

#[test]
fn construct_result_output_correct_result() {
    let engine = init_engine_and_finish_typing();

    let lap_request = LapRequest::Spell(NonZeroUsize::new(3).unwrap());
    let result = engine
        .construct_result(lap_request)
        .expect("Failed to get result");

    assert_eq!(result.summary().key_stroke().whole_count(), 1);
    assert_eq!(result.summary().key_stroke().finished_count(), 1);
    assert_eq!(result.summary().key_stroke().completely_correct_count(), 1);
    assert_eq!(result.summary().key_stroke().wrong_count(), 0);

    assert_eq!(result.summary().ideal_key_stroke().whole_count(), 1);
    assert_eq!(result.summary().ideal_key_stroke().finished_count(), 1);
    assert_eq!(
        result
            .summary()
            .ideal_key_stroke()
            .completely_correct_count(),
        1
    );
    assert_eq!(result.summary().ideal_key_stroke().wrong_count(), 0);

    assert_eq!(result.summary().spell().whole_count(), 1);
    assert_eq!(result.summary().spell().finished_count(), 1);
    assert_eq!(result.summary().spell().completely_correct_count(), 1);
    assert_eq!(result.summary().spell().wrong_count(), 0);

    assert_eq!(result.summary().chunk().whole_count(), 1);
    assert_eq!(result.summary().chunk().finished_count(), 1);
    assert_eq!(result.summary().chunk().completely_correct_count(), 1);
    assert_eq!(result.summary().chunk().wrong_count(), 0);

    assert_eq!(result.total_time(), std::time::Duration::from_millis(100));
}

#[test]
fn construct_result_output_correct_skill_statistics() {
    let engine = init_engine_and_finish_typing();

    let lap_request = LapRequest::Spell(NonZeroUsize::new(3).unwrap());
    let result = engine
        .construct_result(lap_request)
        .expect("Failed to get result");
    let skill_statistics = result.skill_statistics();

    assert_eq!(skill_statistics.single_key_stroke().len(), 1);
    assert_eq!(
        skill_statistics
            .single_key_stroke()
            .get(0)
            .unwrap()
            .entity(),
        &<char as TryInto<KeyStrokeChar>>::try_into('a').unwrap()
    );
    assert_eq!(
        skill_statistics
            .single_key_stroke()
            .get(0)
            .unwrap()
            .accuracy(),
        1.0
    );
    assert_eq!(
        skill_statistics
            .single_key_stroke()
            .get(0)
            .unwrap()
            .average_time(),
        Duration::from_millis(100)
    );
    assert_eq!(
        skill_statistics
            .single_key_stroke()
            .get(0)
            .unwrap()
            .wrong_count_ranking(),
        vec![]
    );
}
