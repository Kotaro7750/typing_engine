use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::Hash;
use std::ops::Add;
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

impl Add for SkillStatistics {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        let mut result_map = BTreeMap::new();

        // Process the elements from self
        for stat in self.single_key_stroke.iter() {
            result_map.insert(stat.entity().clone(), stat.clone());
        }

        // Process the elements from rhs
        for stat in rhs.single_key_stroke.iter() {
            if let Some(existing_stat) = result_map.get_mut(stat.entity()) {
                // If the entity already exists, add the statistics
                *existing_stat = existing_stat.clone() + stat.clone();
            } else {
                // If the entity doesn't exist, insert it
                result_map.insert(stat.entity().clone(), stat.clone());
            }
        }

        Self::new_with(result_map.into_values().collect())
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

impl<T: Eq + Hash + Clone + Ord> Add for EntitySkillStatistics<T> {
    type Output = Self;

    /// Adds two [`EntitySkillStatistics`](EntitySkillStatistics) together.
    /// When add instances with different entities, the entity of the first instance is used.
    fn add(self, rhs: Self) -> Self::Output {
        let mut wrong_count_map = self.wrong_count_map.clone();
        for (k, v) in rhs.wrong_count_map.iter() {
            *wrong_count_map.entry(k.clone()).or_insert(0) += *v;
        }
        Self::new_with(
            self.entity,
            self.count + rhs.count,
            self.cumulative_time + rhs.cumulative_time,
            wrong_count_map,
            self.completely_correct_count + rhs.completely_correct_count,
        )
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
    fn add_entity_skill_statistics_with_same_entity() {
        let ss1 = EntitySkillStatistics::<KeyStrokeChar>::new_with(
            'a'.try_into().unwrap(),
            2,
            Duration::from_secs(2),
            BTreeMap::from([('b'.try_into().unwrap(), 1), ('c'.try_into().unwrap(), 3)]),
            1,
        );

        let ss2 = EntitySkillStatistics::<KeyStrokeChar>::new_with(
            'a'.try_into().unwrap(),
            3,
            Duration::from_secs(3),
            BTreeMap::from([('a'.try_into().unwrap(), 1), ('b'.try_into().unwrap(), 1)]),
            2,
        );

        let added = ss1 + ss2;

        assert_eq!(added.entity(), &KeyStrokeChar::try_from('a').unwrap());
        assert_eq!(added.count(), 5);
        assert_eq!(added.completely_correct_count(), 3);
        assert_eq!(
            added.wrong_count_ranking(),
            vec![
                (KeyStrokeChar::try_from('c').unwrap(), 3),
                (KeyStrokeChar::try_from('b').unwrap(), 2),
                (KeyStrokeChar::try_from('a').unwrap(), 1),
            ]
        );
    }

    #[test]
    fn add_entity_skill_statistics_with_different_entity() {
        let ss1 = EntitySkillStatistics::<KeyStrokeChar>::new_with(
            'a'.try_into().unwrap(),
            2,
            Duration::from_secs(2),
            BTreeMap::from([('b'.try_into().unwrap(), 1), ('c'.try_into().unwrap(), 3)]),
            1,
        );

        let ss2 = EntitySkillStatistics::<KeyStrokeChar>::new_with(
            'b'.try_into().unwrap(),
            3,
            Duration::from_secs(3),
            BTreeMap::from([('a'.try_into().unwrap(), 1), ('b'.try_into().unwrap(), 1)]),
            2,
        );

        let added = ss1 + ss2;

        assert_eq!(added.entity(), &KeyStrokeChar::try_from('a').unwrap());
        assert_eq!(added.count(), 5);
        assert_eq!(added.completely_correct_count(), 3);
        assert_eq!(
            added.wrong_count_ranking(),
            vec![
                (KeyStrokeChar::try_from('c').unwrap(), 3),
                (KeyStrokeChar::try_from('b').unwrap(), 2),
                (KeyStrokeChar::try_from('a').unwrap(), 1),
            ]
        );
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

    #[test]
    fn add_skill_statistics() {
        let ss1 = SkillStatistics::new_with(vec![
            EntitySkillStatistics::<KeyStrokeChar>::new_with(
                'a'.try_into().unwrap(),
                2,
                Duration::from_secs(2),
                BTreeMap::new(),
                1,
            ),
            EntitySkillStatistics::<KeyStrokeChar>::new_with(
                'b'.try_into().unwrap(),
                3,
                Duration::from_secs(3),
                BTreeMap::new(),
                2,
            ),
        ]);
        let ss2 = SkillStatistics::new_with(vec![
            EntitySkillStatistics::<KeyStrokeChar>::new_with(
                'c'.try_into().unwrap(),
                5,
                Duration::from_secs(5),
                BTreeMap::new(),
                4,
            ),
            EntitySkillStatistics::<KeyStrokeChar>::new_with(
                'a'.try_into().unwrap(),
                4,
                Duration::from_secs(4),
                BTreeMap::new(),
                3,
            ),
        ]);

        let added = ss1 + ss2;

        let expected = SkillStatistics::new_with(vec![
            EntitySkillStatistics::<KeyStrokeChar>::new_with(
                'a'.try_into().unwrap(),
                6,
                Duration::from_secs(6),
                BTreeMap::new(),
                4,
            ),
            EntitySkillStatistics::<KeyStrokeChar>::new_with(
                'b'.try_into().unwrap(),
                3,
                Duration::from_secs(3),
                BTreeMap::new(),
                2,
            ),
            EntitySkillStatistics::<KeyStrokeChar>::new_with(
                'c'.try_into().unwrap(),
                5,
                Duration::from_secs(5),
                BTreeMap::new(),
                4,
            ),
        ]);

        assert_eq!(added, expected);
    }
}
