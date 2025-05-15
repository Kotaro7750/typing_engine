use serde::{Deserialize, Serialize};
use std::ops::Add;

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
    pub(crate) fn empty_counter() -> Self {
        Self::new(0, 0, 0, 0)
    }

    /// Get count of finished entities.
    pub(crate) fn finished_count(&self) -> usize {
        self.finished_count
    }

    /// Get count of whole entities.
    pub(crate) fn whole_count(&self) -> usize {
        self.whole_count
    }

    /// Get count of entities that are finished without any miss.
    pub(crate) fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }

    /// Get count of entities that are wrong typed regardless of duplication.
    /// If a target is wrong typed multiple times, each mistype is counted.
    pub(crate) fn wrong_count(&self) -> usize {
        self.wrong_count
    }

    /// Update statistics when entities are took into account.
    pub(crate) fn on_target_add(&mut self, delta: usize) {
        self.whole_count += delta;
    }

    /// Update statistics when entities are finished.
    pub(crate) fn on_finished(&mut self, delta: usize, completely_correct: bool) {
        self.finished_count += delta;

        if completely_correct {
            self.completely_correct_count += delta;
        }
    }

    /// Update statistics when entities are wrong typed.
    pub(crate) fn on_wrong(&mut self, delta: usize) {
        self.wrong_count += delta;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing aggregated statistics of typing for each entities.
pub struct EntitySummaryStatistics {
    /// Count how many entities are target of typing
    whole_count: usize,
    /// Count how many entities are finished typing
    finished_count: usize,
    /// Count how many entities are finished typing without wrong types
    completely_correct_count: usize,
    /// Count how many wrong entity is observed
    /// This count includes duplication, so count may be above 1 when typed wrong multiple times
    wrong_count: usize,
}

impl EntitySummaryStatistics {
    /// Returns count how many entities are target of typing
    pub fn whole_count(&self) -> usize {
        self.whole_count
    }

    /// Returns count how many entities are finished typing
    pub fn finished_count(&self) -> usize {
        self.finished_count
    }

    /// Returns progress of typing
    pub fn progress(&self) -> f64 {
        if self.whole_count == 0 {
            return 0.0;
        }
        self.finished_count as f64 / self.whole_count as f64
    }

    /// Returns count how many entities are finished typing without wrong types
    pub fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }

    /// Returns count how many wrong entity is observed
    /// This count includes duplication, so count may be above 1 when typed wrong multiple times
    pub fn wrong_count(&self) -> usize {
        self.wrong_count
    }
}

impl From<&PrimitiveStatisticsCounter> for EntitySummaryStatistics {
    fn from(value: &PrimitiveStatisticsCounter) -> Self {
        Self {
            whole_count: value.whole_count(),
            finished_count: value.finished_count(),
            completely_correct_count: value.completely_correct_count(),
            wrong_count: value.wrong_count(),
        }
    }
}

impl Add for EntitySummaryStatistics {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let whole_count = self.whole_count + rhs.whole_count;
        let finished_count = self.finished_count + rhs.finished_count;
        let completely_correct_count = self.completely_correct_count + rhs.completely_correct_count;
        let wrong_count = self.wrong_count + rhs.wrong_count;

        Self {
            whole_count,
            finished_count,
            completely_correct_count,
            wrong_count,
        }
    }
}
