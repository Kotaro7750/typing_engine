use std::num::NonZeroUsize;

use crate::typing_primitive_types::chunk::candidate_unappended::ChunkCandidateUnappended;
use crate::typing_primitive_types::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::typing_primitive_types::spell::SpellString;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
/// Each spells of a vocabulary.
/// [`Vec<VocabularySpellElement>`] represents spells for single vocabulary.
pub enum VocabularySpellElement {
    /// Spell for almost all of spells.
    /// This variant represents spells for single charactor of view.
    ///
    /// Ex. When vocabulary is
    /// * `巨大`, spells are `[Normal("きょ"), Normal("だい")]`
    /// * `Big`, spells are `[Normal("B"), Normal("i"), Normal("g")]`
    Normal(SpellString),
    /// Spell for compound vocabularies(熟字訓).
    /// This variant represents spells for some parts of view.
    ///
    /// Second element of inner tuple represents how many view charactors this spell corresponds to.
    ///
    /// Ex. When vocabulary is
    /// * `今日`, spells are `[Compound("きょう", 2)]`
    /// * `五月雨`, spells are `[Compound("さみだれ", 3)]`
    Compound((SpellString, NonZeroUsize)),
}

impl VocabularySpellElement {
    // Construct single SpellString of this element.
    pub(crate) fn construct_whole_spell(&self) -> SpellString {
        match self {
            Self::Normal(spell) | Self::Compound((spell, _)) => spell.clone(),
        }
    }
}

/// A struct representing a vocabulary for constructing query.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct VocabularyEntry {
    view: String,
    spells: Vec<VocabularySpellElement>,
}

impl VocabularyEntry {
    /// Construct a new [`VocabularyEntry`].
    ///
    /// `view` is this vocabulary itself.
    /// Each element of `spells` describes spells of each part of `view` string.
    ///
    /// For example,
    /// * `"巨大"` has `"巨大"` as `view` , and `[VocabularySpellElement::Normal("きょ"), VocabularySpellElement::Normal("だい")]` as `spells`
    /// * `"Big"` has `"Big"` as `view` , and `[VocabularySpellElement::Normal("B"),VocabularySpellElement::Normal("i"),VocabularySpellElement::Normal("g")]` as `spells`
    /// * `"七夕送り"` has `"七夕送り"` as `view` , and `[VocabularySpellElement::Compound("たなばた", 2), VocabularySpellElement::Normal("おく"), VocabularySpellElement::Normal("り")]` as `spells`
    pub fn new(view: String, spells: Vec<VocabularySpellElement>) -> Option<Self> {
        let view_count = spells.iter().fold(0, |acc, vocabulary_spell_element| {
            acc + match vocabulary_spell_element {
                VocabularySpellElement::Normal(_) => 1,
                VocabularySpellElement::Compound((_, count)) => count.get(),
            }
        });

        if view.chars().count() != view_count {
            None
        } else {
            Some(Self { view, spells })
        }
    }

    /// Return the view of this vocabulary.
    pub fn view(&self) -> &str {
        self.view.as_str()
    }

    /// Return the spells of this vocabulary.
    pub fn spells(&self) -> &Vec<VocabularySpellElement> {
        &self.spells
    }

    /// Construct the whole spell of this entry.
    /// This is accomlished by concatenating spell elements of this entry.
    pub(crate) fn construct_whole_spell(&self) -> SpellString {
        let mut s = String::new();

        self.spells
            .iter()
            .for_each(|spell| s.push_str(&spell.construct_whole_spell()));

        s.try_into().unwrap()
    }

    /// Construct VocabularyInfo from this entry.
    pub(crate) fn construct_vocabulary_info(&self, chunk_count: NonZeroUsize) -> VocabularyInfo {
        let mut view_position_of_spell: Vec<ViewPosition> = vec![];

        let mut i = 0;
        self.spells.iter().for_each(|spell| match spell {
            VocabularySpellElement::Normal(spell) => {
                spell.chars().for_each(|_| {
                    view_position_of_spell.push(ViewPosition::Normal(i));
                });
                i += 1;
            }
            VocabularySpellElement::Compound((spell, view_count)) => {
                spell.chars().for_each(|_| {
                    view_position_of_spell.push(ViewPosition::Compound(
                        (i..(i + view_count.get())).collect(),
                    ));
                });
                i += view_count.get();
            }
        });

        VocabularyInfo {
            view: self.view.clone(),
            spell: self.construct_whole_spell(),
            view_position_of_spell,
            chunk_count,
        }
    }

    /// Construct chunks from this entry.
    /// At this stage, key stroke candidates for each chunk are not set.
    pub(crate) fn construct_chunks(&self) -> Vec<ChunkCandidateUnappended> {
        let mut chunks = Vec::<ChunkCandidateUnappended>::new();

        let spell_chars: Vec<char> = self.construct_whole_spell().chars().collect();

        let mut i = 0;
        while i < spell_chars.len() {
            // uniグラムとbiグラムの内長い方をチャンクとして採用する
            let uni = spell_chars[i];
            let bi = if i != spell_chars.len() - 1 {
                let mut bi = spell_chars[i].to_string();
                bi.push(spell_chars[i + 1]);
                bi
            } else {
                String::from("")
            };

            let spell =
                if uni.is_ascii_graphic() || uni == ' ' {
                    i += 1;
                    uni.to_string()
                } else if CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY.contains_key(bi.as_str()) {
                    i += 2;
                    bi
                } else {
                    assert!(
                        CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY.contains_key(uni.to_string().as_str())
                    );
                    i += 1;
                    uni.to_string()
                }
                .try_into()
                .unwrap();

            chunks.push(ChunkCandidateUnappended::new(spell));
        }

        chunks
    }
}

/// An enum representing position indexes in view string for each spell charactors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ViewPosition {
    Normal(usize),
    /// Represents position indexes for a spell charactor in compound vocabulary.
    Compound(Vec<usize>),
}

impl ViewPosition {
    /// Apply static offset to position.
    fn offset(&self, offset: usize) -> Self {
        match self {
            Self::Normal(position) => Self::Normal(position + offset),
            Self::Compound(positions) => {
                Self::Compound(positions.iter().map(|position| position + offset).collect())
            }
        }
    }

    /// Return the last view position of corrensponding spell charactor.
    pub(crate) fn last_position(&self) -> usize {
        match self {
            Self::Normal(position) => *position,
            Self::Compound(positions) => *(positions.last().unwrap()),
        }
    }
}

/// Return view position indexes for each spell positions.
pub(crate) fn corresponding_view_positions_for_spell(
    spell_positions: &[usize],
    view_position_of_spell: &[ViewPosition],
) -> Vec<usize> {
    let mut view_positions = vec![];

    spell_positions.iter().for_each(|spell_position| {
        // カーソル位置など綴り字数を超える場合がある
        let view_position = if *spell_position >= view_position_of_spell.len() {
            view_position_of_spell.last().unwrap()
        } else {
            view_position_of_spell.get(*spell_position).unwrap()
        };

        match view_position {
            ViewPosition::Normal(position) => view_positions.push(*position),
            ViewPosition::Compound(positions) => positions
                .iter()
                .for_each(|position| view_positions.push(*position)),
        }
    });

    view_positions
}

/// A struct representing an actual vocabulary in a query.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VocabularyInfo {
    view: String,
    spell: SpellString,
    /// Corresponding position index in view string for each spell charactors. Position is 0 oriented.
    view_position_of_spell: Vec<ViewPosition>,
    chunk_count: NonZeroUsize,
}

impl VocabularyInfo {
    #[cfg(test)]
    pub(crate) fn new(
        view: String,
        spell: SpellString,
        view_position_of_spell: Vec<ViewPosition>,
        chunk_count: NonZeroUsize,
    ) -> Self {
        Self {
            view,
            spell,
            view_position_of_spell,
            chunk_count,
        }
    }

    /// Return the count of chunks of this vocabulary.
    pub(crate) fn chunk_count(&self) -> NonZeroUsize {
        self.chunk_count
    }

    /// Return the view of this vocabulary.
    pub(crate) fn view(&self) -> &str {
        self.view.as_str()
    }

    /// Reset the count of chunks of this vocabulary.
    pub(crate) fn reset_chunk_count(&mut self, chunk_count: NonZeroUsize) {
        self.chunk_count = chunk_count;
    }
}

/// Construct view position indexes for spell charactors concatenated from passed VocabularyInfos.
pub(crate) fn view_position_of_spell_for_vocabulary_infos(
    vocabulary_infos: &[VocabularyInfo],
) -> Vec<ViewPosition> {
    let mut view_position_of_spell: Vec<ViewPosition> = vec![];

    let mut index = 0;

    vocabulary_infos.iter().for_each(|vocabulary_info| {
        vocabulary_info
            .view_position_of_spell
            .iter()
            .for_each(|in_vocabulary_view_position| {
                view_position_of_spell.push(in_vocabulary_view_position.offset(index));
            });

        index += vocabulary_info.view().chars().count();
    });

    view_position_of_spell
}

#[cfg(test)]
mod test {
    use crate::{gen_chunk_candidate_unappended, gen_vocabulary_entry};

    use super::{corresponding_view_positions_for_spell, ViewPosition};

    macro_rules! equal_check_construct_chunks {
        (($vs:literal,[$(($spell:literal$(,$view_count:literal)?)),*]), [$($s:literal),*]) => {
            let ve = gen_vocabulary_entry!($vs,[$(($spell$(,$view_count)?)),*]);

            assert_eq!(
                ve.construct_chunks(),
                vec![$(gen_chunk_candidate_unappended!($s)),*]
            );
        };
    }

    #[test]
    fn construct_chunks_from_vocabulary_entry_1() {
        equal_check_construct_chunks!(("今日", [("きょう", 2)]), ["きょ", "う"]);
    }

    #[test]
    fn construct_chunks_from_vocabulary_entry_2() {
        equal_check_construct_chunks!((" 　", [(" "), ("　")]), [" ", "　"]);
    }

    #[test]
    fn construct_chunks_from_vocabulary_entry_3() {
        equal_check_construct_chunks!(("big", [("b"), ("i"), ("g")]), ["b", "i", "g"]);
    }

    #[test]
    fn corresponding_view_positions_for_spell_1() {
        let vp = corresponding_view_positions_for_spell(
            &vec![0, 1, 2],
            &vec![
                ViewPosition::Compound(vec![0, 1, 2, 3]),
                ViewPosition::Compound(vec![0, 1, 2, 3]),
                ViewPosition::Normal(4),
            ],
        );

        assert_eq!(vp, vec![0, 1, 2, 3, 0, 1, 2, 3, 4]);
    }
}
