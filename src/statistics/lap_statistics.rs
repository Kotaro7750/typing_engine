use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

use super::multi_target_position_convert::MultiTargetDeltaConverter;
use super::LapRequest;
use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing lap statistics for each primitive type entities.
pub(crate) struct PrimitiveLapStatisticsBuilder {
    // ラップ当たりの対象数
    /// Count of entities per lap.
    /// This is [`None`](std::option::Option::None) when target primitive is not a target for taking laps.
    targets_per_lap: Option<NonZeroUsize>,
    /// Elapsed time of each lap end.
    /// This is [`None`](std::option::Option::None) when target primitive is not a target for taking laps.
    lap_end_time: Option<Vec<Duration>>,
    /// Position indexes of each lap end.
    lap_end_position: Vec<usize>,
    /// Count of finished entities.
    finished_count: usize,
    /// Count of whole entities.
    whole_count: usize,
}

impl PrimitiveLapStatisticsBuilder {
    fn new(
        finished_count: usize,
        whole_count: usize,
        targets_per_lap: Option<NonZeroUsize>,
        lap_end_time: Option<Vec<Duration>>,
        lap_end_position: Vec<usize>,
    ) -> Self {
        assert_eq!(targets_per_lap.is_some(), lap_end_time.is_some());

        Self {
            finished_count,
            whole_count,
            targets_per_lap,
            lap_end_time,
            lap_end_position,
        }
    }

    /// Get count of whole targets.
    pub(crate) fn whole_count(&self) -> usize {
        self.whole_count
    }

    /// Get targets count per lap.
    pub(crate) fn targets_per_lap(&self) -> Option<NonZeroUsize> {
        self.targets_per_lap
    }

    /// Get lap end time of target.
    /// This returns [`None`](std::option::Option::None) when target is not a target for take laps.
    pub(crate) fn lap_end_time(&self) -> Option<&Vec<Duration>> {
        self.lap_end_time.as_ref()
    }

    /// Get lap end position indexes of target.
    /// Each positions is converted from requested target.
    pub(crate) fn lap_end_positions(&self) -> &Vec<usize> {
        &self.lap_end_position
    }

    /// Update statistics when entities are took into account.
    /// If lap end is added by this addition, deltas to lap end is returned.
    ///
    /// ex. When current whole count is 2 and targets per lap is 5, and delta is 8, this returns
    /// [3, 8].
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

    /// Update statistics when entities are finished.
    fn on_finished(&mut self, delta: usize, elapsed_time: Duration) {
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
    }

    /// Add lap end positions manually.
    fn add_lap_ends(&mut self, lap_end_deltas: &[usize], base_whole_count: usize) {
        lap_end_deltas.iter().for_each(|lap_end_delta| {
            self.lap_end_position
                .push(base_whole_count + lap_end_delta - 1);
        });
    }
}

/// タイピング中の各対象の統計情報を管理する
pub(crate) struct LapStatiticsBuilder {
    // 実際のキーストローク系列に基づいた統計
    key_stroke: PrimitiveLapStatisticsBuilder,
    // 理想的なキーストローク系列に基づいた統計
    ideal_key_stroke: PrimitiveLapStatisticsBuilder,
    spell: PrimitiveLapStatisticsBuilder,
    chunk: PrimitiveLapStatisticsBuilder,
    lap_request: LapRequest,
    this_candidate_key_stroke_count: Option<usize>,
    this_ideal_candidate_key_stroke_count: Option<usize>,
    in_candidate_key_stroke_count: usize,
    last_key_stroke_elapsed_time: Option<Duration>,
}

impl LapStatiticsBuilder {
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
            key_stroke: PrimitiveLapStatisticsBuilder::new(
                0,
                0,
                key_stroke_targets_per_lap,
                key_stroke_lap_end_time,
                vec![],
            ),
            ideal_key_stroke: PrimitiveLapStatisticsBuilder::new(
                0,
                0,
                ideal_key_stroke_targets_per_lap,
                ideal_key_stroke_lap_end_time,
                vec![],
            ),
            spell: PrimitiveLapStatisticsBuilder::new(
                0,
                0,
                spell_targets_per_lap,
                spell_lap_end_time,
                vec![],
            ),
            chunk: PrimitiveLapStatisticsBuilder::new(
                0,
                0,
                chunk_targets_per_lap,
                chunk_lap_end_time,
                vec![],
            ),
            lap_request,
            this_candidate_key_stroke_count: None,
            this_ideal_candidate_key_stroke_count: None,
            in_candidate_key_stroke_count: 0,
            last_key_stroke_elapsed_time: None,
        }
    }

    /// Update statistics when a chunk is took into account.
    pub(crate) fn on_add_chunk(
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
    pub(crate) fn on_actual_key_stroke(&mut self, is_correct: bool, elapsed_time: Duration) {
        if is_correct {
            self.in_candidate_key_stroke_count += 1;
            self.key_stroke.on_finished(1, elapsed_time);

            // 次打つインデックス(実際のキーストローク内インデックス)
            let in_actual_candidate_new_index = self.in_candidate_key_stroke_count;

            // 今打ったキーストロークによって理想的な候補内のインデックスが変わった場合には理想的な候補を打ち終えたとみなす
            if self.calc_ideal_key_stroke_index(in_actual_candidate_new_index - 1)
                != self.calc_ideal_key_stroke_index(in_actual_candidate_new_index)
            {
                self.ideal_key_stroke.on_finished(1, elapsed_time);
            }
        }

        self.last_key_stroke_elapsed_time.replace(elapsed_time);
    }

    /// Update statistics when spell is finished.
    pub(crate) fn on_finish_spell(&mut self, spell_count: usize) {
        self.spell
            .on_finished(spell_count, self.last_key_stroke_elapsed_time.unwrap());
    }

    /// Update statistics when chunk is finished.
    pub(crate) fn on_finish_chunk(&mut self) {
        self.chunk
            .on_finished(1, self.last_key_stroke_elapsed_time.unwrap());

        self.in_candidate_key_stroke_count = 0;
    }

    pub(crate) fn emit(
        self,
    ) -> (
        PrimitiveLapStatisticsBuilder,
        PrimitiveLapStatisticsBuilder,
        PrimitiveLapStatisticsBuilder,
        PrimitiveLapStatisticsBuilder,
    ) {
        (
            self.key_stroke,
            self.ideal_key_stroke,
            self.spell,
            self.chunk,
        )
    }
}
