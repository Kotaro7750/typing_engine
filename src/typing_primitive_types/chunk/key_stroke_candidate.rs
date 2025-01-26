//! A module for key stroke candidates for a chunk.
use crate::typing_primitive_types::{key_stroke::KeyStrokeChar, key_stroke::KeyStrokeString};
use crate::utility::convert_by_weighted_count;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct representing single candidate of key strokes for a chunk.
pub struct ChunkKeyStrokeCandidate {
    /// Key stroke of this candidate.
    /// Although this vec usually has only one element, it has two elements when the chunk is
    /// double characters and each character is input separately like "きょ".
    key_stroke_elements: Vec<KeyStrokeString>,
    /// Constraint for the next chunk head key stroke.
    /// This constraint is used when the next chunk head key stroke is restricted by this
    /// candidate.
    next_chunk_head_constraint: Option<KeyStrokeChar>,
    /// Information of delayed confirmed candidate if this candidate is a delayed confirmed
    /// candidate.
    delayed_confirmed_candidate_info: Option<DelayedConfirmedCandidateInfo>,

    // TODO
    // candidateを付与する段階で後のキーストロークの制約を満たすもののみを選択しているが、チャンク構築時にありえるもの全てを付与しておいて後続のチャンクを追加する際に制約を満たさないものを無効にするというやり方もありえる。
    // インクリメンタルなチャンク追加がやりやすくなるかもしれない
    /// Whether this candidate is target of typing.
    /// Inactive candidate is reduced due to other candidate is hit and so on.
    is_active: bool,

    // TODO
    cursor_position: Option<usize>,
}

impl ChunkKeyStrokeCandidate {
    pub(crate) fn new(
        key_stroke_elements: Vec<KeyStrokeString>,
        next_chunk_head_constraint: Option<KeyStrokeChar>,
        delayed_confirmed_candidate_info: Option<DelayedConfirmedCandidateInfo>,
        is_active: bool,
        cursor_position: Option<usize>,
    ) -> Self {
        Self {
            key_stroke_elements,
            next_chunk_head_constraint,
            delayed_confirmed_candidate_info,
            is_active,
            cursor_position,
        }
    }

    /// Returns constraints for the next chunk head key stroke of this candidate.
    pub(crate) fn next_chunk_head_constraint(&self) -> &Option<KeyStrokeChar> {
        &self.next_chunk_head_constraint
    }

    /// Returns delay confirmed candidate information of this candidate.
    pub(crate) fn delayed_confirmed_candiate_info(&self) -> &Option<DelayedConfirmedCandidateInfo> {
        &self.delayed_confirmed_candidate_info
    }

    /// Returns current cursor position of this candidate.
    pub(crate) fn cursor_position(&self) -> Option<usize> {
        self.cursor_position
    }

    /// Returns if this candidate is active.
    pub(crate) fn is_active(&self) -> bool {
        self.is_active
    }

    /// Inactivate this candidate.
    pub(crate) fn inactivate(&mut self) {
        self.is_active = false;
    }

    /// Returns if this candidate is confirmed.
    /// This distinction is done by comparing cursor position.
    pub(crate) fn is_confirmed(&self) -> bool {
        match self.cursor_position {
            Some(cursor_position) => cursor_position >= self.calc_key_stroke_count(),
            None => false,
        }
    }

    /// Returns if chunk of this candidate is double characters and each character is input
    /// separately like "きょ".
    pub(crate) fn is_splitted(&self) -> bool {
        self.key_stroke_elements.len() == 2
    }

    /// Returns if this candidate is a delayed confirmed candidate.
    pub(crate) fn is_delayed_confirmed_candidate(&self) -> bool {
        self.delayed_confirmed_candidate_info.is_some()
    }

    /// Returns index of key stroke element which passed key stroke index belongs to.
    /// Usually, this function returns 0, but it can return 1 when the chunk is double characters
    /// and each character is input separately.
    pub(crate) fn belonging_element_index_of_key_stroke(&self, key_stroke_index: usize) -> usize {
        assert!(key_stroke_index < self.calc_key_stroke_count());

        let mut element_index = 0;

        let mut element_head_key_stroke_index = 0;
        for (i, element) in self.key_stroke_elements.iter().enumerate() {
            if key_stroke_index < element_head_key_stroke_index + element.chars().count() {
                element_index = i;
                break;
            }

            element_head_key_stroke_index += element.chars().count();
        }

        element_index
    }

    /// Returns if passed key stroke index is at the end of key stroke element.
    pub(crate) fn is_element_end_at_key_stroke_index(&self, key_stroke_index: usize) -> bool {
        assert!(key_stroke_index < self.calc_key_stroke_count());

        let mut element_head_key_stroke_index_index = 0;

        for element in &self.key_stroke_elements {
            let element_len = element.chars().count();

            if key_stroke_index == (element_head_key_stroke_index_index + element_len - 1) {
                return true;
            }
            element_head_key_stroke_index_index += element_len;
        }

        false
    }

    /// Returns whole key stroke string of this candidate.
    pub(crate) fn whole_key_stroke(&self) -> KeyStrokeString {
        let mut s = String::new();

        for key_stroke in &self.key_stroke_elements {
            s.push_str(key_stroke);
        }

        s.try_into().unwrap()
    }

    /// Returns key stroke element count of this candidate.
    pub(crate) fn construct_key_stroke_element_count(&self) -> KeyStrokeElementCount {
        KeyStrokeElementCount::new(
            &(self
                .key_stroke_elements
                .iter()
                .map(|s| s.chars().count())
                .collect::<Vec<usize>>()),
        )
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

    /// Returns if passed key stroke is valid for this candidate current cursor position.
    pub(super) fn is_hit(&self, key_stroke: &KeyStrokeChar) -> bool {
        match self.cursor_position {
            Some(cursor_position) => {
                self.key_stroke_char_at_position(cursor_position) == *key_stroke
            }
            None => false,
        }
    }

    /// Advance cursor position of this candidate.
    /// If cursor position is None, this function reset cursor position to 0.
    pub(super) fn advance_cursor(&mut self) {
        match self.cursor_position {
            Some(cursor_position) => {
                self.cursor_position = Some(cursor_position + 1);
            }
            None => {
                self.cursor_position = Some(0);
            }
        }
    }

    /// Return how many key strokes are needed to type this candidate.
    pub(crate) fn calc_key_stroke_count(&self) -> usize {
        self.whole_key_stroke().chars().count()
    }

    /// Restrics the key stroke count of this candidate to key_stroke_count_striction.
    ///
    /// This function is assumed to be called when the chunk of this candidate is the last chunk.
    pub(super) fn strict_key_stroke_count(&mut self, key_stroke_count_striction: NonZeroUsize) {
        assert!(
            key_stroke_count_striction.get() < self.calc_key_stroke_count()
                || self.is_delayed_confirmed_candidate()
        );

        let mut new_key_stroke_elements: Vec<KeyStrokeString> = Vec::new();

        let mut count = 0;
        for key_stroke_element in &self.key_stroke_elements {
            let count_of_element = key_stroke_element.chars().count();

            if count + count_of_element > key_stroke_count_striction.get() {
                let count_after_truncate = key_stroke_count_striction.get() - count;

                let mut truncated = String::new();
                for (i, c) in key_stroke_element.chars().enumerate() {
                    if i < count_after_truncate {
                        truncated.push(c);
                    }
                }

                new_key_stroke_elements.push(truncated.try_into().unwrap());
                break;
            }

            new_key_stroke_elements.push(key_stroke_element.clone());
            count += count_of_element;
        }

        self.key_stroke_elements = new_key_stroke_elements;
        // この候補の属するチャンクが最後のチャンクであることを想定しているので次のチャンクへの制限はなくてもよい
        self.next_chunk_head_constraint.take();
        // 遅延確定候補は普通の候補にする必要がある
        self.delayed_confirmed_candidate_info.take();
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
/// に限られるが(TODO たぶんそうだが確証はもてない)
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
        match self {
            Self::Double(_) => true,
            _ => false,
        }
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
