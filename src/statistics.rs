use serde::{Deserialize, Serialize};

// キーストローク・綴り・表示文字列・語彙といった対象に関するタイピング中に使われることを意図した統計情報
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OnTypingStatistics {
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

impl OnTypingStatistics {
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

    /// 対象を打ち終えたときに呼ぶ
    /// completely_correctはその対象を1回もミスタイプなしで打ち終えたかどうかを表す
    pub(crate) fn on_finished(&mut self, completely_correct: bool) {
        self.finished_count += 1;
        self.on_target_add();

        if completely_correct {
            self.completely_correct_count += 1;
        }
    }

    /// 対象を統計に加えるときに呼ぶ
    pub(crate) fn on_target_add(&mut self) {
        self.whole_count += 1;
    }

    /// 対象をミスタイプしたときに呼ぶ
    pub(crate) fn on_wrong(&mut self) {
        self.wrong_count += 1;
    }
}
