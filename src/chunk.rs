use std::collections::HashSet;

use crate::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::key_stroke::{KeyStrokeChar, KeyStrokeString};
use crate::spell::SpellString;

#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, PartialEq, Eq, Clone)]
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
}

// 綴りのみの不完全なチャンク列にキーストローク候補を追加する
fn append_key_stroke_to_chunks(chunks: &mut Vec<Chunk>) {
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
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ChunkKeyStrokeCandidate {
    key_stroke_elements: Vec<KeyStrokeString>,
    // 次のチャンクの先頭キーストロークに制限があるケースがある
    next_chunk_head_constraint: Option<KeyStrokeChar>,
}

impl ChunkKeyStrokeCandidate {
    fn new(
        key_stroke_elements: Vec<KeyStrokeString>,
        next_chunk_head_constraint: Option<KeyStrokeChar>,
    ) -> Self {
        Self {
            key_stroke_elements,
            next_chunk_head_constraint,
        }
    }

    // この候補のキーストローク系列の特定のキーストロークを取り出す
    fn key_stroke_char_at_position(&self, position: usize) -> KeyStrokeChar {
        let mut s = String::new();

        for key_stroke in &self.key_stroke_elements {
            s.push_str(key_stroke);
        }

        assert!(position <= s.chars().count() - 1);

        s.chars().nth(position).unwrap().try_into().unwrap()
    }

    // 何回のキーストロークで打つことができるか
    fn calc_key_stroke_count(&self) -> usize {
        let mut s = String::new();

        for key_stroke in &self.key_stroke_elements {
            s.push_str(key_stroke);
        }

        s.chars().count()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! unprocessed_chunk {
        ($chunk_spell:literal) => {
            Chunk::new($chunk_spell.to_string().try_into().unwrap(), None)
        };
    }

    macro_rules! candidate {
        ([$($key_stroke:literal),*]$(, $constraint:literal)?) => {
            {
                let _constraint: Option<KeyStrokeChar> = None;
                $(let _constraint = Some($constraint.try_into().unwrap());)?
                ChunkKeyStrokeCandidate::new(vec![$($key_stroke.to_string().try_into().unwrap()),*],_constraint)
            }
        };
    }

    macro_rules! chunk {
        (
            $chunk_spell:literal,
            $key_stroke_candidates:expr
        ) => {
            Chunk::new(
                $chunk_spell.to_string().try_into().unwrap(),
                Some($key_stroke_candidates),
            )
        };
    }

    #[test]
    fn append_key_stroke_to_chunks_1() {
        let mut chunks = vec![unprocessed_chunk!("じょ"), unprocessed_chunk!("ん")];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                chunk!(
                    "じょ",
                    vec![
                        candidate!(["jo"]),
                        candidate!(["zyo"]),
                        candidate!(["jyo"]),
                        candidate!(["zi", "lyo"]),
                        candidate!(["zi", "xyo"]),
                        candidate!(["ji", "lyo"]),
                        candidate!(["ji", "xyo"]),
                    ]
                ),
                chunk!("ん", vec![candidate!(["nn"]), candidate!(["xn"])])
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_2() {
        let mut chunks = vec![
            unprocessed_chunk!("う"),
            unprocessed_chunk!("っ"),
            unprocessed_chunk!("う"),
        ];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                chunk!(
                    "う",
                    vec![candidate!(["u"]), candidate!(["wu"]), candidate!(["whu"])]
                ),
                chunk!(
                    "っ",
                    vec![
                        candidate!(["w"], 'w'),
                        candidate!(["ltu"]),
                        candidate!(["xtu"]),
                        candidate!(["ltsu"])
                    ]
                ),
                chunk!(
                    "う",
                    vec![candidate!(["u"]), candidate!(["wu"]), candidate!(["whu"])]
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_3() {
        let mut chunks = vec![
            unprocessed_chunk!("か"),
            unprocessed_chunk!("ん"),
            unprocessed_chunk!("じ"),
        ];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                chunk!("か", vec![candidate!(["ka"]), candidate!(["ca"])]),
                chunk!(
                    "ん",
                    vec![candidate!(["n"]), candidate!(["nn"]), candidate!(["xn"])]
                ),
                chunk!("じ", vec![candidate!(["zi"]), candidate!(["ji"])]),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_4() {
        let mut chunks = vec![
            unprocessed_chunk!("B"),
            unprocessed_chunk!("i"),
            unprocessed_chunk!("g"),
        ];

        append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                chunk!("B", vec![candidate!(["B"])]),
                chunk!("i", vec![candidate!(["i"])]),
                chunk!("g", vec![candidate!(["g"])]),
            ]
        );
    }
}
