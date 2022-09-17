use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

mod multi_target_position_convert;

use crate::chunk::KeyStrokeElementCount;
use multi_target_position_convert::MultiTargetDeltaConverter;

use self::multi_target_position_convert::BaseTarget;

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
    // ラップ当たりの対象数
    targets_per_lap: Option<NonZeroUsize>,
    // 各ラップ末の経過時間
    lap_end_time: Option<Vec<Duration>>,
    // 各ラップ末の位置
    lap_end_position: Vec<usize>,
}

impl OnTypingStatisticsTarget {
    pub(crate) fn new(
        finished_count: usize,
        whole_count: usize,
        completely_correct_count: usize,
        wrong_count: usize,
        targets_per_lap: Option<NonZeroUsize>,
        lap_end_time: Option<Vec<Duration>>,
        lap_end_position: Vec<usize>,
    ) -> Self {
        assert_eq!(targets_per_lap.is_some(), lap_end_time.is_some());

        Self {
            finished_count,
            whole_count,
            completely_correct_count,
            wrong_count,
            targets_per_lap,
            lap_end_time,
            lap_end_position,
        }
    }

    fn whole_count(&self) -> usize {
        self.whole_count
    }

    fn on_finished(&mut self, delta: usize, completely_correct: bool, elapsed_time: Duration) {
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
    }

    /// 対象を追加する時に呼ぶ
    /// もし追加によってラップ末を追加する必要があるときにはラップ末となる位置を返す
    ///
    /// ex. 現在の対象数が2でラップあたりの対象数が5のときdeltaが8で呼ばれたケースには[3,8]が返される
    fn on_target_add(&mut self, delta: usize) -> Option<Vec<usize>> {
        let mut may_lap_end_deltas: Option<Vec<usize>> = None;

        if let Some(tpl) = self.targets_per_lap {
            let mut lap_end_deltas = vec![];

            let mut delta_to_next_lap_end = tpl.get() - (self.whole_count % tpl.get());
            while delta_to_next_lap_end <= delta {
                lap_end_deltas.push(delta_to_next_lap_end);
                delta_to_next_lap_end += tpl.get();
            }

            if !lap_end_deltas.is_empty() {
                may_lap_end_deltas.replace(lap_end_deltas);
            }
        }
        self.whole_count += delta;

        may_lap_end_deltas
    }

    fn on_wrong(&mut self, delta: usize) {
        self.wrong_count += delta;
    }

    fn add_lap_ends(&mut self, lap_end_deltas: &[usize], base_whole_count: usize) {
        lap_end_deltas.iter().for_each(|lap_end_delta| {
            self.lap_end_position
                .push(base_whole_count + lap_end_delta - 1);
        });
    }
}

pub(crate) enum LapRequest {
    KeyStroke(NonZeroUsize),
    IdealKeyStroke(NonZeroUsize),
    Spell(NonZeroUsize),
    Chunk(NonZeroUsize),
}

impl LapRequest {
    fn construct_base_target(&self) -> BaseTarget {
        match self {
            Self::KeyStroke(_) => BaseTarget::KeyStroke,
            Self::IdealKeyStroke(_) => BaseTarget::IdealKeyStroke,
            Self::Spell(_) => BaseTarget::Spell,
            Self::Chunk(_) => BaseTarget::Chunk,
        }
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
    lap_request: LapRequest,
    this_key_stroke_wrong: bool,
    this_ideal_key_stroke_wrong: bool,
    this_spell_wrong: bool,
    this_chunk_wrong: bool,
    this_candidate_key_stroke_count: Option<usize>,
    this_ideal_candidate_key_stroke_count: Option<usize>,
    in_candidate_key_stroke_count: usize,
    last_key_stroke_elapsed_time: Option<Duration>,
}

impl OnTypingStatisticsManager {
    pub(crate) fn new(lap_request: LapRequest) -> Self {
        let mut key_stroke_targets_per_lap: Option<NonZeroUsize> = None;
        let mut ideal_key_stroke_targets_per_lap: Option<NonZeroUsize> = None;
        let mut spell_targets_per_lap: Option<NonZeroUsize> = None;
        let mut chunk_targets_per_lap: Option<NonZeroUsize> = None;

        let mut key_stroke_lap_end_time: Option<Vec<Duration>> = None;
        let mut ideal_key_stroke_lap_end_time: Option<Vec<Duration>> = None;
        let mut spell_lap_end_time: Option<Vec<Duration>> = None;
        let mut chunk_lap_end_time: Option<Vec<Duration>> = None;

        match lap_request {
            LapRequest::KeyStroke(tpl) => {
                key_stroke_targets_per_lap.replace(tpl);
                key_stroke_lap_end_time.replace(vec![]);
            }
            LapRequest::IdealKeyStroke(tpl) => {
                ideal_key_stroke_targets_per_lap.replace(tpl);
                ideal_key_stroke_lap_end_time.replace(vec![]);
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
            key_stroke: OnTypingStatisticsTarget::new(
                0,
                0,
                0,
                0,
                key_stroke_targets_per_lap,
                key_stroke_lap_end_time,
                vec![],
            ),
            ideal_key_stroke: OnTypingStatisticsTarget::new(
                0,
                0,
                0,
                0,
                ideal_key_stroke_targets_per_lap,
                ideal_key_stroke_lap_end_time,
                vec![],
            ),
            spell: OnTypingStatisticsTarget::new(
                0,
                0,
                0,
                0,
                spell_targets_per_lap,
                spell_lap_end_time,
                vec![],
            ),
            chunk: OnTypingStatisticsTarget::new(
                0,
                0,
                0,
                0,
                chunk_targets_per_lap,
                chunk_lap_end_time,
                vec![],
            ),
            lap_request,
            this_key_stroke_wrong: false,
            this_ideal_key_stroke_wrong: false,
            this_spell_wrong: false,
            this_chunk_wrong: false,
            this_candidate_key_stroke_count: None,
            this_ideal_candidate_key_stroke_count: None,
            in_candidate_key_stroke_count: 0,
            last_key_stroke_elapsed_time: None,
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
    pub(crate) fn on_actual_key_stroke(
        &mut self,
        is_correct: bool,
        spell_count: usize,
        elapsed_time: Duration,
    ) {
        if is_correct {
            self.in_candidate_key_stroke_count += 1;
            self.key_stroke
                .on_finished(1, !self.this_key_stroke_wrong, elapsed_time);

            // 次打つインデックス(実際のキーストローク内インデックス)
            let in_actual_candidate_new_index = self.in_candidate_key_stroke_count;

            // 今打ったキーストロークによって理想的な候補内のインデックスが変わった場合には理想的な候補を打ち終えたとみなす
            if self.calc_ideal_key_stroke_index(in_actual_candidate_new_index - 1)
                != self.calc_ideal_key_stroke_index(in_actual_candidate_new_index)
            {
                self.ideal_key_stroke.on_finished(
                    1,
                    !self.this_ideal_key_stroke_wrong,
                    elapsed_time,
                );
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
        self.last_key_stroke_elapsed_time.replace(elapsed_time);
    }

    /// 綴りを打ち終えたときに呼ぶ
    pub(crate) fn finish_spell(&mut self, spell_count: usize) {
        self.spell.on_finished(
            spell_count,
            !self.this_spell_wrong,
            self.last_key_stroke_elapsed_time.unwrap(),
        );
        self.this_spell_wrong = false;
    }

    /// チャンクを打ち終えたときに呼ぶ
    pub(crate) fn finish_chunk(
        &mut self,
        key_stroke_element_count: KeyStrokeElementCount,
        ideal_key_stroke_element_count: KeyStrokeElementCount,
        spell_count: usize,
    ) {
        self.chunk.on_finished(
            1,
            !self.this_chunk_wrong,
            self.last_key_stroke_elapsed_time.unwrap(),
        );
        self.this_chunk_wrong = false;

        self.in_candidate_key_stroke_count = 0;

        let ks_whole_count = self.key_stroke.whole_count();
        let ksle = self
            .key_stroke
            .on_target_add(key_stroke_element_count.whole_count());

        let iks_whole_count = self.ideal_key_stroke.whole_count();
        let iksle = self
            .ideal_key_stroke
            .on_target_add(ideal_key_stroke_element_count.whole_count());

        let s_whole_count = self.spell.whole_count();
        let sle = self.spell.on_target_add(spell_count);

        let c_whole_count = self.chunk.whole_count();
        let cle = self.chunk.on_target_add(1);

        if ksle.is_some() || iksle.is_some() || sle.is_some() || cle.is_some() {
            let lap_ends = match self.lap_request {
                LapRequest::KeyStroke(_) => ksle,
                LapRequest::IdealKeyStroke(_) => iksle,
                LapRequest::Spell(_) => sle,
                LapRequest::Chunk(_) => cle,
            };

            let lap_ends = lap_ends.unwrap();

            let mdc = MultiTargetDeltaConverter::new(
                spell_count,
                ideal_key_stroke_element_count,
                key_stroke_element_count,
                self.lap_request.construct_base_target(),
            );

            self.key_stroke
                .add_lap_ends(&mdc.key_stroke_delta(&lap_ends), ks_whole_count);
            self.ideal_key_stroke
                .add_lap_ends(&mdc.ideal_key_stroke_delta(&lap_ends), iks_whole_count);
            self.spell
                .add_lap_ends(&mdc.spell_delta(&lap_ends), s_whole_count);
            self.chunk
                .add_lap_ends(&mdc.chunk_delta(&lap_ends), c_whole_count);
        }
    }

    /// 打ち終えていないチャンクをカウントする時に呼ぶ
    pub(crate) fn add_unfinished_chunk(
        &mut self,
        key_stroke_element_count: KeyStrokeElementCount,
        ideal_key_stroke_element_count: KeyStrokeElementCount,
        spell_count: usize,
    ) {
        let ks_whole_count = self.key_stroke.whole_count();
        let ksle = self
            .key_stroke
            .on_target_add(key_stroke_element_count.whole_count());

        let iks_whole_count = self.ideal_key_stroke.whole_count();
        let iksle = self
            .ideal_key_stroke
            .on_target_add(ideal_key_stroke_element_count.whole_count());

        let s_whole_count = self.spell.whole_count();
        let sle = self.spell.on_target_add(spell_count);

        let c_whole_count = self.chunk.whole_count();
        let cle = self.chunk.on_target_add(1);

        if ksle.is_some() || iksle.is_some() || sle.is_some() || cle.is_some() {
            let lap_ends = match self.lap_request {
                LapRequest::KeyStroke(_) => ksle,
                LapRequest::IdealKeyStroke(_) => iksle,
                LapRequest::Spell(_) => sle,
                LapRequest::Chunk(_) => cle,
            };

            let lap_ends = lap_ends.unwrap();

            let mdc = MultiTargetDeltaConverter::new(
                spell_count,
                ideal_key_stroke_element_count,
                key_stroke_element_count,
                self.lap_request.construct_base_target(),
            );

            self.key_stroke
                .add_lap_ends(&mdc.key_stroke_delta(&lap_ends), ks_whole_count);
            self.ideal_key_stroke
                .add_lap_ends(&mdc.ideal_key_stroke_delta(&lap_ends), iks_whole_count);
            self.spell
                .add_lap_ends(&mdc.spell_delta(&lap_ends), s_whole_count);
            self.chunk
                .add_lap_ends(&mdc.chunk_delta(&lap_ends), c_whole_count);
        }
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
