use serde::{Deserialize, Serialize};

use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing statistics counter for each primitive type entities.
pub struct PrimitiveStatisticsCounter {
    /// Count of finished entities.
    finished_count: usize,
    /// Count of whole entities.
    whole_count: usize,
    /// Count of entities that are finished without any miss.
    completely_correct_count: usize,
    /// Count of entities that are wrong typed regardless of duplication.
    /// If a target is wrong typed multiple times, each mistype is counted.
    wrong_count: usize,
}

impl PrimitiveStatisticsCounter {
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

    /// Create an empty counter.
    fn empty_counter() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Get count of finished entities.
    pub(super) fn finished_count(&self) -> usize {
        self.finished_count
    }

    /// Get count of whole entities.
    pub(super) fn whole_count(&self) -> usize {
        self.whole_count
    }

    /// Get count of entities that are finished without any miss.
    pub(super) fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }

    /// Get count of entities that are wrong typed regardless of duplication.
    /// If a target is wrong typed multiple times, each mistype is counted.
    pub(super) fn wrong_count(&self) -> usize {
        self.wrong_count
    }

    /// Update statistics when entities are took into account.
    fn on_target_add(&mut self, delta: usize) {
        self.whole_count += delta;
    }

    /// Update statistics when entities are finished.
    fn on_finished(&mut self, delta: usize, completely_correct: bool) {
        self.finished_count += delta;

        if completely_correct {
            self.completely_correct_count += delta;
        }
    }

    /// Update statistics when entities are wrong typed.
    fn on_wrong(&mut self, delta: usize) {
        self.wrong_count += delta;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing statistics counter.
pub(crate) struct StatisticsCounter {
    key_stroke: PrimitiveStatisticsCounter,
    ideal_key_stroke: PrimitiveStatisticsCounter,
    spell: PrimitiveStatisticsCounter,
    chunk: PrimitiveStatisticsCounter,
    /// Below fields are for teporary state flags.
    this_key_stroke_wrong: bool,
    this_ideal_key_stroke_wrong: bool,
    this_spell_wrong: bool,
    this_chunk_wrong: bool,
    this_candidate_key_stroke_count: Option<usize>,
    this_ideal_candidate_key_stroke_count: Option<usize>,
    in_candidate_key_stroke_count: usize,
}

impl StatisticsCounter {
    pub(crate) fn new() -> Self {
        Self {
            key_stroke: PrimitiveStatisticsCounter::empty_counter(),
            ideal_key_stroke: PrimitiveStatisticsCounter::empty_counter(),
            spell: PrimitiveStatisticsCounter::empty_counter(),
            chunk: PrimitiveStatisticsCounter::empty_counter(),
            this_key_stroke_wrong: false,
            this_ideal_key_stroke_wrong: false,
            this_spell_wrong: false,
            this_chunk_wrong: false,
            this_candidate_key_stroke_count: None,
            this_ideal_candidate_key_stroke_count: None,
            in_candidate_key_stroke_count: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_with_values(
        key_stroke: PrimitiveStatisticsCounter,
        ideal_key_stroke: PrimitiveStatisticsCounter,
        spell: PrimitiveStatisticsCounter,
        chunk: PrimitiveStatisticsCounter,
        this_key_stroke_wrong: bool,
        this_ideal_key_stroke_wrong: bool,
        this_spell_wrong: bool,
        this_chunk_wrong: bool,
        this_candidate_key_stroke_count: Option<usize>,
        this_ideal_candidate_key_stroke_count: Option<usize>,
        in_candidate_key_stroke_count: usize,
    ) -> Self {
        Self {
            key_stroke,
            ideal_key_stroke,
            spell,
            chunk,
            this_key_stroke_wrong,
            this_ideal_key_stroke_wrong,
            this_spell_wrong,
            this_chunk_wrong,
            this_candidate_key_stroke_count,
            this_ideal_candidate_key_stroke_count,
            in_candidate_key_stroke_count,
        }
    }

    /// Update statistics when a chunk is took into account.
    pub(crate) fn on_add_chunk(
        &mut self,
        key_stroke_element_count: KeyStrokeElementCount,
        ideal_key_stroke_element_count: KeyStrokeElementCount,
        spell_count: usize,
    ) {
        self.key_stroke
            .on_target_add(key_stroke_element_count.whole_count());
        self.ideal_key_stroke
            .on_target_add(ideal_key_stroke_element_count.whole_count());
        self.spell.on_target_add(spell_count);
        self.chunk.on_target_add(1);
    }

    /// Prepare when a chunk is started.
    /// This is due to converting actual key stroke index to ideal key stroke index.
    pub(crate) fn on_start_chunk(
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

    /// Update statistics when a key is stroked.
    pub(crate) fn on_stroke_key(&mut self, is_correct: bool, spell_count: usize) {
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

    /// Update statistics when spell is finished.
    pub(crate) fn on_finish_spell(&mut self, spell_count: usize) {
        self.spell.on_finished(spell_count, !self.this_spell_wrong);
        self.this_spell_wrong = false;
    }

    /// Update statistics when chunk is finished.
    pub(crate) fn on_finish_chunk(&mut self) {
        self.chunk.on_finished(1, !self.this_chunk_wrong);
        self.this_chunk_wrong = false;

        self.this_candidate_key_stroke_count = None;
        self.this_ideal_candidate_key_stroke_count = None;
        self.in_candidate_key_stroke_count = 0;
    }

    pub(crate) fn emit(
        self,
    ) -> (
        PrimitiveStatisticsCounter,
        PrimitiveStatisticsCounter,
        PrimitiveStatisticsCounter,
        PrimitiveStatisticsCounter,
    ) {
        (
            self.key_stroke,
            self.ideal_key_stroke,
            self.spell,
            self.chunk,
        )
    }
}
