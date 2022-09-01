use serde::{Deserialize, Serialize};

// キーストローク・綴り・表示文字列・語彙といった対象に関するタイピング中に使われることを意図した統計情報
pub(crate) trait OnTypingStatistics {
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
        self.on_target_add(delta);

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

    pub(crate) fn on_target_add_not_ideal_only(&mut self, delta: usize) {
        self.whole_count += delta;
    }
}

impl OnTypingStatistics for OnTypingStatisticsDynamicTarget {
    fn on_finished(&mut self, delta: usize, completely_correct: bool) {
        self.finished_count += delta;
        self.on_target_add(delta);

        if completely_correct {
            self.completely_correct_count += delta;
        }
    }

    fn on_target_add(&mut self, delta: usize) {
        self.on_target_add_not_ideal_only(delta);
        self.ideal_whole_count += delta;
    }

    fn on_wrong(&mut self, delta: usize) {
        self.wrong_count += delta;
    }
}
