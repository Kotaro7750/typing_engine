use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::key_stroke::{KeyStrokeChar, KeyStrokeString};
use crate::spell::SpellString;

pub(crate) mod confirmed;
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
}

impl Chunk {
    pub fn new(
        spell: SpellString,
        key_stroke_candidates: Option<Vec<ChunkKeyStrokeCandidate>>,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            key_stroke_candidates,
        }
    }

    pub(crate) fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    pub(crate) fn key_stroke_candidates(&self) -> &Option<Vec<ChunkKeyStrokeCandidate>> {
        &self.key_stroke_candidates
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
            // 変更するのは制限よりも長い候補のみでいい
            .filter(|candidate| {
                candidate.calc_key_stroke_count() > key_stroke_count_striction.get()
            })
            .for_each(|candidate| {
                candidate.strict_key_stroke_count(key_stroke_count_striction.get())
            });

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
    let mut key_strokes_can_represent_ltu_by_repeat: HashSet<KeyStrokeChar> = HashSet::new();

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
                            'l' | 'x' => key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![char::from(key_stroke.clone())
                                    .to_string()
                                    .try_into()
                                    .unwrap()],
                                Some(key_stroke.clone()),
                                Some(DelayedConfirmedCandidateInfo::new(
                                    key_strokes_can_confirm_delayed_candidate.clone(),
                                )),
                            )),
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

        key_strokes_can_represent_ltu_by_repeat.clear();
        chunk
            .key_stroke_candidates
            .as_ref()
            .unwrap()
            .iter()
            .for_each(|key_stroke_candidate| {
                let first_char = key_stroke_candidate.key_stroke_char_at_position(0);
                key_strokes_can_confirm_delayed_candidate.push(first_char);
            });

        // 次に処理するチャンク（逆順で処理しているので一つ前のチャンク）が「っ」だった場合に備えて子音などのキーストロークを構築する
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
                        key_strokes_can_represent_ltu_by_repeat.insert(head_key_stroke_char);
                    }
                }),
            // 直後のチャンクがASCIIだったら子音の連続で表すことはできない
            ChunkSpell::DisplayableAscii(_) => {}
        }
    }
}

// 「ん」のキーストロークとして「n」を使っていいか判定する
fn allow_single_n_as_key_stroke(next_chunk_spell: &Option<ChunkSpell>) -> bool {
    // 最後のチャンクの場合には許容しない
    if next_chunk_spell.is_none() {
        return false;
    }

    let next_chunk_spell = next_chunk_spell.as_ref().unwrap();

    // 次のチャンクがASCII・母音・な行・「ゃ」「ゅ」「ょ」を除くや行の場合には許容しない
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
        ChunkSpell::DoubleChar(spell_string) => matches!(
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

    // この候補が複数文字チャンクを分けて打つ候補か
    // ex. 「きょ」というチャンクには「き」と「ょ」に分けて打つケースもある
    pub(crate) fn is_splitted(&self) -> bool {
        self.key_stroke_elements.len() == 2
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

    // キーストローク全体の文字列を生成する
    pub(crate) fn whole_key_stroke(&self) -> KeyStrokeString {
        let mut s = String::new();

        for key_stroke in &self.key_stroke_elements {
            s.push_str(key_stroke);
        }

        s.try_into().unwrap()
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

    // この候補のキーストローク回数をstrict_count回に制限する
    // この候補の属するチャンクが最後のチャンクであることを想定している
    fn strict_key_stroke_count(&mut self, strict_count: usize) {
        assert!(strict_count < self.calc_key_stroke_count());

        let mut new_key_stroke_elements: Vec<KeyStrokeString> = Vec::new();

        let mut count = 0;
        for key_stroke_element in &self.key_stroke_elements {
            let count_of_element = key_stroke_element.chars().count();

            if count + count_of_element >= strict_count {
                let count_after_truncate = strict_count - count;
                assert!(count_after_truncate > 0);

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
                    ]
                ),
                gen_chunk!("ん", vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])])
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
                    ]
                ),
                gen_chunk!(
                    "っ",
                    vec![
                        gen_candidate!(["w"], 'w'),
                        gen_candidate!(["ltu"]),
                        gen_candidate!(["xtu"]),
                        gen_candidate!(["ltsu"])
                    ]
                ),
                gen_chunk!(
                    "う",
                    vec![
                        gen_candidate!(["u"]),
                        gen_candidate!(["wu"]),
                        gen_candidate!(["whu"])
                    ]
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
                gen_chunk!("か", vec![gen_candidate!(["ka"]), gen_candidate!(["ca"])]),
                gen_chunk!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], ['z', 'j']),
                        gen_candidate!(["nn"]),
                        gen_candidate!(["xn"])
                    ]
                ),
                gen_chunk!("じ", vec![gen_candidate!(["zi"]), gen_candidate!(["ji"])]),
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
                gen_chunk!("B", vec![gen_candidate!(["B"])]),
                gen_chunk!("i", vec![gen_candidate!(["i"])]),
                gen_chunk!("g", vec![gen_candidate!(["g"])]),
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
            ]
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk!("じょ", vec![gen_candidate!(["j"]), gen_candidate!(["z"]),])
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
            ]
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk!("ん", vec![gen_candidate!(["n"]), gen_candidate!(["x"])])
        )
    }
}
