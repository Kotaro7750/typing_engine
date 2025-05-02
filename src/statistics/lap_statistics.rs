use std::{num::NonZeroUsize, time::Duration};

use serde::{Deserialize, Serialize};

use super::multi_target_position_convert::MultiTargetDeltaConverter;
use super::LapRequest;
use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;
use crate::typing_primitive_types::vocabulary::{
    corresponding_view_positions_for_spell, ViewPosition,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing lap information.
pub struct LapInfo {
    /// Each lap information.
    finished_laps: Vec<SingleFinishedLapInfo>,
    unfinished_laps: Vec<SingleUnfinishedLapInfo>,
}

impl LapInfo {
    pub(crate) fn new(
        finished_laps: Vec<SingleFinishedLapInfo>,
        unfinished_laps: Vec<SingleUnfinishedLapInfo>,
    ) -> Self {
        Self {
            finished_laps,
            unfinished_laps,
        }
    }

    /// Returns elapsed times of laps since typing started.
    pub fn elapsed_times(&self) -> Vec<Duration> {
        self.finished_laps
            .iter()
            .map(|lap| lap.elapsed_time())
            .collect()
    }

    /// Returns duration time of each lap.
    pub fn lap_times(&self) -> Vec<Duration> {
        self.elapsed_times()
            .iter()
            .scan(Duration::default(), |prev, &current| {
                let duration = current - *prev;
                *prev = current;
                Some(duration)
            })
            .collect()
    }

    /// Returns position indexes of lap ends for key stroke.
    pub fn key_stroke_lap_end_positions(&self) -> Vec<usize> {
        self.finished_laps
            .iter()
            .map(|lap: &SingleFinishedLapInfo| SingleLapHasLapEnd::key_stroke_lap_end_position(lap))
            .chain(
                self.unfinished_laps
                    .iter()
                    .map(|lap: &SingleUnfinishedLapInfo| {
                        SingleLapHasLapEnd::key_stroke_lap_end_position(lap)
                    }),
            )
            .collect()
    }

    #[allow(dead_code)] // TODO: Currently, ideal key stroke string is not constructed.
    /// Returns position indexes of lap ends for ideal key stroke.
    fn ideal_key_stroke_lap_end_positions(&self) -> Vec<usize> {
        self.finished_laps
            .iter()
            .map(|lap: &SingleFinishedLapInfo| {
                SingleLapHasLapEnd::ideal_key_stroke_lap_end_position(lap)
            })
            .chain(
                self.unfinished_laps
                    .iter()
                    .map(|lap: &SingleUnfinishedLapInfo| {
                        SingleLapHasLapEnd::ideal_key_stroke_lap_end_position(lap)
                    }),
            )
            .collect()
    }

    /// Returns position indexes of lap ends for spell.
    pub fn spell_lap_end_positions(&self) -> Vec<usize> {
        self.finished_laps
            .iter()
            .map(|lap: &SingleFinishedLapInfo| SingleLapHasLapEnd::spell_lap_end_position(lap))
            .chain(
                self.unfinished_laps
                    .iter()
                    .map(|lap: &SingleUnfinishedLapInfo| {
                        SingleLapHasLapEnd::spell_lap_end_position(lap)
                    }),
            )
            .collect()
    }

    /// Returns position indexes of lap ends for chunk.
    pub fn chunk_lap_end_positions(&self) -> Vec<usize> {
        self.finished_laps
            .iter()
            .map(|lap: &SingleFinishedLapInfo| SingleLapHasLapEnd::chunk_lap_end_position(lap))
            .chain(
                self.unfinished_laps
                    .iter()
                    .map(|lap: &SingleUnfinishedLapInfo| {
                        SingleLapHasLapEnd::chunk_lap_end_position(lap)
                    }),
            )
            .collect()
    }

    /// Returns position indexes of lap ends for view.
    pub fn view_lap_end_positions(&self) -> Vec<usize> {
        self.finished_laps
            .iter()
            .map(|lap: &SingleFinishedLapInfo| SingleLapHasLapEnd::view_lap_end_position(lap))
            .chain(
                self.unfinished_laps
                    .iter()
                    .map(|lap: &SingleUnfinishedLapInfo| {
                        SingleLapHasLapEnd::view_lap_end_position(lap)
                    }),
            )
            .collect()
    }
}

/// A trait representing common behavior of lap end position
trait SingleLapHasLapEnd {
    /// Returns position index of this lap end for key stroke.
    fn key_stroke_lap_end_position(&self) -> usize;

    /// Returns position index of this lap end for ideal key stroke.
    fn ideal_key_stroke_lap_end_position(&self) -> usize;

    /// Returns position index of this lap end for spell.
    fn spell_lap_end_position(&self) -> usize;

    /// Returns position index of this lap end for chunk.
    fn chunk_lap_end_position(&self) -> usize;

    /// Returns position index of this lap end for view.
    fn view_lap_end_position(&self) -> usize;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing single finished lap information.
pub(crate) struct SingleFinishedLapInfo {
    /// Elapsed time of this lap end since typing started.
    elapsed_time: Duration,
    /// Position index of this lap end for key stroke.
    key_stroke_lap_end_position: usize,
    /// Position index of this lap end for ideal key stroke.
    ideal_key_stroke_lap_end_position: usize,
    /// Position index of this lap end for spell.
    spell_lap_end_position: usize,
    /// Position index of this lap end for chunk.
    chunk_lap_end_position: usize,
    /// Position index of this lap end for view.
    view_lap_end_position: usize,
}

impl SingleFinishedLapInfo {
    pub(crate) fn new(
        elapsed_time: Duration,
        key_stroke_lap_end_position: usize,
        ideal_key_stroke_lap_end_position: usize,
        spell_lap_end_position: usize,
        chunk_lap_end_position: usize,
        view_lap_end_position: usize,
    ) -> Self {
        Self {
            elapsed_time,
            key_stroke_lap_end_position,
            ideal_key_stroke_lap_end_position,
            spell_lap_end_position,
            chunk_lap_end_position,
            view_lap_end_position,
        }
    }

    /// Returns elapsed time of this lap since typing started.
    fn elapsed_time(&self) -> Duration {
        self.elapsed_time
    }
}

impl SingleLapHasLapEnd for SingleFinishedLapInfo {
    fn key_stroke_lap_end_position(&self) -> usize {
        self.key_stroke_lap_end_position
    }

    fn ideal_key_stroke_lap_end_position(&self) -> usize {
        self.ideal_key_stroke_lap_end_position
    }

    fn spell_lap_end_position(&self) -> usize {
        self.spell_lap_end_position
    }

    fn chunk_lap_end_position(&self) -> usize {
        self.chunk_lap_end_position
    }

    fn view_lap_end_position(&self) -> usize {
        self.view_lap_end_position
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing single unfinished lap information.
pub(crate) struct SingleUnfinishedLapInfo {
    /// Position index of this lap end for key stroke.
    key_stroke_lap_end_position: usize,
    /// Position index of this lap end for ideal key stroke.
    ideal_key_stroke_lap_end_position: usize,
    /// Position index of this lap end for spell.
    spell_lap_end_position: usize,
    /// Position index of this lap end for chunk.
    chunk_lap_end_position: usize,
    /// Position index of this lap end for view.
    view_lap_end_position: usize,
}

impl SingleUnfinishedLapInfo {
    pub(crate) fn new(
        key_stroke_lap_end_position: usize,
        ideal_key_stroke_lap_end_position: usize,
        spell_lap_end_position: usize,
        chunk_lap_end_position: usize,
        view_lap_end_position: usize,
    ) -> Self {
        Self {
            key_stroke_lap_end_position,
            ideal_key_stroke_lap_end_position,
            spell_lap_end_position,
            chunk_lap_end_position,
            view_lap_end_position,
        }
    }
}

impl SingleLapHasLapEnd for SingleUnfinishedLapInfo {
    fn key_stroke_lap_end_position(&self) -> usize {
        self.key_stroke_lap_end_position
    }

    fn ideal_key_stroke_lap_end_position(&self) -> usize {
        self.ideal_key_stroke_lap_end_position
    }

    fn spell_lap_end_position(&self) -> usize {
        self.spell_lap_end_position
    }

    fn chunk_lap_end_position(&self) -> usize {
        self.chunk_lap_end_position
    }

    fn view_lap_end_position(&self) -> usize {
        self.view_lap_end_position
    }
}

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
    pub(self) fn whole_count(&self) -> usize {
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

    /// Construct [`LapInfo`](LapInfo)
    pub(crate) fn construct_lap_info(&self, view_position_of_spell: &[ViewPosition]) -> LapInfo {
        let lap_end_times = match self.lap_request {
            LapRequest::KeyStroke(_) => self.key_stroke.lap_end_time().unwrap().clone(),
            LapRequest::IdealKeyStroke(_) => self.ideal_key_stroke.lap_end_time().unwrap().clone(),
            LapRequest::Spell(_) => self.spell.lap_end_time().unwrap().clone(),
            LapRequest::Chunk(_) => self.chunk.lap_end_time().unwrap().clone(),
        };

        let key_stroke_positions = self.key_stroke.lap_end_positions();
        let ideal_key_stroke_positions = self.ideal_key_stroke.lap_end_positions();
        let spell_positions = self.spell.lap_end_positions();
        let chunk_positions = self.chunk.lap_end_positions();
        let view_positions =
            &corresponding_view_positions_for_spell(spell_positions, view_position_of_spell);

        // TODO This assertion should be satisfied at compile time by using type system.
        assert!(key_stroke_positions.len() == ideal_key_stroke_positions.len());
        assert!(key_stroke_positions.len() == spell_positions.len());
        assert!(key_stroke_positions.len() == chunk_positions.len());

        assert!(key_stroke_positions.len() >= lap_end_times.len());

        let finished_laps = lap_end_times
            .iter()
            .enumerate()
            .map(|(i, &elapsed_time)| {
                let key_stroke_pos = key_stroke_positions[i];
                let ideal_key_stroke_pos = ideal_key_stroke_positions[i];
                let spell_pos = spell_positions[i];
                let chunk_pos = chunk_positions[i];
                let view_pos = view_positions[i];

                SingleFinishedLapInfo::new(
                    elapsed_time,
                    key_stroke_pos,
                    ideal_key_stroke_pos,
                    spell_pos,
                    chunk_pos,
                    view_pos,
                )
            })
            .collect();

        let unfinished_laps = (lap_end_times.len()..key_stroke_positions.len())
            .map(|i| {
                let key_stroke_pos = key_stroke_positions[i];
                let ideal_key_stroke_pos = ideal_key_stroke_positions[i];
                let spell_pos = spell_positions[i];
                let chunk_pos = chunk_positions[i];
                let view_pos = view_positions[i];

                SingleUnfinishedLapInfo::new(
                    key_stroke_pos,
                    ideal_key_stroke_pos,
                    spell_pos,
                    chunk_pos,
                    view_pos,
                )
            })
            .collect();

        LapInfo::new(finished_laps, unfinished_laps)
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;

    use super::*;

    #[test]
    fn only_requested_entity_has_lap_end_when_chunk_is_requested() {
        let lap_statistics =
            LapStatiticsBuilder::new(LapRequest::Chunk(NonZeroUsize::new(5).unwrap()));

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_time(), None);
        assert_eq!(iks.lap_end_time(), None);
        assert_eq!(s.lap_end_time(), None);
        assert_eq!(c.lap_end_time(), Some(&vec![]));
    }

    #[test]
    fn only_requested_entity_has_lap_end_when_spell_is_requested() {
        let lap_statistics =
            LapStatiticsBuilder::new(LapRequest::Spell(NonZeroUsize::new(5).unwrap()));

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_time(), None);
        assert_eq!(iks.lap_end_time(), None);
        assert_eq!(s.lap_end_time(), Some(&vec![]));
        assert_eq!(c.lap_end_time(), None);
    }

    #[test]
    fn only_requested_entity_has_lap_end_when_ideal_key_stroke_is_requested() {
        let lap_statistics =
            LapStatiticsBuilder::new(LapRequest::IdealKeyStroke(NonZeroUsize::new(5).unwrap()));

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_time(), None);
        assert_eq!(iks.lap_end_time(), Some(&vec![]));
        assert_eq!(s.lap_end_time(), None);
        assert_eq!(c.lap_end_time(), None);
    }

    #[test]
    fn only_requested_entity_has_lap_end_when_key_stroke_is_requested() {
        let lap_statistics =
            LapStatiticsBuilder::new(LapRequest::KeyStroke(NonZeroUsize::new(5).unwrap()));

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_time(), Some(&vec![]));
        assert_eq!(iks.lap_end_time(), None);
        assert_eq!(s.lap_end_time(), None);
        assert_eq!(c.lap_end_time(), None);
    }

    #[test]
    fn lap_end_is_converted_correctly_when_chunk_is_requested() {
        let mut lap_statistics =
            LapStatiticsBuilder::new(LapRequest::Chunk(NonZeroUsize::new(2).unwrap()));

        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2]),
            KeyStrokeElementCount::new(&[2]),
            1,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_positions(), &vec![6, 16]);
        assert_eq!(iks.lap_end_positions(), &vec![4, 10]);
        assert_eq!(s.lap_end_positions(), &vec![2, 6]);
        assert_eq!(c.lap_end_positions(), &vec![1, 3]);
    }

    #[test]
    fn lap_end_is_converted_correctly_when_spell_is_requested() {
        let mut lap_statistics =
            LapStatiticsBuilder::new(LapRequest::Spell(NonZeroUsize::new(2).unwrap()));

        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2]),
            KeyStrokeElementCount::new(&[2]),
            1,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_positions(), &vec![4, 8, 13]);
        assert_eq!(iks.lap_end_positions(), &vec![2, 5, 8]);
        assert_eq!(s.lap_end_positions(), &vec![1, 3, 5]);
        assert_eq!(c.lap_end_positions(), &vec![0, 2, 3]);
    }

    #[test]
    fn lap_end_is_converted_correctly_when_ideal_key_stroke_is_requested() {
        let mut lap_statistics =
            LapStatiticsBuilder::new(LapRequest::IdealKeyStroke(NonZeroUsize::new(3).unwrap()));

        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2]),
            KeyStrokeElementCount::new(&[2]),
            1,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_positions(), &vec![4, 8, 13]);
        assert_eq!(iks.lap_end_positions(), &vec![2, 5, 8]);
        assert_eq!(s.lap_end_positions(), &vec![1, 3, 5]);
        assert_eq!(c.lap_end_positions(), &vec![0, 2, 3]);
    }

    #[test]
    fn lap_end_is_converted_correctly_when_key_stroke_is_requested() {
        let mut lap_statistics =
            LapStatiticsBuilder::new(LapRequest::KeyStroke(NonZeroUsize::new(5).unwrap()));

        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2]),
            KeyStrokeElementCount::new(&[2]),
            1,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );
        lap_statistics.on_add_chunk(
            KeyStrokeElementCount::new(&[2, 3]),
            KeyStrokeElementCount::new(&[3]),
            2,
        );

        let (ks, iks, s, c) = lap_statistics.emit();

        assert_eq!(ks.lap_end_positions(), &vec![4, 9, 14]);
        assert_eq!(iks.lap_end_positions(), &vec![2, 6, 9]);
        assert_eq!(s.lap_end_positions(), &vec![1, 4, 6]);
        assert_eq!(c.lap_end_positions(), &vec![0, 2, 3]);
    }

    fn tested_lap_info() -> LapInfo {
        LapInfo::new(
            vec![
                SingleFinishedLapInfo::new(Duration::from_secs(1), 0, 0, 0, 0, 0),
                SingleFinishedLapInfo::new(Duration::from_secs(3), 1, 2, 3, 4, 5),
                SingleFinishedLapInfo::new(Duration::from_secs(4), 2, 4, 6, 8, 10),
            ],
            vec![SingleUnfinishedLapInfo::new(3, 6, 9, 12, 15)],
        )
    }

    #[test]
    fn lap_info_construct_lap_end_correctly() {
        let lap_info = tested_lap_info();

        assert_eq!(lap_info.key_stroke_lap_end_positions(), vec![0, 1, 2, 3]);
        assert_eq!(
            lap_info.ideal_key_stroke_lap_end_positions(),
            vec![0, 2, 4, 6]
        );
        assert_eq!(lap_info.spell_lap_end_positions(), vec![0, 3, 6, 9]);
        assert_eq!(lap_info.chunk_lap_end_positions(), vec![0, 4, 8, 12]);
        assert_eq!(lap_info.view_lap_end_positions(), vec![0, 5, 10, 15]);
    }

    #[test]
    fn lap_info_construct_elapsed_time_correctly() {
        let lap_info = tested_lap_info();

        assert_eq!(
            lap_info.elapsed_times(),
            vec![
                Duration::from_secs(1),
                Duration::from_secs(3),
                Duration::from_secs(4),
            ]
        );
    }

    #[test]
    fn lap_info_construct_lap_time_correctly() {
        let lap_info = tested_lap_info();

        assert_eq!(
            lap_info.lap_times(),
            vec![
                Duration::from_secs(1),
                Duration::from_secs(2),
                Duration::from_secs(1),
            ]
        );
    }
}
