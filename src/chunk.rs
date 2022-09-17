use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::key_stroke::{KeyStrokeChar, KeyStrokeString};
use crate::spell::SpellString;
use crate::utility::convert_by_weighted_count;

pub(crate) mod confirmed;
pub(crate) mod has_actual_key_strokes;
pub(crate) mod typed;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ChunkSpell {
    DisplayableAscii(SpellString),
    SingleChar(SpellString),
    DoubleChar(SpellString),
}

impl ChunkSpell {
    fn new(ss: SpellString) -> Self {
        if ss.contains_displayable_ascii() {
            assert!(ss.chars().count() == 1);
            Self::DisplayableAscii(ss)
        } else {
            match ss.chars().count() {
                1 => Self::SingleChar(ss),
                2 => Self::DoubleChar(ss),
                _ => unreachable!("ChunkSpell's length must be 1 or 2"),
            }
        }
    }

    // 2文字のチャンクから1文字ずつの2つの綴りを生成する
    fn split_double_char(&self) -> (SpellString, SpellString) {
        match self {
            Self::DoubleChar(spell_string) => (
                spell_string
                    .chars()
                    .next()
                    .unwrap()
                    .to_string()
                    .try_into()
                    .unwrap(),
                spell_string
                    .chars()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .try_into()
                    .unwrap(),
            ),
            _ => panic!("cannot split this ChunkSpell type"),
        }
    }

    // 綴りの文字数
    pub(crate) fn count(&self) -> usize {
        match self {
            ChunkSpell::DoubleChar(_) => 2,
            _ => 1,
        }
    }
}

impl AsRef<SpellString> for ChunkSpell {
    fn as_ref(&self) -> &SpellString {
        match self {
            ChunkSpell::DisplayableAscii(ss)
            | ChunkSpell::SingleChar(ss)
            | ChunkSpell::DoubleChar(ss) => ss,
        }
    }
}

// タイピングの入力単位
// 基本的には綴りは１文字だが「きょ」など複数文字の綴りになる場合もある
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Chunk {
    spell: ChunkSpell,
    // チャンクを入力するためのキーストロークは複数の候補がありえる
    // ex. 「きょ」というチャンクには「kyo」・「kilyo」といったキーストロークがある
    key_stroke_candidates: Option<Vec<ChunkKeyStrokeCandidate>>,
    // 最短で打ったときの候補
    // キーストローク付与時に決められるためキーストローク系列によってはこの候補を打つことができない場合もある
    ideal_candidate: Option<ChunkKeyStrokeCandidate>,
}

impl Chunk {
    pub fn new(
        spell: SpellString,
        key_stroke_candidates: Option<Vec<ChunkKeyStrokeCandidate>>,
        ideal_candidate: Option<ChunkKeyStrokeCandidate>,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            key_stroke_candidates,
            ideal_candidate,
        }
    }

    pub(crate) fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    pub(crate) fn key_stroke_candidates(&self) -> &Option<Vec<ChunkKeyStrokeCandidate>> {
        &self.key_stroke_candidates
    }

    pub(crate) fn ideal_key_stroke_candidate(&self) -> &Option<ChunkKeyStrokeCandidate> {
        &self.ideal_candidate
    }

    // このチャンクを打つのに必要な最小のキーストローク数を推測する
    // キーストロークをまだ付与していないチャンクに対して行うため推測である
    pub fn estimate_min_key_stroke_count(&self) -> usize {
        assert!(self.key_stroke_candidates.is_none());

        // ここで推測するのはあくまでも最小なので基本的には変換辞書から引いたものをそのまま使う
        // これは，2文字のチャンクの最小キーストロークは2文字をまとめて打つものだからである
        // 「っ」は次のチャンクによっては1回のキーストロークで打てるため1回としてカウントする
        match &self.spell {
            ChunkSpell::DisplayableAscii(_) => 1,
            ChunkSpell::SingleChar(spell_string) | ChunkSpell::DoubleChar(spell_string) => {
                if spell_string.as_str() == "っ" {
                    1
                } else {
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get(spell_string.as_str())
                        .unwrap()
                        .iter()
                        .map(|key_stroke_str| key_stroke_str.chars().count())
                        .min()
                        .unwrap()
                }
            }
        }
    }

    // 制限を満たすなかで最小のキーストロークとなる候補を選択する
    // 同じキーストローク回数の候補が複数あった場合にはもともとの順番が早い方を選択する
    pub(crate) fn min_candidate(
        &self,
        chunk_head_striction: Option<KeyStrokeChar>,
    ) -> &ChunkKeyStrokeCandidate {
        assert!(self.key_stroke_candidates.is_some());

        let min_candidate = self
            .key_stroke_candidates
            .as_ref()
            .unwrap()
            .iter()
            .filter(|candidate| {
                if let Some(chunk_head_striction) = &chunk_head_striction {
                    &candidate.key_stroke_char_at_position(0) == chunk_head_striction
                } else {
                    true
                }
            })
            .reduce(|min_candidate, candidate| {
                if candidate.calc_key_stroke_count() < min_candidate.calc_key_stroke_count() {
                    candidate
                } else {
                    min_candidate
                }
            });

        assert!(min_candidate.is_some());

        min_candidate.as_ref().unwrap()
    }

    // このチャンクを打つのに必要な最小のキーストローク数を計算する
    pub fn calc_min_key_stroke_count(&self) -> usize {
        self.min_candidate(None).calc_key_stroke_count()
    }

    pub(crate) fn key_stroke_candidates_count(&self) -> Option<usize> {
        self.key_stroke_candidates.as_ref().map(|v| v.len())
    }

    /// チャンクをkey_stroke_count_striction回のキーストロークで終わるように制限する
    /// ex. 「し」というチャンクには「si」「shi」「ci」という候補があるがこれを1回のキーストロークに制限すると「s」「c」となる
    ///
    /// 最後のチャンクに使うことを想定している
    pub(crate) fn strict_key_stroke_count(&mut self, key_stroke_count_striction: NonZeroUsize) {
        // 制限によって必要キーストローク数が増えてはいけない
        assert!(key_stroke_count_striction.get() <= self.calc_min_key_stroke_count());

        let mut new_key_stroke_candidates = self.key_stroke_candidates.as_ref().unwrap().clone();

        new_key_stroke_candidates
            .iter_mut()
            // 変更するのは基本的には制限よりも長い候補のみでいい
            // 遅延確定候補は制限と同じタイプ数であっても通常の候補にする必要がある
            // 通常の候補にしないと制限だけタイプしても確定しなくなってしまう
            .filter(|candidate| {
                candidate.calc_key_stroke_count() > key_stroke_count_striction.get()
                    || candidate.is_delayed_confirmed_candidate()
            })
            .for_each(|candidate| candidate.strict_key_stroke_count(key_stroke_count_striction));

        // 制限の結果重複するキーストロークが生じる可能性があるので縮退させる
        let mut exists_in_candidates: HashSet<String> = HashSet::new();
        new_key_stroke_candidates.retain(|candidate| {
            let whole_key_stroke = candidate.whole_key_stroke().to_string();
            if exists_in_candidates.contains(&whole_key_stroke) {
                false
            } else {
                exists_in_candidates.insert(whole_key_stroke);
                true
            }
        });

        self.ideal_candidate
            .replace(new_key_stroke_candidates.get(0).unwrap().clone());

        self.key_stroke_candidates
            .replace(new_key_stroke_candidates);
    }

    // チャンクの候補を先頭キーストロークで制限する
    pub(crate) fn strict_chunk_head(&mut self, chunk_head_striction: KeyStrokeChar) {
        let key_stroke_candidates = self.key_stroke_candidates.as_mut().unwrap();

        key_stroke_candidates
            .retain(|candidate| candidate.key_stroke_char_at_position(0) == chunk_head_striction);
    }

    // 候補を減らす
    pub(crate) fn reduce_candidate(&mut self, retain_vector: &[bool]) {
        let mut index = 0;
        self.key_stroke_candidates.as_mut().unwrap().retain(|_| {
            let is_hit = *retain_vector.get(index).unwrap();
            index += 1;
            is_hit
        });
    }
}

// 綴りのみの不完全なチャンク列にキーストローク候補を追加する
pub fn append_key_stroke_to_chunks(chunks: &mut [Chunk]) {
    let mut next_chunk_spell: Option<ChunkSpell> = None;

    // このチャンクが「っ」としたときにキーストロークの連続によって表現できるキーストローク群
    // 次のチャンク先頭の子音などのキーストロークともいえる
    // ex. 次のチャンクが「た」だったときには [t] となる
    let mut key_strokes_can_represent_ltu_by_repeat: Vec<KeyStrokeChar> = Vec::new();

    // 遅延確定候補を確定できるキーストローク群
    // 次のチャンク先頭のキーストロークと同値
    let mut key_strokes_can_confirm_delayed_candidate: Vec<KeyStrokeChar> = Vec::new();

    // キーストローク候補は次のチャンクに依存するので後ろから走査する
    for chunk in chunks.iter_mut().rev() {
        assert!(chunk.key_stroke_candidates.is_none());

        let mut key_stroke_candidates = Vec::<ChunkKeyStrokeCandidate>::new();

        match &chunk.spell {
            // 表示可能なASCIIで構成されるチャンクならそのままキーストロークにする
            ChunkSpell::DisplayableAscii(spell_string) => {
                key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                    vec![String::from(spell_string.clone()).try_into().unwrap()],
                    None,
                    None,
                ));
            }
            ChunkSpell::SingleChar(spell_string) => match spell_string.chars().as_str() {
                "ん" => {
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get("ん")
                        .unwrap()
                        .iter()
                        // 「n」というキーストロークは次のチャンクによっては使えない
                        .filter(|key_stroke| match **key_stroke {
                            "n" => allow_single_n_as_key_stroke(&next_chunk_spell),
                            _ => true,
                        })
                        .for_each(|key_stroke| match *key_stroke {
                            "n" => key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![key_stroke.to_string().try_into().unwrap()],
                                None,
                                Some(DelayedConfirmedCandidateInfo::new(
                                    key_strokes_can_confirm_delayed_candidate.clone(),
                                )),
                            )),
                            _ => key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![key_stroke.to_string().try_into().unwrap()],
                                None,
                                None,
                            )),
                        });
                }
                // 「っ」は単独で打つ以外にも次のチャンクの子音で済ませる(「った」なら「tta」)ことができる
                "っ" => {
                    // 「ltu」「ltsu」「xtu」は任意の状況で次のチャンクへの制限なしに打てる
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get("っ")
                        .unwrap()
                        .iter()
                        .for_each(|key_stroke| {
                            key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![key_stroke.to_string().try_into().unwrap()],
                                None,
                                None,
                            ))
                        });

                    // 子音の連続で打つ場合には次のチャンクへの制限をする
                    key_strokes_can_represent_ltu_by_repeat
                        .iter()
                        .for_each(|key_stroke| match char::from(key_stroke.clone()) {
                            'l' | 'x' => {
                                key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                    vec![char::from(key_stroke.clone())
                                        .to_string()
                                        .try_into()
                                        .unwrap()],
                                    Some(key_stroke.clone()),
                                    // 次のチャンクへの制限があるときには遅延確定候補を確定できるのはその制限だけである
                                    Some(DelayedConfirmedCandidateInfo::new(
                                        key_strokes_can_confirm_delayed_candidate
                                            .iter()
                                            .filter(|ks| *ks == key_stroke)
                                            .map(|ks| ks.clone())
                                            .collect(),
                                    )),
                                ))
                            }
                            _ => key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![char::from(key_stroke.clone())
                                    .to_string()
                                    .try_into()
                                    .unwrap()],
                                Some(key_stroke.clone()),
                                None,
                            )),
                        });
                }
                _ => {
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get(spell_string.as_str())
                        .unwrap()
                        .iter()
                        .for_each(|key_stroke| {
                            key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![key_stroke.to_string().try_into().unwrap()],
                                None,
                                None,
                            ));
                        });
                }
            },
            // 2文字のチャンクはまとめて入力する場合と1文字ずつ入力する場合がある
            ChunkSpell::DoubleChar(spell_string) => {
                // まとめて入力できるキーストローク
                CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                    .get(spell_string.as_str())
                    .unwrap()
                    .iter()
                    .for_each(|key_stroke| {
                        key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                            vec![key_stroke.to_string().try_into().unwrap()],
                            None,
                            None,
                        ));
                    });

                let (first_spell_string, second_spell_string) = chunk.spell.split_double_char();

                // 1文字ずつのキーストローク
                CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                    .get(first_spell_string.as_str())
                    .unwrap()
                    .iter()
                    .for_each(|first_key_stroke| {
                        CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                            .get(second_spell_string.as_str())
                            .unwrap()
                            .iter()
                            .for_each(|second_key_stroke| {
                                key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                    vec![
                                        first_key_stroke.to_string().try_into().unwrap(),
                                        second_key_stroke.to_string().try_into().unwrap(),
                                    ],
                                    None,
                                    None,
                                ));
                            });
                    });
            }
        }

        // タイプ数が少ないキーストロークを第一候補として選択する
        key_stroke_candidates.sort_by(|a, b| {
            a.calc_key_stroke_count()
                .partial_cmp(&(b.calc_key_stroke_count()))
                .unwrap()
        });

        chunk.key_stroke_candidates.replace(key_stroke_candidates);

        next_chunk_spell.replace(chunk.spell.clone());

        key_strokes_can_confirm_delayed_candidate.clear();

        let mut already_pushed_key_strokes_can_confirm_delayed_candidate =
            HashSet::<KeyStrokeChar>::new();
        chunk
            .key_stroke_candidates
            .as_ref()
            .unwrap()
            .iter()
            .for_each(|key_stroke_candidate| {
                let first_char = key_stroke_candidate.key_stroke_char_at_position(0);
                if !already_pushed_key_strokes_can_confirm_delayed_candidate.contains(&first_char) {
                    already_pushed_key_strokes_can_confirm_delayed_candidate
                        .insert(first_char.clone());
                    key_strokes_can_confirm_delayed_candidate.push(first_char);
                }
            });

        // 次に処理するチャンク（逆順で処理しているので一つ前のチャンク）が「っ」だった場合に備えて子音などのキーストロークを構築する
        key_strokes_can_represent_ltu_by_repeat.clear();

        let mut already_pushed_key_strokes_can_represent_ltu_by_repeat =
            HashSet::<KeyStrokeChar>::new();
        match &chunk.spell {
            ChunkSpell::SingleChar(_) | ChunkSpell::DoubleChar(_) => chunk
                .key_stroke_candidates
                .as_ref()
                .unwrap()
                .iter()
                .for_each(|key_stroke_candidate| {
                    let head_key_stroke_char = key_stroke_candidate.key_stroke_char_at_position(0);

                    // 直後のチャンクの先頭が「n」を除く子音だった場合に「っ」を子音の連続で表すことができる
                    if head_key_stroke_char != 'a'
                        && head_key_stroke_char != 'i'
                        && head_key_stroke_char != 'u'
                        && head_key_stroke_char != 'e'
                        && head_key_stroke_char != 'o'
                        && head_key_stroke_char != 'n'
                    {
                        if !already_pushed_key_strokes_can_represent_ltu_by_repeat
                            .contains(&head_key_stroke_char)
                        {
                            already_pushed_key_strokes_can_represent_ltu_by_repeat
                                .insert(head_key_stroke_char.clone());
                            key_strokes_can_represent_ltu_by_repeat.push(head_key_stroke_char);
                        }
                    }
                }),
            // 直後のチャンクがASCIIだったら子音の連続で表すことはできない
            ChunkSpell::DisplayableAscii(_) => {}
        }
    }

    append_ideal_candidates_to_chunks(chunks);
}

/// 理想的なキーストローク候補をチャンク列に付与する
/// 候補が削減されていないことを前提とする
fn append_ideal_candidates_to_chunks(chunks: &mut [Chunk]) {
    // 本来なら理想的なキーストローク候補は全探索によって付与されるべきであるが計算量の観点から前のチャンクから貪欲に行うことで付与している
    // このことによって理想的ではないキーストローク候補が付与されてしまう可能性は以下の理由からないと言える
    //
    // チャンク列を処理していったときに次チャンクへの制限がない場合にはチャンク内で最短となる候補が理想的である
    // 次チャンクへの制限を持つ候補があるチャンクに遭遇したときにはそのチャンク内で最短となる候補が理想的であり
    // もしその候補が次チャンクへの制限があった場合には次のチャンクで選択の対象とする候補は制限によって削減する必要がある
    //
    // 次チャンクへの制限を持つ候補があるチャンクの次のチャンクでは制限によって削減される候補群（A）とそうでない候補（B）がある
    // このときAの最短キーストローク数がBの最短キーストローク数と比べて「制限を持つ候補によって短縮することのできるキーストローク数」分より大きい場合には前から貪欲にやってはならない
    //
    // XXX 現在の実装では「ん」には制限を持つ候補はない
    // しかし次チャンクへの制限を持つ候補がある「っ」「ん」の次のチャンクでAとB両方の候補を持つのはそれぞれ「い(AがiでBがyi)」「う(AがuでBがwuなど)」と「う」だけであり
    // これらのAとBの最短キーストローク数の差は制限を持つ候補による短縮分以下である

    let mut next_chunk_head_constraint: Option<KeyStrokeChar> = None;

    chunks.iter_mut().for_each(|chunk| {
        let ideal_candidate = chunk.min_candidate(next_chunk_head_constraint.clone());
        next_chunk_head_constraint = ideal_candidate.next_chunk_head_constraint.clone();

        chunk.ideal_candidate = Some(ideal_candidate.clone());
    });
}

// 「ん」のキーストロークとして「n」を使っていいか判定する
fn allow_single_n_as_key_stroke(next_chunk_spell: &Option<ChunkSpell>) -> bool {
    // 最後のチャンクの場合には許容しない
    if next_chunk_spell.is_none() {
        return false;
    }

    let next_chunk_spell = next_chunk_spell.as_ref().unwrap();

    // 次のチャンクがASCII・母音・な行・「ゃ」「ゅ」「ょ」を除くや行の場合には許容しない
    // XXX 「んう」を「nwu」で打つことができるIMEもあるのでどの規格に沿うのか一貫させておいたほうがよい
    match next_chunk_spell {
        ChunkSpell::DisplayableAscii(_) => false,
        ChunkSpell::SingleChar(spell_string) => !matches!(
            spell_string.as_str(),
            "あ" | "い"
                | "う"
                | "え"
                | "お"
                | "な"
                | "に"
                | "ぬ"
                | "ね"
                | "の"
                | "や"
                | "ゆ"
                | "よ"
                | "ん"
        ),
        ChunkSpell::DoubleChar(spell_string) => !matches!(
            spell_string.as_str(),
            "にゃ" | "にぃ" | "にゅ" | "にぇ" | "にょ"
        ),
    }
}

// チャンクに対応するキーストロークの候補
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkKeyStrokeCandidate {
    key_stroke_elements: Vec<KeyStrokeString>,
    // 次のチャンクの先頭キーストロークに制限があるケースがある
    next_chunk_head_constraint: Option<KeyStrokeChar>,
    delayed_confirmed_candidate_info: Option<DelayedConfirmedCandidateInfo>,
}

impl ChunkKeyStrokeCandidate {
    pub(crate) fn new(
        key_stroke_elements: Vec<KeyStrokeString>,
        next_chunk_head_constraint: Option<KeyStrokeChar>,
        delayed_confirmed_candidate_info: Option<DelayedConfirmedCandidateInfo>,
    ) -> Self {
        Self {
            key_stroke_elements,
            next_chunk_head_constraint,
            delayed_confirmed_candidate_info,
        }
    }

    pub(crate) fn next_chunk_head_constraint(&self) -> &Option<KeyStrokeChar> {
        &self.next_chunk_head_constraint
    }

    pub(crate) fn delayed_confirmed_candiate_info(&self) -> &Option<DelayedConfirmedCandidateInfo> {
        &self.delayed_confirmed_candidate_info
    }

    // この候補が複数文字チャンクを分けて打つ候補か
    // ex. 「きょ」というチャンクには「き」と「ょ」に分けて打つケースもある
    pub(crate) fn is_splitted(&self) -> bool {
        self.key_stroke_elements.len() == 2
    }

    pub(crate) fn is_delayed_confirmed_candidate(&self) -> bool {
        self.delayed_confirmed_candidate_info.is_some()
    }

    // 候補の中の特定のキーストロークがどちらの要素に属しているか
    // 基本的には0だが複数文字を個別で入力するような候補では1にもなりうる
    pub(crate) fn element_index_at_key_stroke_index(&self, key_stroke_index: usize) -> usize {
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

    // 候補の中の特定のキーストロークが要素の末尾であるか
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

    // キーストローク全体の文字列を生成する
    pub(crate) fn whole_key_stroke(&self) -> KeyStrokeString {
        let mut s = String::new();

        for key_stroke in &self.key_stroke_elements {
            s.push_str(key_stroke);
        }

        s.try_into().unwrap()
    }

    pub(crate) fn construct_key_stroke_element_count(&self) -> KeyStrokeElementCount {
        KeyStrokeElementCount::new(
            &(self
                .key_stroke_elements
                .iter()
                .map(|s| s.chars().count())
                .collect::<Vec<usize>>()),
        )
    }

    // この候補のキーストローク系列の特定のキーストロークを取り出す
    fn key_stroke_char_at_position(&self, position: usize) -> KeyStrokeChar {
        let whole_key_stroke = self.whole_key_stroke();

        assert!(position < whole_key_stroke.chars().count());

        whole_key_stroke
            .chars()
            .nth(position)
            .unwrap()
            .try_into()
            .unwrap()
    }

    // 何回のキーストロークで打つことができるか
    fn calc_key_stroke_count(&self) -> usize {
        let mut s = String::new();

        for key_stroke in &self.key_stroke_elements {
            s.push_str(key_stroke);
        }

        s.chars().count()
    }

    /// この候補のキーストローク回数をkey_stroke_count_striction回に制限する
    ///
    /// この候補の属するチャンクが最後のチャンクであることを想定している
    fn strict_key_stroke_count(&mut self, key_stroke_count_striction: NonZeroUsize) {
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

/// チャンク内の各綴り要素に対応するキーストローク数
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum KeyStrokeElementCount {
    Sigle(usize),
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

    pub(crate) fn whole_count(&self) -> usize {
        match self {
            Self::Sigle(c) => *c,
            Self::Double((c1, c2)) => c1 + c2,
        }
    }

    pub(crate) fn is_double(&self) -> bool {
        match self {
            Self::Double(_) => true,
            _ => false,
        }
    }

    // 理想的なキーストローク・キーストロークの位置を綴りの位置に変換する
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

    // 綴りの位置を理想的なキーストローク・キーストロークの位置に変換する
    pub(crate) fn convert_spell_delta_to_key_stroke_delta(&self, spell_delta: usize) -> usize {
        match self {
            Self::Sigle(c) => *c,
            Self::Double((c1, c2)) => {
                if spell_delta == 1 {
                    *c1
                } else if spell_delta == 2 {
                    c1 + c2
                } else {
                    unreachable!()
                }
            }
        }
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

    /// キーストローク位置が綴り要素の何番目に属するか
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

    pub(crate) fn key_stroke_count_offset(&self, spell_elements_index: usize) -> usize {
        match self {
            Self::Sigle(_) => 0,
            Self::Double((c1, c2)) => {
                if spell_elements_index == 0 {
                    0
                } else {
                    *c1
                }
            }
        }
    }
}

// 打ち終えてもチャンクを確定させてはいけない遅延確定候補の情報
// ex. 「ん」というチャンクには「n」・「nn」・「xn」という候補があるが「n」というキーストロークの後に「n」と打った場合にもこのチャンクのキーストロークとして有効である
// このようなケースにも適切にタイプを行うためには最初の「n」というキーストロークで完全に確定してはならない
// 一方最初の「n」の後のチャンク先頭として有効なキーストロークが与えられた場合にはチャンクを確定させて次のチャンクの先頭を打ったことにする必要がある
//
// 一般的には
// チャンク1のある候補Aのキーストローク列が他の候補Bの接頭辞となっている場合に候補Aはチャンク1の遅延確定候補となる
// 候補Aを確定するのは次のチャンク2の先頭キーストロークが打たれた時
// 図示すると
//
// チャンク1  -> チャンク2
// -----------------------
// 候補A(X)
//            -> 候補C(Z)
// 候補B(XxY)
//
// X,Y,Z := キーストローク列
// x := キーストローク
// ただしZの先頭キーストロークはxであってはいけない
// もしZの先頭キーストロークがxだとすると候補Bを打とうとしているのかチャンク2の候補Cを打とうとしているのか1回のキーストロークで確定せず2回以上遅延する必要がある
//
// ただし実際には必ず1回のキーストロークで確定する
// 遅延確定候補となるのは
// 1. チャンク「ん」の「n」という候補（「nn」が候補B）
// 2. チャンク「っ」を「l」「x」で打てるときの「l」「x」という候補（「ltu」「xtu」「ltsu」が候補B）
// に限られるが(TODO たぶんそうだが確証はもてない)
// 1. 次のチャンクが「n」で始まるときにはそもそも「n」は候補になることはない
// 2. チャンク「っ」を「l」や「x」で打てるときには次のチャンク先頭のキーストロークは「t」ではない
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct DelayedConfirmedCandidateInfo {
    // 次のチャンク先頭として有効なキーストローク列
    next_chunk_head: Vec<KeyStrokeChar>,
}

impl DelayedConfirmedCandidateInfo {
    pub(crate) fn new(next_chunk_head: Vec<KeyStrokeChar>) -> Self {
        Self { next_chunk_head }
    }

    /// 次のチャンク先頭のキーストロークとして与えられたキーストロークが有効かどうか
    pub(crate) fn is_valid_key_stroke(&self, key_stroke: KeyStrokeChar) -> bool {
        self.next_chunk_head.contains(&key_stroke)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{gen_candidate, gen_chunk, gen_unprocessed_chunk};

    #[test]
    fn append_key_stroke_to_chunks_1() {
        let mut chunks = vec![gen_unprocessed_chunk!("じょ"), gen_unprocessed_chunk!("ん")];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk!(
                    "じょ",
                    vec![
                        gen_candidate!(["jo"]),
                        gen_candidate!(["zyo"]),
                        gen_candidate!(["jyo"]),
                        gen_candidate!(["zi", "lyo"]),
                        gen_candidate!(["zi", "xyo"]),
                        gen_candidate!(["ji", "lyo"]),
                        gen_candidate!(["ji", "xyo"]),
                    ],
                    gen_candidate!(["jo"])
                ),
                gen_chunk!(
                    "ん",
                    vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])],
                    gen_candidate!(["nn"])
                )
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_2() {
        let mut chunks = vec![
            gen_unprocessed_chunk!("う"),
            gen_unprocessed_chunk!("っ"),
            gen_unprocessed_chunk!("う"),
        ];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"]),
                        gen_candidate!(["wu"]),
                        gen_candidate!(["whu"])
                    ],
                    gen_candidate!(["u"])
                ),
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["w"], 'w'),
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"])
                    ],
                    gen_candidate!(["w"], 'w')
                ),
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"]),
                        gen_candidate!(["wu"]),
                        gen_candidate!(["whu"])
                    ],
                    gen_candidate!(["wu"])
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_3() {
        let mut chunks = vec![
            gen_unprocessed_chunk!("か"),
            gen_unprocessed_chunk!("ん"),
            gen_unprocessed_chunk!("じ"),
        ];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk!(
                    "か",
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"])],
                    gen_candidate!(["ka"])
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], ['z', 'j']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['z', 'j'])
                ),
                gen_chunk!(
                    "じ",
                    vec![gen_candidate!(["zi"]), gen_candidate!(["ji"])],
                    gen_candidate!(["zi"])
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_4() {
        let mut chunks = vec![
            gen_unprocessed_chunk!("B"),
            gen_unprocessed_chunk!("i"),
            gen_unprocessed_chunk!("g"),
        ];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk!("B", vec![gen_candidate!(["B"])], gen_candidate!(["B"])),
                gen_chunk!("i", vec![gen_candidate!(["i"])], gen_candidate!(["i"])),
                gen_chunk!("g", vec![gen_candidate!(["g"])], gen_candidate!(["g"])),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_5() {
        let mut chunks = vec![gen_unprocessed_chunk!("っ"), gen_unprocessed_chunk!("っ")];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["l"], 'l', ['l']),
                        gen_candidate!(["x"], 'x', ['x']),
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"]),
                    ],
                    gen_candidate!(["l"], 'l', ['l'])
                ),
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"]),
                    ],
                    gen_candidate!(["ltu"])
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_6() {
        let mut chunks = vec![gen_unprocessed_chunk!("っ"), gen_unprocessed_chunk!("か")];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["k"], 'k'),
                        gen_candidate!(["c"], 'c'),
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"]),
                    ],
                    gen_candidate!(["k"], 'k')
                ),
                gen_chunk!(
                    "か",
                    vec![gen_candidate!(["ka"]), gen_candidate!(["ca"]),],
                    gen_candidate!(["ka"])
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_7() {
        let mut chunks = vec![
            gen_unprocessed_chunk!("い"),
            gen_unprocessed_chunk!("ん"),
            gen_unprocessed_chunk!("しょ"),
            gen_unprocessed_chunk!("う"),
        ];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk!(
                    "い",
                    vec![gen_candidate!(["i"]), gen_candidate!(["yi"]),],
                    gen_candidate!(["i"])
                ),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], ['s', 'c']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ],
                    gen_candidate!(["n"], ['s', 'c'])
                ),
                gen_chunk!(
                    "しょ",
                    vec![
                        gen_candidate!(["syo"]),
                        gen_candidate!(["sho"]),
                        gen_candidate!(["si", "lyo"]),
                        gen_candidate!(["si", "xyo"]),
                        gen_candidate!(["ci", "lyo"]),
                        gen_candidate!(["ci", "xyo"]),
                        gen_candidate!(["shi", "lyo"]),
                        gen_candidate!(["shi", "xyo"]),
                    ],
                    gen_candidate!(["syo"])
                ),
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"]),
                        gen_candidate!(["wu"]),
                        gen_candidate!(["whu"])
                    ],
                    gen_candidate!(["u"])
                ),
            ]
        );
    }

    #[test]
    fn strict_key_stroke_count_1() {
        let mut chunk = gen_chunk!(
            "じょ",
            vec![
                gen_candidate!(["jo"]),
                gen_candidate!(["zyo"]),
                gen_candidate!(["jyo"]),
                gen_candidate!(["zi", "lyo"]),
                gen_candidate!(["zi", "xyo"]),
                gen_candidate!(["ji", "lyo"]),
                gen_candidate!(["ji", "xyo"]),
            ],
            gen_candidate!(["jo"])
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk!(
                "じょ",
                vec![gen_candidate!(["j"]), gen_candidate!(["z"]),],
                gen_candidate!(["j"])
            )
        )
    }

    #[test]
    fn strict_key_stroke_count_2() {
        let mut chunk = gen_chunk!(
            "ん",
            vec![
                gen_candidate!(["n"], ['j', 'z']),
                gen_candidate!(["nn"]),
                gen_candidate!(["xn"]),
            ],
            gen_candidate!(["n"], ['j', 'z'])
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk!(
                "ん",
                vec![gen_candidate!(["n"]), gen_candidate!(["x"])],
                gen_candidate!(["n"])
            )
        )
    }

    #[test]
    fn is_element_end_at_key_stroke_index_1() {
        let c = gen_candidate!(["ki", "xyo"]);

        assert!(!c.is_element_end_at_key_stroke_index(0));
        assert!(c.is_element_end_at_key_stroke_index(1));
        assert!(!c.is_element_end_at_key_stroke_index(2));
        assert!(!c.is_element_end_at_key_stroke_index(3));
        assert!(c.is_element_end_at_key_stroke_index(4));
    }
}
