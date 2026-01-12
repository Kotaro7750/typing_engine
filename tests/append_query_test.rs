use std::num::NonZeroUsize;
use std::time::Duration;

use typing_engine::TypingResultSummary;
use typing_engine::{
    LapRequest, QueryRequest, TypingEngine, VocabularyEntry, VocabularyOrder, VocabularyQuantifier,
    VocabularySeparator, VocabularySpellElement,
};

#[test]
fn append_query_updates_statistics_without_affecting_finished_counts() {
    let mut engine = init_engine_after_wrong_then_correct();

    append_third_vocabulary(&mut engine);

    let display_info_after = construct_display_info(&mut engine);
    assert_display_summary_counts(&display_info_after, 3, 1, 1, 0);
}

#[test]
fn append_query_updates_display_string_without_moving_cursor_or_wrong_positions() {
    let mut engine = init_engine_after_wrong_then_correct();
    let display_info_before = construct_display_info(&mut engine);

    append_third_vocabulary(&mut engine);

    let display_info_after = construct_display_info(&mut engine);
    // Check display strings and last positions are updated correctly
    assert_eq!(display_info_after.view_info().view(), "阿胃宇");
    assert_eq!(display_info_after.spell_info().spell(), "あいう");
    assert_eq!(display_info_after.key_stroke_info().key_stroke(), "aiu");
    assert_eq!(display_info_after.view_info().last_position(), 2);
    assert_eq!(display_info_after.spell_info().last_position(), 2);

    // Check cursor positions and wrong positions remain unchanged
    assert_eq!(
        display_info_after.view_info().current_cursor_positions(),
        display_info_before.view_info().current_cursor_positions()
    );
    assert_eq!(
        display_info_after.view_info().wrong_positions(),
        display_info_before.view_info().wrong_positions(),
    );
    assert_eq!(
        display_info_after.spell_info().current_cursor_positions(),
        display_info_before.spell_info().current_cursor_positions(),
    );
    assert_eq!(
        display_info_after.spell_info().wrong_positions(),
        display_info_before.spell_info().wrong_positions(),
    );
    assert_eq!(
        display_info_after
            .key_stroke_info()
            .current_cursor_position(),
        display_info_before
            .key_stroke_info()
            .current_cursor_position(),
    );
    assert_eq!(
        display_info_after.key_stroke_info().wrong_positions(),
        display_info_before.key_stroke_info().wrong_positions(),
    );
}

#[test]
fn append_query_allows_finish_and_result_is_correct() {
    let mut engine = init_engine_after_wrong_then_correct();
    append_third_vocabulary(&mut engine);

    engine
        .stroke_key_with_elapsed_time('i'.try_into().unwrap(), Duration::from_millis(300))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('u'.try_into().unwrap(), Duration::from_millis(400))
        .unwrap();

    let result = engine
        .construct_result(LapRequest::Spell(NonZeroUsize::new(2).unwrap()))
        .unwrap();
    assert_result_summary_counts(result.summary(), 3, 3, 1, 2);

    assert_eq!(result.total_time(), Duration::from_millis(400));
}

#[test]
fn append_query_after_finished_allows_second_run_and_result_is_correct() {
    let mut engine = init_engine_after_wrong_then_correct();
    engine
        .stroke_key_with_elapsed_time('i'.try_into().unwrap(), Duration::from_millis(300))
        .unwrap();

    append_third_vocabulary(&mut engine);

    engine
        .stroke_key_with_elapsed_time('u'.try_into().unwrap(), Duration::from_millis(400))
        .unwrap();

    let result = engine
        .construct_result(LapRequest::Spell(NonZeroUsize::new(2).unwrap()))
        .unwrap();
    assert_result_summary_counts(result.summary(), 3, 3, 1, 2);

    assert_eq!(result.total_time(), Duration::from_millis(400));
}

fn build_vocabularies() -> Vec<VocabularyEntry> {
    vec![
        VocabularyEntry::new(
            "阿".to_string(),
            vec![VocabularySpellElement::Normal(
                "あ".to_string().try_into().unwrap(),
            )],
        )
        .unwrap(),
        VocabularyEntry::new(
            "胃".to_string(),
            vec![VocabularySpellElement::Normal(
                "い".to_string().try_into().unwrap(),
            )],
        )
        .unwrap(),
        VocabularyEntry::new(
            "宇".to_string(),
            vec![VocabularySpellElement::Normal(
                "う".to_string().try_into().unwrap(),
            )],
        )
        .unwrap(),
    ]
}

fn init_engine_after_wrong_then_correct() -> TypingEngine {
    let mut engine = TypingEngine::new();
    let vocabularies = build_vocabularies();

    let query_request = QueryRequest::new(
        vec![&vocabularies[0], &vocabularies[1]].as_ref(),
        VocabularyQuantifier::Vocabulary(NonZeroUsize::new(2).unwrap()),
        VocabularySeparator::None,
        VocabularyOrder::InOrder,
    );
    engine.init(query_request);
    engine
        .start()
        .expect("start() should be finished without errors");

    engine
        .stroke_key_with_elapsed_time('q'.try_into().unwrap(), Duration::from_millis(100))
        .unwrap();
    engine
        .stroke_key_with_elapsed_time('a'.try_into().unwrap(), Duration::from_millis(200))
        .unwrap();

    let display_info_before = construct_display_info(&mut engine);
    assert_display_summary_counts(&display_info_before, 2, 1, 1, 0);

    assert_eq!(display_info_before.view_info().view(), "阿胃");
    assert_eq!(display_info_before.spell_info().spell(), "あい");
    assert_eq!(display_info_before.key_stroke_info().key_stroke(), "ai");
    assert_eq!(display_info_before.view_info().last_position(), 1);
    assert_eq!(display_info_before.spell_info().last_position(), 1);
    assert_eq!(
        display_info_before.view_info().current_cursor_positions(),
        &vec![1]
    );
    assert_eq!(display_info_before.view_info().wrong_positions(), &vec![0]);
    assert_eq!(
        display_info_before.spell_info().current_cursor_positions(),
        &vec![1]
    );
    assert_eq!(display_info_before.spell_info().wrong_positions(), &vec![0]);
    assert_eq!(
        display_info_before
            .key_stroke_info()
            .current_cursor_position(),
        1
    );
    assert_eq!(
        display_info_before.key_stroke_info().wrong_positions(),
        &vec![0]
    );

    engine
}

fn append_third_vocabulary(engine: &mut TypingEngine) {
    let vocabularies = build_vocabularies();

    let append_entries: Vec<&VocabularyEntry> = vec![&vocabularies[2]];
    let append_request = QueryRequest::new(
        &append_entries,
        VocabularyQuantifier::Vocabulary(NonZeroUsize::new(1).unwrap()),
        VocabularySeparator::None,
        VocabularyOrder::InOrder,
    );
    engine
        .append_query(append_request)
        .expect("append_query() should be finished without errors");
}

fn construct_display_info(engine: &mut TypingEngine) -> typing_engine::DisplayInfo {
    engine
        .construct_display_info(LapRequest::Spell(NonZeroUsize::new(2).unwrap()))
        .unwrap()
}

fn assert_display_summary_counts(
    display_info: &typing_engine::DisplayInfo,
    whole: usize,
    finished: usize,
    wrong: usize,
    completely_correct: usize,
) {
    let key_stroke = display_info.key_stroke_info().summary_statistics();
    let ideal = display_info.ideal_key_stroke_info().summary_statistics();
    let spell = display_info.spell_info().summary_statistics();

    assert_eq!(key_stroke.whole_count(), whole);
    assert_eq!(key_stroke.finished_count(), finished);
    assert_eq!(key_stroke.wrong_count(), wrong);
    assert_eq!(key_stroke.completely_correct_count(), completely_correct);
    assert_eq!(ideal.whole_count(), whole);
    assert_eq!(ideal.finished_count(), finished);
    assert_eq!(ideal.wrong_count(), wrong);
    assert_eq!(ideal.completely_correct_count(), completely_correct);
    assert_eq!(spell.whole_count(), whole);
    assert_eq!(spell.finished_count(), finished);
    assert_eq!(spell.wrong_count(), wrong);
    assert_eq!(spell.completely_correct_count(), completely_correct);
}

fn assert_result_summary_counts(
    summary: &TypingResultSummary,
    whole: usize,
    finished: usize,
    wrong: usize,
    completely_correct: usize,
) {
    assert_eq!(summary.key_stroke().whole_count(), whole);
    assert_eq!(summary.key_stroke().finished_count(), finished);
    assert_eq!(summary.key_stroke().wrong_count(), wrong);
    assert_eq!(
        summary.key_stroke().completely_correct_count(),
        completely_correct
    );

    assert_eq!(summary.ideal_key_stroke().whole_count(), whole);
    assert_eq!(summary.ideal_key_stroke().finished_count(), finished);
    assert_eq!(summary.ideal_key_stroke().wrong_count(), wrong);
    assert_eq!(
        summary.ideal_key_stroke().completely_correct_count(),
        completely_correct
    );

    assert_eq!(summary.spell().whole_count(), whole);
    assert_eq!(summary.spell().finished_count(), finished);
    assert_eq!(summary.spell().wrong_count(), wrong);
    assert_eq!(
        summary.spell().completely_correct_count(),
        completely_correct
    );

    assert_eq!(summary.chunk().whole_count(), whole);
    assert_eq!(summary.chunk().finished_count(), finished);
    assert_eq!(summary.chunk().wrong_count(), wrong);
    assert_eq!(
        summary.chunk().completely_correct_count(),
        completely_correct
    );
}
