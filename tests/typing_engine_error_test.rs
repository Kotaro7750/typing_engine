use std::num::NonZeroUsize;
use std::time::Duration;

use typing_engine::{
    KeyStrokeChar, LapRequest, QueryRequest, TypingEngine, VocabularyEntry, VocabularyOrder,
    VocabularyQuantifier, VocabularySeparator, VocabularySpellElement,
};

fn build_entry(view: &str, spell: &str) -> VocabularyEntry {
    VocabularyEntry::new(
        view.to_string(),
        vec![VocabularySpellElement::Normal(
            spell.to_string().try_into().unwrap(),
        )],
    )
    .unwrap()
}

fn build_query<'a>(entries: &'a [&'a VocabularyEntry]) -> QueryRequest<'a> {
    QueryRequest::new(
        entries,
        VocabularyQuantifier::Vocabulary(NonZeroUsize::new(entries.len()).unwrap()),
        VocabularySeparator::None,
        VocabularyOrder::InOrder,
    )
}

fn assert_err_message(err: typing_engine::TypingEngineError, expected: &str) {
    assert_eq!(err.to_string(), expected);
}

#[test]
fn stroke_key_without_active_target_returns_not_initialized() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));

    let result = engine.stroke_key_with_elapsed_time(
        KeyStrokeChar::try_from('a').unwrap(),
        Duration::from_millis(100),
    );

    assert_err_message(result.unwrap_err(), "not initialized");
}

#[test]
fn start_before_init_returns_not_initialized() {
    let mut engine = TypingEngine::new();

    #[allow(deprecated)]
    let result = engine.start();

    assert_err_message(result.unwrap_err(), "not initialized");
}

#[test]
fn append_query_before_init_returns_not_initialized() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();

    #[allow(deprecated)]
    let result = engine.append_query(build_query(vec![&entry_a].as_ref()));

    assert_err_message(result.unwrap_err(), "not initialized");
}

#[test]
fn construct_display_info_before_start_returns_not_started() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    #[allow(deprecated)]
    engine.init(build_query(vec![&entry_a].as_ref()));

    #[allow(deprecated)]
    let result =
        engine.construct_display_info(LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()));

    assert_err_message(result.unwrap_err(), "not started");
}

#[test]
fn construct_result_before_start_returns_not_started() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    #[allow(deprecated)]
    engine.init(build_query(vec![&entry_a].as_ref()));

    #[allow(deprecated)]
    let result = engine.construct_result(LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()));

    assert_err_message(result.unwrap_err(), "not started");
}

#[test]
fn construct_result_before_finish_returns_not_finished() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    #[allow(deprecated)]
    engine.init(build_query(vec![&entry_a].as_ref()));
    #[allow(deprecated)]
    engine.start().unwrap();

    #[allow(deprecated)]
    let result = engine.construct_result(LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()));

    assert_err_message(result.unwrap_err(), "not finished");
}

#[test]
fn stroke_key_before_start_returns_not_started() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.activate_target(0, Duration::from_millis(0));

    let result = engine.stroke_key_with_elapsed_time(
        KeyStrokeChar::try_from('a').unwrap(),
        Duration::from_millis(100),
    );

    assert_err_message(result.unwrap_err(), "not started");
}

#[test]
fn start_target_twice_returns_already_started() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.start_target(0).unwrap();

    let result = engine.start_target(0);

    assert_err_message(result.unwrap_err(), "already started");
}

#[test]
fn start_target_with_invalid_index_returns_target_not_found() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));

    let result = engine.start_target(1);

    assert_err_message(result.unwrap_err(), "target not found");
}

#[test]
fn append_query_to_target_with_invalid_index_returns_target_not_found() {
    let entry_a = build_entry("阿", "あ");
    let entry_i = build_entry("胃", "い");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));

    let result = engine.append_query_to_target(2, build_query(vec![&entry_i].as_ref()));

    assert_err_message(result.unwrap_err(), "target not found");
}

#[test]
fn construct_display_info_of_target_before_start_returns_must_be_started() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));

    let result = engine
        .construct_display_info_of_target(0, LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()));

    assert_err_message(result.unwrap_err(), "not started");
}

#[test]
fn construct_display_info_of_target_with_invalid_index_returns_target_not_found() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.start_target(0).unwrap();

    let result = engine
        .construct_display_info_of_target(1, LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()));

    assert_err_message(result.unwrap_err(), "target not found");
}

#[test]
fn stroke_key_after_finished_returns_already_finished() {
    let entry_a = build_entry("阿", "あ");
    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.activate_target(0, Duration::from_millis(0));
    engine.start_target(0).unwrap();

    engine
        .stroke_key_with_elapsed_time(
            KeyStrokeChar::try_from('a').unwrap(),
            Duration::from_millis(100),
        )
        .unwrap();

    let result = engine.stroke_key_with_elapsed_time(
        KeyStrokeChar::try_from('a').unwrap(),
        Duration::from_millis(200),
    );

    assert_err_message(result.unwrap_err(), "already finished");
}
