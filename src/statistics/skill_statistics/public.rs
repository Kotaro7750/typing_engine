use std::collections::HashMap;
use std::hash::Hash;
use std::time::Duration;

use crate::KeyStrokeChar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillStatistics {
    pub single_key_stroke: Vec<EntitySkillStatistics<KeyStrokeChar>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A struct representing skill statistics for a single entity.
/// Entity is like a [`KeyStrokeChar`](KeyStrokeChar).
pub struct EntitySkillStatistics<T: Eq + Hash + Clone> {
    /// The entity for which the statistics are taken.
    pub entity: T,
    /// Count of occurrences of entity.
    pub count: usize,
    /// Cumulative required time of all occurrences.
    pub cumulative_time: Duration,
    /// Map of wrong occurrences count for other entity.
    pub wrong_count_map: HashMap<T, usize>,
    /// Count of occurrences without wrong.
    pub completely_correct_count: usize,
}

impl<T: Eq + Hash + Clone> EntitySkillStatistics<T> {
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
            wrong_count_map: HashMap::new(),
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
            wrong_count_map: HashMap::new(),
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
            wrong_count_map: HashMap::new(),
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
            wrong_count_map: HashMap::from([
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
