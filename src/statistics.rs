use serde::{Deserialize, Serialize};

// キーストローク・綴り・表示文字列・語彙といった対象に関するタイピング中に使われることを意図した統計情報
trait OnTypingStatistics {
    /// 対象を打ち終えた時に呼ぶ
    /// completely_correctはその対象を1回もミスタイプなしで打ち終えたかどうかを表す
    fn on_finished(&mut self, delta: usize, completely_correct: bool);

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
}

impl OnTypingStatisticsStaticTarget {
    pub(crate) fn new(
        finished_count: usize,
        whole_count: usize,
        completely_correct_count: usize,
        wrong_count: usize,
    ) -> Self {
        Self {
            finished_count,
            whole_count,
            completely_correct_count,
            wrong_count,
        }
    }
}

impl OnTypingStatistics for OnTypingStatisticsStaticTarget {
    fn on_finished(&mut self, delta: usize, completely_correct: bool) {
        self.finished_count += delta;
        //self.on_target_add(delta);

        if completely_correct {
            self.completely_correct_count += delta;
        }
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
}

impl OnTypingStatisticsDynamicTarget {
    pub(crate) fn new(
        finished_count: usize,
        whole_count: usize,
        ideal_whole_count: usize,
        completely_correct_count: usize,
        wrong_count: usize,
    ) -> Self {
        Self {
            finished_count,
            whole_count,
            ideal_whole_count,
            completely_correct_count,
            wrong_count,
        }
    }

    fn on_ideal_target_add(&mut self, delta: usize) {
        self.ideal_whole_count += delta;
    }
}

impl OnTypingStatistics for OnTypingStatisticsDynamicTarget {
    fn on_finished(&mut self, delta: usize, completely_correct: bool) {
        self.finished_count += delta;
        //self.on_target_add(delta);

        if completely_correct {
            self.completely_correct_count += delta;
        }
    }

    fn on_target_add(&mut self, delta: usize) {
        self.whole_count += delta;
    }

    fn on_wrong(&mut self, delta: usize) {
        self.wrong_count += delta;
    }
}

/// タイピング中の各対象の統計情報を管理する
pub(crate) struct OnTypingStatisticsManager {
    key_stroke: OnTypingStatisticsDynamicTarget,
    spell: OnTypingStatisticsStaticTarget,
    chunk: OnTypingStatisticsStaticTarget,
    this_key_stroke_wrong: bool,
    this_spell_wrong: bool,
    this_chunk_wrong: bool,
}

impl OnTypingStatisticsManager {
    pub(crate) fn new() -> Self {
        Self {
            key_stroke: OnTypingStatisticsDynamicTarget::new(0, 0, 0, 0, 0),
            spell: OnTypingStatisticsStaticTarget::new(0, 0, 0, 0),
            chunk: OnTypingStatisticsStaticTarget::new(0, 0, 0, 0),
            this_key_stroke_wrong: false,
            this_spell_wrong: false,
            this_chunk_wrong: false,
        }
    }

    /// 実際のキーストロークをしたときに呼ぶ
    /// TODO 将来的に経過時間を与える
    pub(crate) fn on_actual_key_stroke(&mut self, is_correct: bool, spell_count: usize) {
        if is_correct {
            self.key_stroke.on_finished(1, !self.this_key_stroke_wrong);
        } else {
            self.key_stroke.on_wrong(1);
            self.spell.on_wrong(spell_count);
            self.chunk.on_wrong(1);

            self.this_spell_wrong = true;
            self.this_chunk_wrong = true;
        }

        self.this_key_stroke_wrong = !is_correct;
    }

    /// 綴りが打ち終えたときに呼ぶ
    pub(crate) fn finish_spell(&mut self, spell_count: usize) {
        self.spell.on_finished(spell_count, !self.this_spell_wrong);
        self.this_spell_wrong = false;
    }

    /// 打ち終えていない綴りをカウントする時に呼ぶ
    pub(crate) fn add_unfinished_spell(&mut self, spell_count: usize) {
        self.spell.on_target_add(spell_count);
        self.this_spell_wrong = false;
    }

    /// チャンクが終了したときに呼び理想的な場合の候補のキーストローク数をセットする
    pub(crate) fn finish_chunk(&mut self, whole_count: usize, ideal_whole_count: usize) {
        self.key_stroke.on_target_add(whole_count);
        self.key_stroke.on_ideal_target_add(ideal_whole_count);
        self.chunk.on_finished(1, !self.this_chunk_wrong);
    }

    /// 打ち終えていないチャンクをカウントする時に呼ぶ
    pub(crate) fn add_unfinished_chunk(&mut self, whole_count: usize, ideal_whole_count: usize) {
        self.key_stroke.on_target_add(whole_count);
        self.key_stroke.on_ideal_target_add(ideal_whole_count);
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
