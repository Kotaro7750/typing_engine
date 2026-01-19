use std::num::NonZeroUsize;
use std::time::Duration;

use typing_engine::{
    LapRequest, QueryRequest, TypingEngine, VocabularyEntry, VocabularyOrder, VocabularyQuantifier,
    VocabularySeparator, VocabularySpellElement,
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

#[test]
fn add_target_increments_count_and_activate_updates_active_index() {
    let entry_a = build_entry("阿", "あ");
    let entry_i = build_entry("胃", "い");
    let entry_u = build_entry("宇", "う");

    let mut engine = TypingEngine::new();

    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.add_target(build_query(vec![&entry_i].as_ref()));
    engine.add_target(build_query(vec![&entry_u].as_ref()));

    let count_after_add = engine.target_count();
    let active_before = engine.active_target_index();

    engine.activate_target(1, Duration::from_millis(0));
    let active_after_first = engine.active_target_index();

    engine.activate_target(2, Duration::from_millis(10));
    let active_after_second = engine.active_target_index();

    assert_eq!(count_after_add, 3);
    assert_eq!(active_before, None);
    assert_eq!(active_after_first, Some(1));
    assert_eq!(active_after_second, Some(2));
}

#[test]
fn activate_target_with_invalid_index_does_not_change_active_target() {
    let entry_a = build_entry("阿", "あ");
    let entry_i = build_entry("胃", "い");

    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.add_target(build_query(vec![&entry_i].as_ref()));

    engine.activate_target(0, Duration::from_millis(0));
    let active_before = engine.active_target_index();

    engine.activate_target(2, Duration::from_millis(10));
    let active_after = engine.active_target_index();

    assert_eq!(active_before, Some(0));
    assert_eq!(active_after, Some(0));
}

#[test]
fn multi_target_switching_ignores_inactive_time_for_local_elapsed() {
    let entry_a = build_entry("阿", "あ");
    let entry_i = build_entry("胃", "い");
    let entry_u = build_entry("宇", "う");

    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a, &entry_i].as_ref()));
    engine.add_target(build_query(vec![&entry_u].as_ref()));

    engine.activate_target(0, Duration::from_millis(0));
    engine.start_target(0).unwrap();
    engine
        .stroke_key_with_elapsed_time('a'.try_into().unwrap(), Duration::from_millis(100))
        .unwrap();

    engine.activate_target(1, Duration::from_millis(200));
    engine.start_target(1).unwrap();
    engine
        .stroke_key_with_elapsed_time('u'.try_into().unwrap(), Duration::from_millis(300))
        .unwrap();

    engine.activate_target(0, Duration::from_millis(1000));
    engine
        .stroke_key_with_elapsed_time('i'.try_into().unwrap(), Duration::from_millis(1200))
        .unwrap();

    let lap_request = LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap());
    let a_display = engine
        .construct_display_info_of_target(0, lap_request)
        .unwrap();
    let b_display = engine
        .construct_display_info_of_target(1, LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()))
        .unwrap();

    assert_eq!(
        a_display.lap_info().elapsed_times(),
        vec![Duration::from_millis(100), Duration::from_millis(300)]
    );
    assert_eq!(
        b_display.lap_info().elapsed_times(),
        vec![Duration::from_millis(100)]
    );
}

#[test]
fn append_query_to_target_updates_only_selected_target() {
    let entry_a = build_entry("阿", "あ");
    let entry_i = build_entry("胃", "い");
    let entry_u = build_entry("宇", "う");

    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.add_target(build_query(vec![&entry_i].as_ref()));

    engine.activate_target(0, Duration::from_millis(0));
    engine.start_target(0).unwrap();
    engine.activate_target(1, Duration::from_millis(0));
    engine.start_target(1).unwrap();

    let a_before = engine
        .construct_display_info_of_target(0, LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()))
        .unwrap();
    let b_before = engine
        .construct_display_info_of_target(1, LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()))
        .unwrap();

    engine
        .append_query_to_target(0, build_query(vec![&entry_u].as_ref()))
        .unwrap();

    let a_after = engine
        .construct_display_info_of_target(0, LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()))
        .unwrap();
    let b_after = engine
        .construct_display_info_of_target(1, LapRequest::KeyStroke(NonZeroUsize::new(1).unwrap()))
        .unwrap();

    assert_eq!(a_before.view_info().view(), "阿");
    assert_eq!(a_after.view_info().view(), "阿宇");
    assert_eq!(a_before.spell_info().spell(), "あ");
    assert_eq!(a_after.spell_info().spell(), "あう");

    assert_eq!(b_before.view_info().view(), "胃");
    assert_eq!(b_after.view_info().view(), "胃");
    assert_eq!(b_before.spell_info().spell(), "い");
    assert_eq!(b_after.spell_info().spell(), "い");

    assert_display_summary_counts(&a_before, 1, 0, 0, 0);
    assert_display_summary_counts(&a_after, 2, 0, 0, 0);
    assert_display_summary_counts(&b_before, 1, 0, 0, 0);
    assert_display_summary_counts(&b_after, 1, 0, 0, 0);
}

#[test]
fn construct_aggregated_result_sums_started_targets_ignores_unstarted() {
    let entry_a = build_entry("阿", "あ");
    let entry_i = build_entry("胃", "い");
    let entry_u = build_entry("宇", "う");

    let mut engine = TypingEngine::new();
    engine.add_target(build_query(vec![&entry_a].as_ref()));
    engine.add_target(build_query(vec![&entry_i].as_ref()));
    engine.add_target(build_query(vec![&entry_u].as_ref()));

    engine.activate_target(0, Duration::from_millis(0));
    engine.start_target(0).unwrap();
    engine
        .stroke_key_with_elapsed_time('a'.try_into().unwrap(), Duration::from_millis(100))
        .unwrap();

    engine.activate_target(1, Duration::from_millis(200));
    engine.start_target(1).unwrap();
    engine
        .stroke_key_with_elapsed_time('i'.try_into().unwrap(), Duration::from_millis(300))
        .unwrap();

    let result = engine.construct_aggregated_result().unwrap();
    let summary = result.summary();

    assert_eq!(summary.key_stroke().whole_count(), 2);
    assert_eq!(summary.key_stroke().finished_count(), 2);
    assert_eq!(summary.key_stroke().wrong_count(), 0);
    assert_eq!(summary.key_stroke().completely_correct_count(), 2);

    assert_eq!(summary.ideal_key_stroke().whole_count(), 2);
    assert_eq!(summary.ideal_key_stroke().finished_count(), 2);
    assert_eq!(summary.ideal_key_stroke().wrong_count(), 0);
    assert_eq!(summary.ideal_key_stroke().completely_correct_count(), 2);

    assert_eq!(summary.spell().whole_count(), 2);
    assert_eq!(summary.spell().finished_count(), 2);
    assert_eq!(summary.spell().wrong_count(), 0);
    assert_eq!(summary.spell().completely_correct_count(), 2);

    assert_eq!(summary.chunk().whole_count(), 2);
    assert_eq!(summary.chunk().finished_count(), 2);
    assert_eq!(summary.chunk().wrong_count(), 0);
    assert_eq!(summary.chunk().completely_correct_count(), 2);

    assert_eq!(result.total_time(), Duration::from_millis(300));
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
