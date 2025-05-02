use std::{collections::BTreeMap, hash::Hash, time::Duration};

use super::StatisticalEvent;
use crate::KeyStrokeChar;
use public::{EntitySkillStatistics, SkillStatistics};

/// A module for public items related to skill statistics.
pub(crate) mod public;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct SkillStatisticsManager {
    /// Skill statistics for a single key stroke.
    single_key_stroke: SingleKeyStrokeSkillStatistics,
}

impl SkillStatisticsManager {
    pub(crate) fn new() -> Self {
        Self {
            single_key_stroke: SingleKeyStrokeSkillStatistics::new(),
        }
    }

    /// Consume the event and update statistics.
    pub(crate) fn consume_event(&mut self, event: &StatisticalEvent) {
        if let StatisticalEvent::KeyStrokeCorrect(key_stroke_correct_context) = event {
            self.single_key_stroke.update(
                key_stroke_correct_context.key_stroke(),
                key_stroke_correct_context.required_time(),
                key_stroke_correct_context.wrong_key_strokes(),
            );
        }
    }

    /// Construct [`SilkStatistics`](SkillStatistics)
    pub(crate) fn construct_skill_statistics(&self) -> SkillStatistics {
        SkillStatistics::new_with(self.single_key_stroke.skill_statisticses())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing skill statistics for a single key stroke.
pub(crate) struct SingleKeyStrokeSkillStatistics {
    /// Map of [`PrimitiveSkillStatistics`](PrimitiveSkillStatistics) for each key stroke
    /// character.
    statistics_per_key_stroke_char:
        BTreeMap<KeyStrokeChar, PrimitiveSkillStatistics<KeyStrokeChar>>,
}

impl SingleKeyStrokeSkillStatistics {
    fn new() -> Self {
        Self {
            statistics_per_key_stroke_char: BTreeMap::new(),
        }
    }

    /// Returns a vector of [`SkillStatistics`](SkillStatistics) for each [`KeyStrokeChar`](KeyStrokeChar).
    pub(crate) fn skill_statisticses(&self) -> Vec<EntitySkillStatistics<KeyStrokeChar>> {
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

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// A struct representing skill statistics for a single entity.
struct PrimitiveSkillStatistics<T: Eq + Hash + Clone> {
    /// Count of occurrences of entity.
    count: usize,
    /// Cumulative required time of all occurrences.
    cumulative_time: Duration,
    /// Map of wrong occurrences count for other entity.
    wrong_count_map: BTreeMap<T, usize>,
    /// Count of occurrences without wrong.
    completely_correct_count: usize,
}

impl<T: Eq + Hash + Clone + Ord> PrimitiveSkillStatistics<T> {
    fn new() -> Self {
        Self {
            count: 0,
            completely_correct_count: 0,
            cumulative_time: Duration::from_secs(0),
            wrong_count_map: BTreeMap::new(),
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
        EntitySkillStatistics::new_with(
            entity,
            self.count,
            self.cumulative_time,
            self.wrong_count_map,
            self.completely_correct_count,
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::statistics::statistical_event::KeyStrokeCorrectContext;

    #[test]
    fn empty_skill_statistics_manager() {
        let ssm = SkillStatisticsManager::new();

        assert_eq!(
            ssm.construct_skill_statistics().single_key_stroke().len(),
            0
        );
    }

    #[test]
    fn key_stroke_correct_event_update_singke_key_stroke_statistics() {
        let mut ssm = SkillStatisticsManager::new();

        ssm.consume_event(&StatisticalEvent::KeyStrokeCorrect(
            KeyStrokeCorrectContext::new(
                'a'.try_into().unwrap(),
                Duration::from_secs(1),
                vec!['b'.try_into().unwrap()],
            ),
        ));
        ssm.consume_event(&StatisticalEvent::KeyStrokeCorrect(
            KeyStrokeCorrectContext::new(
                'b'.try_into().unwrap(),
                Duration::from_secs(2),
                vec!['c'.try_into().unwrap()],
            ),
        ));

        assert_eq!(
            ssm.construct_skill_statistics().single_key_stroke(),
            vec![
                EntitySkillStatistics::new_with(
                    'a'.try_into().unwrap(),
                    1,
                    Duration::from_secs(1),
                    BTreeMap::from([('b'.try_into().unwrap(), 1)]),
                    0,
                ),
                EntitySkillStatistics::new_with(
                    'b'.try_into().unwrap(),
                    1,
                    Duration::from_secs(2),
                    BTreeMap::from([('c'.try_into().unwrap(), 1)]),
                    0,
                )
            ]
        );
    }

    #[test]
    fn empty_primitive_skill_statistics() {
        let pss = PrimitiveSkillStatistics::<KeyStrokeChar>::new();

        assert_eq!(
            pss.into_with_entity('a'.try_into().unwrap()),
            EntitySkillStatistics::new_with(
                'a'.try_into().unwrap(),
                0,
                Duration::from_secs(0),
                BTreeMap::new(),
                0,
            )
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
            EntitySkillStatistics::new_with(
                'a'.try_into().unwrap(),
                1,
                Duration::from_secs(1),
                BTreeMap::new(),
                1,
            )
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
            EntitySkillStatistics::new_with(
                'a'.try_into().unwrap(),
                1,
                Duration::from_secs(1),
                BTreeMap::from([('a'.try_into().unwrap(), 2), ('b'.try_into().unwrap(), 1),]),
                0,
            )
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
            EntitySkillStatistics::new_with(
                'a'.try_into().unwrap(),
                3,
                Duration::from_secs(6),
                BTreeMap::from([('a'.try_into().unwrap(), 2), ('b'.try_into().unwrap(), 1),]),
                1,
            )
        );
    }
}
