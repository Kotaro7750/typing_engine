use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

// キーストローク・綴り・表示文字列・語彙といった対象に関するタイピング中に使われることを意図した統計情報
trait OnTypingStatistics {
    /// 対象を打ち終えた時に呼ぶ
    /// completely_correctはその対象を1回もミスタイプなしで打ち終えたかどうかを表す
    /// 今回打ち終えた対象がラップ末だった場合にはtrueを返す
    fn on_finished(
        &mut self,
        delta: usize,
        completely_correct: bool,
        elapsed_time: Duration,
    ) -> bool;

    /// 対象を統計に加えるときに呼ぶ
    fn on_target_add(&mut self, delta: usize);

    /// 対象をミスタイプしたときに呼ぶ
    fn on_wrong(&mut self, delta: usize);
}

// タイピング中に変わることのない対象列に対しての統計情報
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OnTypingStatisticsStaticTarget {
    // 対象を何個打ち終えたか
    finished_count: usize,
    // クエリに対象は何個あるか
    whole_count: usize,
    // 1回もミスタイプしないで打ち終えた対象は何個あるか
    completely_correct_count: usize,
    // ミスタイプした対象は重複込みで何個あるか
    // 重複というのは1つの対象に対して複数回ミスタイプした場合にもカウントされるため
    wrong_count: usize,
    // ラップ当たりの対象数
    targets_per_lap: Option<NonZeroUsize>,
    // 各ラップが末の経過時間
    lap_end_time: Option<Vec<Duration>>,
}

impl OnTypingStatisticsStaticTarget {
    pub(crate) fn new(
        finished_count: usize,
        whole_count: usize,
        completely_correct_count: usize,
        wrong_count: usize,
        targets_per_lap: Option<NonZeroUsize>,
        lap_end_time: Option<Vec<Duration>>,
    ) -> Self {
        assert_eq!(targets_per_lap.is_some(), lap_end_time.is_some());

        Self {
            finished_count,
            whole_count,
            completely_correct_count,
            wrong_count,
            targets_per_lap,
            lap_end_time,
        }
    }
}

impl OnTypingStatistics for OnTypingStatisticsStaticTarget {
    fn on_finished(
        &mut self,
        delta: usize,
        completely_correct: bool,
        elapsed_time: Duration,
    ) -> bool {
        let lap_finish_num = if let Some(tpl) = &self.targets_per_lap {
            ((self.finished_count + delta) / tpl.get()) - (self.finished_count / tpl.get())
        } else {
            0
        };

        if lap_finish_num != 0 {
            for _ in 0..lap_finish_num {
                self.lap_end_time.as_mut().unwrap().push(elapsed_time);
            }
        }

        self.finished_count += delta;

        if completely_correct {
            self.completely_correct_count += delta;
        }

        lap_finish_num != 0
    }

    fn on_target_add(&mut self, delta: usize) {
        self.whole_count += delta;
    }

    fn on_wrong(&mut self, delta: usize) {
        self.wrong_count += delta;
    }
}

// タイピング中に動的に変わりうる対象列に対しての統計情報
// 基本的にはStaticTargetと同じだが理想的でない対象列になりうるという点が異なる
// ex. キーストローク列は理想的な入力を行った場合とそうでない場合で全体の数が変わりうる
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OnTypingStatisticsDynamicTarget {
    finished_count: usize,
    // クエリに対象は動的に何個あるか
    whole_count: usize,
    // クエリに対象は理想的には何個あるか
    ideal_whole_count: usize,
    completely_correct_count: usize,
    wrong_count: usize,
    // ラップ当たりの対象数
    targets_per_lap: Option<NonZeroUsize>,
    // 各ラップが末の経過時間
    lap_end_time: Option<Vec<Duration>>,
}

impl OnTypingStatisticsDynamicTarget {
    pub(crate) fn new(
        finished_count: usize,
        whole_count: usize,
        ideal_whole_count: usize,
        completely_correct_count: usize,
        wrong_count: usize,
        targets_per_lap: Option<NonZeroUsize>,
        lap_end_time: Option<Vec<Duration>>,
    ) -> Self {
        assert_eq!(targets_per_lap.is_some(), lap_end_time.is_some());

        Self {
            finished_count,
            whole_count,
            ideal_whole_count,
            completely_correct_count,
            wrong_count,
            targets_per_lap,
            lap_end_time,
        }
    }

    fn on_ideal_target_add(&mut self, delta: usize) {
        self.ideal_whole_count += delta;
    }
}

impl OnTypingStatistics for OnTypingStatisticsDynamicTarget {
    fn on_finished(
        &mut self,
        delta: usize,
        completely_correct: bool,
        elapsed_time: Duration,
    ) -> bool {
        let lap_finish_num = if let Some(tpl) = &self.targets_per_lap {
            ((self.finished_count + delta) / tpl.get()) - (self.finished_count / tpl.get())
        } else {
            0
        };

        if lap_finish_num != 0 {
            for _ in 0..lap_finish_num {
                self.lap_end_time.as_mut().unwrap().push(elapsed_time);
            }
        }

        self.finished_count += delta;

        if completely_correct {
            self.completely_correct_count += delta;
        }

        lap_finish_num != 0
    }

    fn on_target_add(&mut self, delta: usize) {
        self.whole_count += delta;
    }

    fn on_wrong(&mut self, delta: usize) {
        self.wrong_count += delta;
    }
}

pub(crate) enum LapRequest {
    KeyStroke(NonZeroUsize),
    Spell(NonZeroUsize),
    Chunk(NonZeroUsize),
}

/// タイピング中の各対象の統計情報を管理する
pub(crate) struct OnTypingStatisticsManager {
    key_stroke: OnTypingStatisticsDynamicTarget,
    spell: OnTypingStatisticsStaticTarget,
    chunk: OnTypingStatisticsStaticTarget,
    this_key_stroke_wrong: bool,
    this_spell_wrong: bool,
    this_chunk_wrong: bool,
    last_key_stroke_elapsed_time: Option<Duration>,
}

impl OnTypingStatisticsManager {
    pub(crate) fn new(lap_request: LapRequest) -> Self {
        let mut key_stroke_targets_per_lap: Option<NonZeroUsize> = None;
        let mut spell_targets_per_lap: Option<NonZeroUsize> = None;
        let mut chunk_targets_per_lap: Option<NonZeroUsize> = None;

        let mut key_stroke_lap_end_time: Option<Vec<Duration>> = None;
        let mut spell_lap_end_time: Option<Vec<Duration>> = None;
        let mut chunk_lap_end_time: Option<Vec<Duration>> = None;

        match lap_request {
            LapRequest::KeyStroke(tpl) => {
                key_stroke_targets_per_lap.replace(tpl);
                key_stroke_lap_end_time.replace(vec![]);
            }
            LapRequest::Spell(tpl) => {
                spell_targets_per_lap.replace(tpl);
                spell_lap_end_time.replace(vec![]);
            }
            LapRequest::Chunk(tpl) => {
                chunk_targets_per_lap.replace(tpl);
                chunk_lap_end_time.replace(vec![]);
            }
        }

        Self {
            key_stroke: OnTypingStatisticsDynamicTarget::new(
                0,
                0,
                0,
                0,
                0,
                key_stroke_targets_per_lap,
                key_stroke_lap_end_time,
            ),
            spell: OnTypingStatisticsStaticTarget::new(
                0,
                0,
                0,
                0,
                spell_targets_per_lap,
                spell_lap_end_time,
            ),
            chunk: OnTypingStatisticsStaticTarget::new(
                0,
                0,
                0,
                0,
                chunk_targets_per_lap,
                chunk_lap_end_time,
            ),
            this_key_stroke_wrong: false,
            this_spell_wrong: false,
            this_chunk_wrong: false,
            last_key_stroke_elapsed_time: None,
        }
    }

    /// 実際のキーストロークをしたときに呼ぶ
    pub(crate) fn on_actual_key_stroke(
        &mut self,
        is_correct: bool,
        spell_count: usize,
        elapsed_time: Duration,
    ) -> bool {
        let mut is_lap_end = false;

        if is_correct {
            is_lap_end = self
                .key_stroke
                .on_finished(1, !self.this_key_stroke_wrong, elapsed_time);
        } else {
            self.key_stroke.on_wrong(1);
            self.spell.on_wrong(spell_count);
            self.chunk.on_wrong(1);

            self.this_spell_wrong = true;
            self.this_chunk_wrong = true;
        }

        self.this_key_stroke_wrong = !is_correct;
        self.last_key_stroke_elapsed_time.replace(elapsed_time);

        is_lap_end
    }

    /// 綴りを打ち終えたときに呼ぶ
    pub(crate) fn finish_spell(&mut self, spell_count: usize) -> bool {
        let is_lap_end = self.spell.on_finished(
            spell_count,
            !self.this_spell_wrong,
            self.last_key_stroke_elapsed_time.unwrap(),
        );
        self.this_spell_wrong = false;

        is_lap_end
    }

    /// チャンクを打ち終えたときに呼ぶ
    pub(crate) fn finish_chunk(
        &mut self,
        key_stroke_whole_count: usize,
        key_stroke_ideal_whole_count: usize,
        spell_count: usize,
    ) -> bool {
        self.key_stroke.on_target_add(key_stroke_whole_count);
        self.key_stroke
            .on_ideal_target_add(key_stroke_ideal_whole_count);
        self.spell.on_target_add(spell_count);
        self.chunk.on_finished(
            1,
            !self.this_chunk_wrong,
            self.last_key_stroke_elapsed_time.unwrap(),
        )
    }

    /// 打ち終えていないチャンクをカウントする時に呼ぶ
    pub(crate) fn add_unfinished_chunk(
        &mut self,
        key_stroke_whole_count: usize,
        key_stroke_ideal_whole_count: usize,
        spell_count: usize,
    ) {
        self.key_stroke.on_target_add(key_stroke_whole_count);
        self.key_stroke
            .on_ideal_target_add(key_stroke_ideal_whole_count);
        self.spell.on_target_add(spell_count);
        self.chunk.on_target_add(1);
    }

    pub(crate) fn emit(
        self,
    ) -> (
        OnTypingStatisticsDynamicTarget,
        OnTypingStatisticsStaticTarget,
        OnTypingStatisticsStaticTarget,
    ) {
        (self.key_stroke, self.spell, self.chunk)
    }
}
