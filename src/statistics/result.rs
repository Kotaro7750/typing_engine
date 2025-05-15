use std::{ops::Add, time::Duration};

use serde::{Deserialize, Serialize};

use super::{
    skill_statistics::public::SkillStatistics, statistics_counter::EntitySummaryStatistics,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing result of typing.
///
/// Because this struct implements `std::ops::Add` trait, you can add two
/// [`TypingResult`](TypingResult) and get an aggregated result of multiple typing try.
pub struct TypingResult {
    /// Duration for whole typing
    total_time: Duration,
    /// Aggregated result of typing
    summary: TypingResultSummary,
    /// Detailed statistics representing typing skill
    skill_statistics: SkillStatistics,
}

impl TypingResult {
    pub(crate) fn new(
        total_time: Duration,
        summary: TypingResultSummary,
        skill_statistics: SkillStatistics,
    ) -> Self {
        Self {
            total_time,
            summary,
            skill_statistics,
        }
    }

    /// Returns duration for whole typing
    pub fn total_time(&self) -> Duration {
        self.total_time
    }

    /// Returns aggregated result of typing
    pub fn summary(&self) -> &TypingResultSummary {
        &self.summary
    }

    /// Returns detailed statistics representing typing skill
    pub fn skill_statistics(&self) -> &SkillStatistics {
        &self.skill_statistics
    }
}

impl Add for TypingResult {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let total_time = self.total_time + rhs.total_time;
        let summary = self.summary + rhs.summary;
        let skill_statistics = self.skill_statistics + rhs.skill_statistics;

        Self::new(total_time, summary, skill_statistics)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing aggregated result of typing.
/// Aggregation is for 4 entities, key stroke, ideal key stroke, spell, and chunk.
pub struct TypingResultSummary {
    /// Aggregated result for key stroke
    key_stroke: EntitySummaryStatistics,
    /// Aggregated result for ideal key stroke
    ideal_key_stroke: EntitySummaryStatistics,
    /// Aggregated result for spell
    spell: EntitySummaryStatistics,
    /// Aggregated result for chunk
    chunk: EntitySummaryStatistics,
}

impl TypingResultSummary {
    pub(crate) fn new(
        key_stroke: EntitySummaryStatistics,
        ideal_key_stroke: EntitySummaryStatistics,
        spell: EntitySummaryStatistics,
        chunk: EntitySummaryStatistics,
    ) -> Self {
        Self {
            key_stroke,
            ideal_key_stroke,
            spell,
            chunk,
        }
    }

    /// Returns aggregated result for key stroke
    pub fn key_stroke(&self) -> &EntitySummaryStatistics {
        &self.key_stroke
    }

    /// Returns aggregated result for key stroke
    pub fn ideal_key_stroke(&self) -> &EntitySummaryStatistics {
        &self.ideal_key_stroke
    }

    /// Returns aggregated result for key stroke
    pub fn spell(&self) -> &EntitySummaryStatistics {
        &self.spell
    }

    /// Returns aggregated result for key stroke
    pub fn chunk(&self) -> &EntitySummaryStatistics {
        &self.chunk
    }
}

impl Add for TypingResultSummary {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let key_stroke = self.key_stroke + rhs.key_stroke;
        let ideal_key_stroke = self.ideal_key_stroke + rhs.ideal_key_stroke;
        let spell = self.spell + rhs.spell;
        let chunk = self.chunk + rhs.chunk;

        Self::new(key_stroke, ideal_key_stroke, spell, chunk)
    }
}
