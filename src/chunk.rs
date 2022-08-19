use std::collections::HashSet;

use crate::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::key_stroke::{KeyStrokeChar, KeyStrokeString};
use crate::spell::SpellString;

pub(crate) mod typed;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum ChunkSpell {
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
                    .nth(0)
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

    pub(crate) fn key_stroke_candidates(&mut self) -> &mut Option<Vec<ChunkKeyStrokeCandidate>> {
        &mut self.key_stroke_candidates
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

    // このチャンクを打つのに必要な最小のキーストローク数を計算する
    pub fn calc_min_key_stroke_count(&self) -> usize {
        assert!(self.key_stroke_candidates.is_some());

        self.key_stroke_candidates
            .as_ref()
            .unwrap()
            .iter()
            .map(|candidate| candidate.calc_key_stroke_count())
            .min()
            .unwrap()
    }

    pub(crate) fn key_stroke_candidates_count(&self) -> Option<usize> {
        match &self.key_stroke_candidates {
            Some(v) => Some(v.len()),
            None => None,
        }
    }

    // チャンクをcount_striction回のキーストロークで終わるように制限する
    // 最後のチャンクに使うことを想定している
    // ex. 「し」というチャンクには「si」「shi」「ci」という候補があるがこれを1回のキーストロークに制限すると「s」「c」となる
    pub fn strict_key_stroke_count(&mut self, count_striction: usize) {
        // 制限によってキーストロークが0回になったり必要キーストローク数が増えてはいけない
        assert!(count_striction != 0 && count_striction <= self.calc_min_key_stroke_count());

        let mut new_key_stroke_candidates = self.key_stroke_candidates.as_ref().unwrap().clone();

        new_key_stroke_candidates
            .iter_mut()
            // 変更するのは制限よりも長い候補のみでいい
            .filter(|candidate| candidate.calc_key_stroke_count() > count_striction)
            .for_each(|candidate| candidate.strict_key_stroke_count(count_striction));

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
        let key_stroke_candidates = self.key_stroke_candidates().as_mut().unwrap();

        key_stroke_candidates
            .retain(|candidate| candidate.key_stroke_char_at_position(0) == chunk_head_striction);
    }
}

// 綴りのみの不完全なチャンク列にキーストローク候補を追加する
pub fn append_key_stroke_to_chunks(chunks: &mut Vec<Chunk>) {
    let mut next_chunk_spell: Option<ChunkSpell> = None;

    // このチャンクが「っ」としたときにキーストロークの連続によって表現できるキーストローク群
    // 次のチャンク先頭の子音などのキーストロークともいえる
    // ex. 次のチャンクが「た」だったときには [t] となる
    let mut key_strokes_can_represent_ltu_by_repeat: HashSet<KeyStrokeChar> = HashSet::new();

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
                        .map(|e| *e)
                        .for_each(|key_stroke| {
                            key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![key_stroke.to_string().try_into().unwrap()],
                                None,
                            ));
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
                            ));
                        });

                    // 子音の連続で打つ場合には次のチャンクへの制限をする
                    key_strokes_can_represent_ltu_by_repeat
                        .iter()
                        .for_each(|key_stroke| {
                            key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![char::from(key_stroke.clone())
                                    .to_string()
                                    .try_into()
                                    .unwrap()],
                                Some(key_stroke.clone()),
                            ))
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

        key_strokes_can_represent_ltu_by_repeat.clear();
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
        ChunkSpell::SingleChar(spell_string) => match spell_string.as_str() {
            "あ" | "い" | "う" | "え" | "お" | "な" | "に" | "ぬ" | "ね" | "の" | "や" | "ゆ"
            | "よ" | "ん" => false,
            _ => true,
        },
        ChunkSpell::DoubleChar(spell_string) => match spell_string.as_str() {
            "にゃ" | "にぃ" | "にゅ" | "にぇ" | "にょ" => false,
            _ => true,
        },
    }
}

// チャンクに対応するキーストロークの候補
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkKeyStrokeCandidate {
    key_stroke_elements: Vec<KeyStrokeString>,
    // 次のチャンクの先頭キーストロークに制限があるケースがある
    next_chunk_head_constraint: Option<KeyStrokeChar>,
}

impl ChunkKeyStrokeCandidate {
    pub fn new(
        key_stroke_elements: Vec<KeyStrokeString>,
        next_chunk_head_constraint: Option<KeyStrokeChar>,
    ) -> Self {
        Self {
            key_stroke_elements,
            next_chunk_head_constraint,
        }
    }

    // キーストローク全体の文字列を生成する
    fn whole_key_stroke(&self) -> KeyStrokeString {
        let mut s = String::new();

        for key_stroke in &self.key_stroke_elements {
            s.push_str(key_stroke);
        }

        s.try_into().unwrap()
    }

    // この候補のキーストローク系列の特定のキーストロークを取り出す
    fn key_stroke_char_at_position(&self, position: usize) -> KeyStrokeChar {
        let whole_key_stroke = self.whole_key_stroke();

        assert!(position <= whole_key_stroke.chars().count() - 1);

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
                        gen_candidate!(["n"]),
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

        chunk.strict_key_stroke_count(1);

        assert_eq!(
            chunk,
            gen_chunk!("じょ", vec![gen_candidate!(["j"]), gen_candidate!(["z"]),])
        )
    }
}
