use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OnTypingStatisticsTarget {
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

impl OnTypingStatisticsTarget {
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

    fn on_finished(&mut self, delta: usize, completely_correct: bool) {
        self.finished_count += delta;

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
    // 実際のキーストローク系列に基づいた統計
    key_stroke: OnTypingStatisticsTarget,
    // 理想的なキーストローク系列に基づいた統計
    ideal_key_stroke: OnTypingStatisticsTarget,
    spell: OnTypingStatisticsTarget,
    chunk: OnTypingStatisticsTarget,
    this_key_stroke_wrong: bool,
    this_ideal_key_stroke_wrong: bool,
    this_spell_wrong: bool,
    this_chunk_wrong: bool,
    this_candidate_key_stroke_count: Option<usize>,
    this_ideal_candidate_key_stroke_count: Option<usize>,
    in_candidate_key_stroke_count: usize,
}

impl OnTypingStatisticsManager {
    pub(crate) fn new() -> Self {
        Self {
            key_stroke: OnTypingStatisticsTarget::new(0, 0, 0, 0),
            ideal_key_stroke: OnTypingStatisticsTarget::new(0, 0, 0, 0),
            spell: OnTypingStatisticsTarget::new(0, 0, 0, 0),
            chunk: OnTypingStatisticsTarget::new(0, 0, 0, 0),
            this_key_stroke_wrong: false,
            this_ideal_key_stroke_wrong: false,
            this_spell_wrong: false,
            this_chunk_wrong: false,
            this_candidate_key_stroke_count: None,
            this_ideal_candidate_key_stroke_count: None,
            in_candidate_key_stroke_count: 0,
        }
    }

    /// 理想的な候補と実際にタイプする候補の対応を取るために各チャンクのキーストローク数をセットする
    pub(crate) fn set_this_candidate_key_stroke_count(
        &mut self,
        candidate_key_stroke_count: usize,
        ideal_candidate_key_stroke_count: usize,
    ) {
        self.this_candidate_key_stroke_count
            .replace(candidate_key_stroke_count);
        self.this_ideal_candidate_key_stroke_count
            .replace(ideal_candidate_key_stroke_count);
    }

    /// 現在セットされたチャンクのキーストローク数を元に実際のキーストローク内のインデックスが理想的な候補内のどのインデックスに対応するかを計算する
    ///
    /// ex. 実際のキーストロークが「kixyo」で理想的なキーストロークが「kyo」だったとき
    /// 実際の1キーストロークは理想的なキーストロークに換算すると3/5キーストロークである
    /// そこでnキーストローク打ったときにはceil(n * 3/5)キーストローク打ったことにする
    fn calc_ideal_key_stroke_index(&self, actual_key_stroke_index: usize) -> usize {
        let ideal_count = self.this_ideal_candidate_key_stroke_count.unwrap();
        let actual_count = self.this_candidate_key_stroke_count.unwrap();

        // ceil(a/b)は (a+b-1)/b とできる
        (((actual_key_stroke_index + 1) * ideal_count) + actual_count - 1) / actual_count - 1
    }

    /// 実際のキーストロークをしたときに呼ぶ
    /// TODO 将来的に経過時間を与える
    pub(crate) fn on_actual_key_stroke(&mut self, is_correct: bool, spell_count: usize) {
        if is_correct {
            self.in_candidate_key_stroke_count += 1;
            self.key_stroke.on_finished(1, !self.this_key_stroke_wrong);

            // 次打つインデックス(実際のキーストローク内インデックス)
            let in_actual_candidate_new_index = self.in_candidate_key_stroke_count;

            // 今打ったキーストロークによって理想的な候補内のインデックスが変わった場合には理想的な候補を打ち終えたとみなす
            if self.calc_ideal_key_stroke_index(in_actual_candidate_new_index - 1)
                != self.calc_ideal_key_stroke_index(in_actual_candidate_new_index)
            {
                self.ideal_key_stroke
                    .on_finished(1, !self.this_ideal_key_stroke_wrong);
                self.this_ideal_key_stroke_wrong = false;
            }
        } else {
            self.key_stroke.on_wrong(1);
            self.ideal_key_stroke.on_wrong(1);
            self.spell.on_wrong(spell_count);
            self.chunk.on_wrong(1);

            self.this_ideal_key_stroke_wrong = true;
            self.this_spell_wrong = true;
            self.this_chunk_wrong = true;
        }

        self.this_key_stroke_wrong = !is_correct;
    }

    /// 綴りを打ち終えたときに呼ぶ
    pub(crate) fn finish_spell(&mut self, spell_count: usize) {
        self.spell.on_finished(spell_count, !self.this_spell_wrong);
        self.this_spell_wrong = false;
    }

    /// チャンクを打ち終えたときに呼ぶ
    pub(crate) fn finish_chunk(
        &mut self,
        key_stroke_whole_count: usize,
        key_stroke_ideal_whole_count: usize,
        spell_count: usize,
    ) {
        self.in_candidate_key_stroke_count = 0;
        self.key_stroke.on_target_add(key_stroke_whole_count);
        self.ideal_key_stroke
            .on_target_add(key_stroke_ideal_whole_count);
        self.spell.on_target_add(spell_count);
        self.chunk.on_finished(1, !self.this_chunk_wrong);
    }

    /// 打ち終えていないチャンクをカウントする時に呼ぶ
    pub(crate) fn add_unfinished_chunk(
        &mut self,
        key_stroke_whole_count: usize,
        key_stroke_ideal_whole_count: usize,
        spell_count: usize,
    ) {
        self.key_stroke.on_target_add(key_stroke_whole_count);
        self.ideal_key_stroke
            .on_target_add(key_stroke_ideal_whole_count);
        self.spell.on_target_add(spell_count);
        self.chunk.on_target_add(1);
    }

    pub(crate) fn emit(
        self,
    ) -> (
        OnTypingStatisticsTarget,
        OnTypingStatisticsTarget,
        OnTypingStatisticsTarget,
        OnTypingStatisticsTarget,
    ) {
        (
            self.key_stroke,
            self.ideal_key_stroke,
            self.spell,
            self.chunk,
        )
    }
}
