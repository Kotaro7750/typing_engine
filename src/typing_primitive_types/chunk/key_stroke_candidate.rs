//! A module for key stroke candidates for a chunk.
use crate::typing_primitive_types::{key_stroke::KeyStrokeChar, key_stroke::KeyStrokeString};
use crate::utility::convert_by_weighted_count;
use std::num::NonZeroUsize;

use super::ChunkElementIndex;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// An enum representing the key stroke of a candidate.
pub(crate) enum CandidateKeyStroke {
    /// Key stroke for a candidate that belongs to single character chunk.
    /// This includes both displayable ascii and single character chunk.
    /// Ex. ( "ki" ) for "き" or ( "a" ) for "a"
    Normal(KeyStrokeString),
    /// Key stroke for a candidate that belongs to double character chunk.
    /// Ex. ( "kyo" ) for "きょ"
    Double(KeyStrokeString),
    /// Key stroke for a candidate that belongs to double character chunk and input separately.
    /// Ex. ( "ki", "lyo" ) for "きょ"
    DoubleSplitted(KeyStrokeString, KeyStrokeString),
}

impl CandidateKeyStroke {
    /// Returns if key stroke is double characters.
    pub(crate) fn is_double(&self) -> bool {
        matches!(self, Self::Double(_) | Self::DoubleSplitted(_, _))
    }

    /// Returns if key stroke is double characters and input separately.
    pub(crate) fn is_double_splitted(&self) -> bool {
        matches!(self, Self::DoubleSplitted(_, _))
    }

    /// Returns whole key stroke.
    fn whole_key_stroke(&self) -> KeyStrokeString {
        match self {
            Self::Normal(s) | Self::Double(s) => s.clone(),
            Self::DoubleSplitted(s1, s2) => {
                let mut s = String::new();
                s.push_str(s1);
                s.push_str(s2);
                s.try_into().unwrap()
            }
        }
    }

    /// Returns key stroke count of this candidate.
    fn construct_key_stroke_element_count(&self) -> KeyStrokeElementCount {
        match self {
            Self::Normal(s) | Self::Double(s) => KeyStrokeElementCount::new(&[s.chars().count()]),
            Self::DoubleSplitted(s1, s2) => {
                KeyStrokeElementCount::new(&[s1.chars().count(), s2.chars().count()])
            }
        }
    }

    /// Returns index of key stroke element which passed key stroke index belongs to.
    fn belonging_element_index_of_key_stroke(
        &self,
        key_stroke_index: usize,
    ) -> Result<ChunkElementIndex, ()> {
        if self.whole_key_stroke().chars().count() <= key_stroke_index {
            return Err(());
        }

        match self {
            Self::Normal(_) | Self::Double(_) => Ok(ChunkElementIndex::OnlyFirst),
            Self::DoubleSplitted(s1, _) => {
                let s1_len = s1.chars().count();
                if key_stroke_index < s1_len {
                    Ok(ChunkElementIndex::DoubleFirst)
                } else {
                    Ok(ChunkElementIndex::DoubleSecond)
                }
            }
        }
    }

    /// Returns if passed key stroke index is at the end of key stroke element.
    fn is_element_end_at_key_stroke_index(&self, key_stroke_index: usize) -> Result<bool, ()> {
        if self.whole_key_stroke().chars().count() <= key_stroke_index {
            return Err(());
        }

        match self {
            Self::Normal(s) | Self::Double(s) => Ok(key_stroke_index == s.chars().count() - 1),
            Self::DoubleSplitted(s1, s2) => {
                let s1_len = s1.chars().count();
                let s2_len = s2.chars().count();
                Ok((key_stroke_index == s1_len - 1) || (key_stroke_index == s1_len + s2_len - 1))
            }
        }
    }

    /// Strict the key stroke count of this candidate to key_stroke_count_striction.
    fn strict_key_stroke_count(&mut self, key_stroke_count_striction: NonZeroUsize) {
        match self {
            Self::Normal(s) => {
                let mut new_key_stroke = String::new();
                for (i, c) in s.chars().enumerate() {
                    if i < key_stroke_count_striction.get() {
                        new_key_stroke.push(c);
                    }
                }
                *self = Self::Normal(new_key_stroke.try_into().unwrap());
            }
            Self::Double(s) => {
                let mut new_key_stroke = String::new();
                for (i, c) in s.chars().enumerate() {
                    if i < key_stroke_count_striction.get() {
                        new_key_stroke.push(c);
                    }
                }
                *self = Self::Double(new_key_stroke.try_into().unwrap());
            }
            Self::DoubleSplitted(s1, s2) => {
                let mut new_key_stroke1 = String::new();
                let mut new_key_stroke2 = String::new();
                let mut count = 0;
                for c in s1.chars() {
                    if count < key_stroke_count_striction.get() {
                        new_key_stroke1.push(c);
                    }
                    count += 1;
                }
                for c in s2.chars() {
                    if count < key_stroke_count_striction.get() {
                        new_key_stroke2.push(c);
                    }
                    count += 1;
                }
                *self = Self::DoubleSplitted(
                    new_key_stroke1.try_into().unwrap(),
                    new_key_stroke2.try_into().unwrap(),
                );
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct representing single candidate of key strokes
pub(crate) struct ChunkKeyStrokeCandidate {
    /// Key stroke of this candidate.
    key_stroke: CandidateKeyStroke,
    /// Constraint for the next chunk head key stroke.
    /// This constraint is used when the next chunk head key stroke is restricted by this
    /// candidate.
    next_chunk_head_constraint: Option<KeyStrokeChar>,
    /// Information of delayed confirmed candidate if this candidate is a delayed confirmed
    /// candidate.
    delayed_confirmed_candidate_info: Option<DelayedConfirmedCandidateInfo>,
}

impl ChunkKeyStrokeCandidate {
    pub(crate) fn new(
        key_stroke: CandidateKeyStroke,
        next_chunk_head_constraint: Option<KeyStrokeChar>,
        delayed_confirmed_candidate_info: Option<DelayedConfirmedCandidateInfo>,
    ) -> Self {
        Self {
            key_stroke,
            next_chunk_head_constraint,
            delayed_confirmed_candidate_info,
        }
    }

    /// Returns key stroke of this candidate.
    pub(crate) fn key_stroke(&self) -> &CandidateKeyStroke {
        &self.key_stroke
    }

    /// Returns constraints for the next chunk head key stroke of this candidate.
    pub(crate) fn next_chunk_head_constraint(&self) -> &Option<KeyStrokeChar> {
        &self.next_chunk_head_constraint
    }

    /// Returns delay confirmed candidate information of this candidate.
    pub(super) fn delayed_confirmed_candidate_info(
        &self,
    ) -> &Option<DelayedConfirmedCandidateInfo> {
        &self.delayed_confirmed_candidate_info
    }

    /// Returns whole key stroke string of this candidate.
    pub(crate) fn whole_key_stroke(&self) -> KeyStrokeString {
        self.key_stroke.whole_key_stroke()
    }

    /// Returns if this candidate is a delayed confirmed candidate.
    pub(super) fn is_delayed_confirmed_candidate(&self) -> bool {
        self.delayed_confirmed_candidate_info().is_some()
    }

    /// Returns if passed key stroke is valid for this candidate current cursor position.
    pub(super) fn is_hit(&self, key_stroke: &KeyStrokeChar, cursor_position: usize) -> bool {
        self.key_stroke_char_at_position(cursor_position) == *key_stroke
    }

    /// Restrics the key stroke count of this candidate to key_stroke_count_striction.
    ///
    /// This function is assumed to be called when the chunk of this candidate is the last chunk.
    pub(crate) fn strict_key_stroke_count(&mut self, key_stroke_count_striction: NonZeroUsize) {
        assert!(
            key_stroke_count_striction.get() < self.calc_key_stroke_count()
                || self.is_delayed_confirmed_candidate()
        );

        self.key_stroke
            .strict_key_stroke_count(key_stroke_count_striction);

        // この候補の属するチャンクが最後のチャンクであることを想定しているので次のチャンクへの制限はなくてもよい
        self.next_chunk_head_constraint.take();
        // 遅延確定候補は普通の候補にする必要がある
        self.delayed_confirmed_candidate_info.take();
    }

    /// Return how many key strokes are needed to type this candidate.
    pub(crate) fn calc_key_stroke_count(&self) -> usize {
        self.whole_key_stroke().chars().count()
    }

    /// Return key stroke char at the passed position index.
    pub(super) fn key_stroke_char_at_position(&self, position: usize) -> KeyStrokeChar {
        let whole_key_stroke = self.whole_key_stroke();

        assert!(position < whole_key_stroke.chars().count());

        whole_key_stroke
            .chars()
            .nth(position)
            .unwrap()
            .try_into()
            .unwrap()
    }

    /// Returns index of key stroke element which passed key stroke index belongs to.
    pub(super) fn belonging_element_index_of_key_stroke(
        &self,
        key_stroke_index: usize,
    ) -> Option<ChunkElementIndex> {
        self.key_stroke()
            .belonging_element_index_of_key_stroke(key_stroke_index)
            .ok()
    }

    /// Returns if passed key stroke index is at the end of key stroke element.
    pub(super) fn is_element_end_at_key_stroke_index(
        &self,
        key_stroke_index: usize,
    ) -> Option<bool> {
        self.key_stroke()
            .is_element_end_at_key_stroke_index(key_stroke_index)
            .ok()
    }

    /// Returns key stroke element count of this candidate.
    pub(crate) fn construct_key_stroke_element_count(&self) -> KeyStrokeElementCount {
        self.key_stroke().construct_key_stroke_element_count()
    }
}

/// 打ち終えてもチャンクを確定させてはいけない遅延確定候補の情報
/// ex. 「ん」というチャンクには「n」・「nn」・「xn」という候補があるが「n」というキーストロークの後に「n」と打った場合にもこのチャンクのキーストロークとして有効である
/// このようなケースにも適切にタイプを行うためには最初の「n」というキーストロークで完全に確定してはならない
/// 一方最初の「n」の後のチャンク先頭として有効なキーストロークが与えられた場合にはチャンクを確定させて次のチャンクの先頭を打ったことにする必要がある
///
/// 一般的には
/// チャンク1のある候補Aのキーストローク列が他の候補Bの接頭辞となっている場合に候補Aはチャンク1の遅延確定候補となる
/// 候補Aを確定するのは次のチャンク2の先頭キーストロークが打たれた時
/// 図示すると
///
/// チャンク1  -> チャンク2
/// -----------------------
/// 候補A(X)
///            -> 候補C(Z)
/// 候補B(XxY)
///
/// X,Y,Z := キーストローク列
/// x := キーストローク
/// ただしZの先頭キーストロークはxであってはいけない
/// もしZの先頭キーストロークがxだとすると候補Bを打とうとしているのかチャンク2の候補Cを打とうとしているのか1回のキーストロークで確定せず2回以上遅延する必要がある
///
/// ただし実際には必ず1回のキーストロークで確定する
/// 遅延確定候補となるのは
/// 1. チャンク「ん」の「n」という候補（「nn」が候補B）
/// 2. チャンク「っ」を「l」「x」で打てるときの「l」「x」という候補（「ltu」「xtu」「ltsu」が候補B）
///     に限られるが(TODO たぶんそうだが確証はもてない)
/// 1. 次のチャンクが「n」で始まるときにはそもそも「n」は候補になることはない
/// 2. チャンク「っ」を「l」や「x」で打てるときには次のチャンク先頭のキーストロークは「t」ではない
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DelayedConfirmedCandidateInfo {
    // 次のチャンク先頭として有効なキーストローク列
    next_chunk_head: Vec<KeyStrokeChar>,
}

impl DelayedConfirmedCandidateInfo {
    pub(crate) fn new(next_chunk_head: Vec<KeyStrokeChar>) -> Self {
        Self { next_chunk_head }
    }

    /// Returns if passed key stroke is valid for the next chunk head and can confirm the chunk.
    pub(crate) fn can_confirm_with_key_stroke(&self, key_stroke: KeyStrokeChar) -> bool {
        self.next_chunk_head.contains(&key_stroke)
    }
}

// TODO: この型やメソッド名は意味がわかりにくいので変更する
/// An enum representing key stroke count for each key stroke elements of a candidate.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum KeyStrokeElementCount {
    /// Key stroke count for single key stroke element.
    Sigle(usize),
    /// Key stroke count for double key stroke elements.
    /// This is used when the chunk is double characters and each character is input separately.
    Double((usize, usize)),
}

impl KeyStrokeElementCount {
    pub(crate) fn new(counts: &[usize]) -> Self {
        match counts.len() {
            1 => Self::Sigle(counts[0]),
            2 => Self::Double((counts[0], counts[1])),
            _ => unreachable!(
                "key stroke elements count must be 1 or 2, but {}",
                counts.len()
            ),
        }
    }

    /// Returns whole key stroke count of a candidate.
    pub(crate) fn whole_count(&self) -> usize {
        match self {
            Self::Sigle(c) => *c,
            Self::Double((c1, c2)) => c1 + c2,
        }
    }

    /// Returns whether double key stroke elements or not.
    pub(crate) fn is_double(&self) -> bool {
        matches!(self, Self::Double(_))
    }

    /// 理想的なキーストローク・キーストロークの位置を綴りの位置に変換する
    pub(crate) fn convert_key_stroke_delta_to_spell_delta(
        &self,
        spell: usize,
        key_stroke_delta: usize,
    ) -> usize {
        let spell_elements_index = self.spell_elements_index_of_delta(key_stroke_delta);

        let in_spell_element_key_stroke_delta = if spell_elements_index == 0 {
            key_stroke_delta
        } else {
            key_stroke_delta - self.key_stroke_count_offset(spell_elements_index)
        };

        let effective_spell_count = if self.is_double() { 1 } else { spell };

        convert_by_weighted_count(
            self.count_of_spell_elements_index(spell_elements_index),
            effective_spell_count,
            in_spell_element_key_stroke_delta,
        ) + spell_elements_index
    }

    /// 綴りの位置を理想的なキーストローク・キーストロークの位置に変換する
    pub(crate) fn convert_spell_delta_to_key_stroke_delta(
        &self,
        spell: usize,
        spell_delta: usize,
    ) -> usize {
        let pseudo_count = self.construct_pseudo_count_of_spell_elements(spell);

        pseudo_count.key_stroke_count_offset(spell_delta - 1)
            + pseudo_count.count_of_spell_elements_index(spell_delta - 1)
    }

    /// 綴りのどの位置に属すかという観点で擬似的な綴り要素ごとの個数を構築する
    /// ex. 「きょ」という綴りの「kyo」というキーストロークの綴り要素は1つだが
    /// 位置変換という文脈ではkは0番目に属しyoは1番目に属する
    pub(crate) fn construct_pseudo_count_of_spell_elements(
        &self,
        spell: usize,
    ) -> KeyStrokeElementCount {
        if (!self.is_double() && spell == 1) || (self.is_double() && spell == 2) {
            return self.clone();
        }

        assert_eq!(spell, 2);
        let key_stroke_count = match self {
            Self::Sigle(c) => *c,
            Self::Double(_) => unreachable!(),
        };

        let mut v = vec![0; spell];

        for i in 1..=key_stroke_count {
            let spell_delta = self.convert_key_stroke_delta_to_spell_delta(spell, i);
            v[spell_delta - 1] += 1;
        }

        Self::Double((v[0], v[1]))
    }

    /// Returns index of key stroke count for passed key stroke delta.
    pub(crate) fn spell_elements_index_of_delta(&self, key_stroke_delta: usize) -> usize {
        match self {
            Self::Sigle(c) => {
                assert!(key_stroke_delta <= *c);
                0
            }
            Self::Double((c1, c2)) => {
                assert!(key_stroke_delta <= (c1 + c2));
                if key_stroke_delta <= *c1 {
                    0
                } else {
                    1
                }
            }
        }
    }

    /// Returns key stroke count for passed spell elements index.
    pub(crate) fn count_of_spell_elements_index(&self, spell_elements_index: usize) -> usize {
        match self {
            Self::Sigle(c) => *c,
            Self::Double((c1, c2)) => {
                if spell_elements_index == 0 {
                    *c1
                } else if spell_elements_index == 1 {
                    *c2
                } else {
                    unreachable!();
                }
            }
        }
    }

    /// Returns index of key stroke count offset for passed spell elements index.
    pub(crate) fn key_stroke_count_offset(&self, spell_elements_index: usize) -> usize {
        match self {
            Self::Sigle(_) => 0,
            Self::Double((c1, _)) => {
                if spell_elements_index == 0 {
                    0
                } else {
                    *c1
                }
            }
        }
    }
}
