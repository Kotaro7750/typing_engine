use std::{collections::HashMap, hash::Hash, time::Duration};

use super::StatisticalEvent;
use crate::KeyStrokeChar;
use public::EntitySkillStatistics;

/// A module for public items related to skill statistics.
mod public;

struct SkillStatisticsManager {
    /// Skill statistics for a single key stroke.
    single_key_stroke: SingleKeyStrokeSkillStatistics,
}

impl SkillStatisticsManager {
    fn new() -> Self {
        Self {
            single_key_stroke: SingleKeyStrokeSkillStatistics::new(),
        }
    }

    /// Consume the event and update statistics.
    pub(crate) fn consume_event(&mut self, event: &StatisticalEvent) {
        unimplemented!()
    }
}

/// A struct representing skill statistics for a single key stroke.
struct SingleKeyStrokeSkillStatistics {
    /// Map of [`PrimitiveSkillStatistics`](PrimitiveSkillStatistics) for each key stroke
    /// character.
    statistics_per_key_stroke_char: HashMap<KeyStrokeChar, PrimitiveSkillStatistics<KeyStrokeChar>>,
}

impl SingleKeyStrokeSkillStatistics {
    fn new() -> Self {
        Self {
            statistics_per_key_stroke_char: HashMap::new(),
        }
    }

    /// Returns a vector of [`SkillStatistics`](SkillStatistics) for each [`KeyStrokeChar`](KeyStrokeChar).
    fn skill_statisticses(&self) -> Vec<EntitySkillStatistics<KeyStrokeChar>> {
        self.statistics_per_key_stroke_char
            .iter()
            .map(|(k, v)| v.clone().into_with_entity(k.clone()))
            .collect()
    }

    /// Update statistics using the given key stroke.
    fn update(
        &mut self,
        key_stroke: &KeyStrokeChar,
        required_time: Duration,
        wrong_key_strokes: &[KeyStrokeChar],
    ) {
        self.statistics_per_key_stroke_char
            .entry(key_stroke.clone())
            .or_insert(PrimitiveSkillStatistics::new())
            .update(required_time, wrong_key_strokes);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A struct representing skill statistics for a single entity.
struct PrimitiveSkillStatistics<T: Eq + Hash + Clone> {
    /// Count of occurrences of entity.
    count: usize,
    /// Cumulative required time of all occurrences.
    cumulative_time: Duration,
    /// Map of wrong occurrences count for other entity.
    wrong_count_map: HashMap<T, usize>,
    /// Count of occurrences without wrong.
    completely_correct_count: usize,
}

impl<T: Eq + Hash + Clone> PrimitiveSkillStatistics<T> {
    fn new() -> Self {
        Self {
            count: 0,
            completely_correct_count: 0,
            cumulative_time: Duration::from_secs(0),
            wrong_count_map: HashMap::new(),
        }
    }

    /// Udate statistics by using required time for finishing entity and wrong entities.
    fn update(&mut self, required_time: Duration, wrong_entities: &[T]) {
        self.count += 1;
        self.cumulative_time += required_time;

        if wrong_entities.is_empty() {
            self.completely_correct_count += 1;
        } else {
            wrong_entities.iter().for_each(|entity| {
                *self.wrong_count_map.entry(entity.clone()).or_insert(0) += 1;
            });
        }
    }

    /// Consume self and return [`SkillStatistics`](SkillStatistics) for the given entity.
    fn into_with_entity(self, entity: T) -> EntitySkillStatistics<T> {
        EntitySkillStatistics {
            entity,
            count: self.count,
            cumulative_time: self.cumulative_time,
            wrong_count_map: self.wrong_count_map,
            completely_correct_count: self.completely_correct_count,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn empty_primitive_skill_statistics() {
        let pss = PrimitiveSkillStatistics::<KeyStrokeChar>::new();

        assert_eq!(
            pss.into_with_entity('a'.try_into().unwrap()),
            EntitySkillStatistics::<KeyStrokeChar> {
                entity: 'a'.try_into().unwrap(),
                count: 0,
                cumulative_time: Duration::from_secs(0),
                wrong_count_map: HashMap::new(),
                completely_correct_count: 0,
            }
        );
    }

    #[test]
    fn update_primitive_skill_statistics_once_without_wrong() {
        let mut pss = PrimitiveSkillStatistics::<KeyStrokeChar>::new();
        let required_time = Duration::from_secs(1);
        let wrong_entities = vec![];

        pss.update(required_time, &wrong_entities);

        assert_eq!(
            pss.into_with_entity('a'.try_into().unwrap()),
            EntitySkillStatistics::<KeyStrokeChar> {
                entity: 'a'.try_into().unwrap(),
                count: 1,
                cumulative_time: Duration::from_secs(1),
                wrong_count_map: HashMap::new(),
                completely_correct_count: 1,
            }
        );
    }

    #[test]
    fn update_primitive_skill_statistics_once_with_wrong() {
        let mut pss = PrimitiveSkillStatistics::<KeyStrokeChar>::new();
        let required_time = Duration::from_secs(1);
        let wrong_entities = vec![
            'a'.try_into().unwrap(),
            'b'.try_into().unwrap(),
            'a'.try_into().unwrap(),
        ];

        pss.update(required_time, &wrong_entities);

        assert_eq!(
            pss.into_with_entity('a'.try_into().unwrap()),
            EntitySkillStatistics::<KeyStrokeChar> {
                entity: 'a'.try_into().unwrap(),
                count: 1,
                cumulative_time: Duration::from_secs(1),
                wrong_count_map: HashMap::from([
                    ('a'.try_into().unwrap(), 2),
                    ('b'.try_into().unwrap(), 1),
                ]),
                completely_correct_count: 0,
            }
        );
    }

    #[test]
    fn update_primitive_skill_statistics_multiple_times() {
        let mut pss = PrimitiveSkillStatistics::<KeyStrokeChar>::new();

        pss.update(
            Duration::from_secs(1),
            &vec!['a'.try_into().unwrap(), 'b'.try_into().unwrap()],
        );
        pss.update(Duration::from_secs(2), &vec!['a'.try_into().unwrap()]);
        pss.update(Duration::from_secs(3), &vec![]);

        assert_eq!(
            pss.into_with_entity('a'.try_into().unwrap()),
            EntitySkillStatistics::<KeyStrokeChar> {
                entity: 'a'.try_into().unwrap(),
                count: 3,
                cumulative_time: Duration::from_secs(6),
                wrong_count_map: HashMap::from([
                    ('a'.try_into().unwrap(), 2),
                    ('b'.try_into().unwrap(), 1),
                ]),
                completely_correct_count: 1,
            }
        );
    }
}
