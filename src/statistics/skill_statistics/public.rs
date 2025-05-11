use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::time::Duration;

use crate::KeyStrokeChar;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SkillStatistics {
    single_key_stroke: Vec<EntitySkillStatistics<KeyStrokeChar>>,
}

impl SkillStatistics {
    pub(crate) fn new_with(single_key_stroke: Vec<EntitySkillStatistics<KeyStrokeChar>>) -> Self {
        Self { single_key_stroke }
    }

    /// Returns skill statistics for a single key stroke.
    pub fn single_key_stroke(&self) -> &[EntitySkillStatistics<KeyStrokeChar>] {
        &self.single_key_stroke
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
/// A struct representing skill statistics for a single entity.
/// Entity is like a [`KeyStrokeChar`](KeyStrokeChar).
pub struct EntitySkillStatistics<T: Eq + Hash + Clone + Ord> {
    /// The entity for which the statistics are taken.
    entity: T,
    /// Count of occurrences of entity.
    count: usize,
    /// Cumulative required time of all occurrences.
    cumulative_time: Duration,
    /// Map of wrong occurrences count for other entity.
    wrong_count_map: BTreeMap<T, usize>,
    /// Count of occurrences without wrong.
    completely_correct_count: usize,
}

impl<T: Eq + Hash + Clone + Ord> EntitySkillStatistics<T> {
    pub(crate) fn new_with(
        entity: T,
        count: usize,
        cumulative_time: Duration,
        wrong_count_map: BTreeMap<T, usize>,
        completely_correct_count: usize,
    ) -> Self {
        Self {
            entity,
            count,
            cumulative_time,
            wrong_count_map,
            completely_correct_count,
        }
    }

    /// Returns entity.
    pub fn entity(&self) -> &T {
        &self.entity
    }

    /// Returns count of occurrences of entity.
    pub fn count(&self) -> usize {
        self.count
    }

    /// Returns entities and counts of wrong occurrences sorted by count.
    /// Sort order is descending, it means that the most wrong entity is first.
    /// Each element of the vector is ( entity, count of wrong occurrences).
    pub fn wrong_count_ranking(&self) -> Vec<(T, usize)> {
        let mut ranking: Vec<(T, usize)> = self
            .wrong_count_map
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        ranking.sort_by(|a, b| b.1.cmp(&a.1));
        ranking
    }

    /// Returns accuracy for typing this entity.
    /// This is the ratio of completely correct occurrences to all occurrences.
    pub fn accuracy(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.completely_correct_count as f64 / self.count as f64
        }
    }

    /// Returns average time for typing this entity.
    pub fn average_time(&self) -> Duration {
        if self.count == 0 {
            Duration::from_secs(0)
        } else {
            self.cumulative_time / self.count as u32
        }
    }

    /// Returns count of occurrences without wrong.
    pub fn completely_correct_count(&self) -> usize {
        self.completely_correct_count
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_skill_statistics() {
        let ss = EntitySkillStatistics::<KeyStrokeChar> {
            entity: 'a'.try_into().unwrap(),
            count: 0,
            cumulative_time: Duration::from_secs(0),
            wrong_count_map: BTreeMap::new(),
            completely_correct_count: 0,
        };

        assert_eq!(ss.accuracy(), 0.0);
        assert_eq!(ss.average_time(), Duration::from_secs(0));
        assert_eq!(ss.wrong_count_ranking(), vec![]);
    }

    #[test]
    fn skill_statistics_accuracy_is_calculated_correctly() {
        let ss = EntitySkillStatistics::<KeyStrokeChar> {
            entity: 'a'.try_into().unwrap(),
            count: 4,
            cumulative_time: Duration::from_secs(4),
            wrong_count_map: BTreeMap::new(),
            completely_correct_count: 1,
        };

        assert_eq!(ss.accuracy(), 0.25);
    }

    #[test]
    fn skill_statistics_averate_time_is_calculated_correctly() {
        let ss = EntitySkillStatistics::<KeyStrokeChar> {
            entity: 'a'.try_into().unwrap(),
            count: 4,
            cumulative_time: Duration::from_secs(5),
            wrong_count_map: BTreeMap::new(),
            completely_correct_count: 1,
        };

        assert_eq!(ss.average_time(), Duration::new(1, 250_000_000));
    }

    #[test]
    fn skill_statistics_wrong_count_ranking_is_constructed_correctly() {
        let ss = EntitySkillStatistics::<KeyStrokeChar> {
            entity: 'a'.try_into().unwrap(),
            count: 4,
            cumulative_time: Duration::from_secs(4),
            wrong_count_map: BTreeMap::from([
                ('a'.try_into().unwrap(), 2),
                ('b'.try_into().unwrap(), 1),
                ('c'.try_into().unwrap(), 3),
                ('!'.try_into().unwrap(), 100),
            ]),
            completely_correct_count: 1,
        };

        assert_eq!(
            ss.wrong_count_ranking(),
            vec![
                ('!'.try_into().unwrap(), 100),
                ('c'.try_into().unwrap(), 3),
                ('a'.try_into().unwrap(), 2),
                ('b'.try_into().unwrap(), 1),
            ]
        );
    }
}
