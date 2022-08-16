use rand::random;
use std::num::NonZeroUsize;

use crate::{
    chunk::{append_key_stroke_to_chunks, Chunk},
    spell::SpellString,
    vocabulary::VocabularyEntry,
};

// 問題文を構成する各語彙の間に入れる語彙
pub enum VocabularySeparator {
    None,
    WhiteSpace,
    Vocabulary(VocabularyEntry),
}

impl VocabularySeparator {
    // 語彙の区切り語彙がないかどうか
    fn is_none(&self) -> bool {
        match self {
            Self::None => true,
            _ => false,
        }
    }

    fn generate_separator_vocabulary(&self) -> VocabularyEntry {
        match self {
            Self::Vocabulary(v) => v.clone(),
            Self::WhiteSpace => {
                VocabularyEntry::new(" ".to_string(), vec![" ".to_string().try_into().unwrap()])
                    .unwrap()
            }
            Self::None => unreachable!("this function cannot be called when self is None"),
        }
    }
}

// 問題文を構成する語彙を語彙リストからどのような順番で選ぶか
pub enum VocabularyOrder<'order_function> {
    Random,
    InOrder,
    Arbitrary(&'order_function dyn Fn(&Option<usize>, &Vec<VocabularyEntry>) -> usize),
}

impl<'order_function> VocabularyOrder<'order_function> {
    // 前回の語彙のインデックスと語彙リストから次使う語彙のインデックスを生成する
    fn next_vocabulary_entry_index(
        &self,
        prev_index: &Option<usize>,
        vocabulary_entries: &Vec<VocabularyEntry>,
    ) -> usize {
        match self {
            Self::Random => random::<usize>() % vocabulary_entries.len(),
            Self::InOrder => {
                if prev_index.is_some() {
                    (prev_index.unwrap() + 1) % vocabulary_entries.len()
                } else {
                    0
                }
            }
            Self::Arbitrary(func) => func(prev_index, vocabulary_entries),
        }
    }
}

pub struct QueryRequest<'vocabulary, 'order_function> {
    vocabulary_entries: &'vocabulary Vec<VocabularyEntry>,
    key_stroke_threshold: NonZeroUsize,
    vocabulary_separator: VocabularySeparator,
    vocabulary_order: VocabularyOrder<'order_function>,
}

impl<'vocabulary, 'order_function> QueryRequest<'vocabulary, 'order_function> {
    pub fn new(
        vocabulary_entries: &'vocabulary Vec<VocabularyEntry>,
        key_stroke_threshold: NonZeroUsize,
        vocabulary_separator: VocabularySeparator,
        vocabulary_order: VocabularyOrder<'order_function>,
    ) -> Self {
        Self {
            vocabulary_entries,
            key_stroke_threshold,
            vocabulary_separator,
            vocabulary_order,
        }
    }

    pub fn construct_query(&self) -> Query {
        let mut query_chunks = Vec::<Chunk>::new();
        let mut query_view = String::new();
        let mut query_spell: SpellString = String::new().try_into().unwrap();
        let mut view_position_of_spell: Vec<usize> = vec![];

        // 語彙リストから選んだ語彙の区切りとして使う語彙
        let separator_vocabulary = if self.vocabulary_separator.is_none() {
            None
        } else {
            Some(self.vocabulary_separator.generate_separator_vocabulary())
        };

        let mut min_key_stroke_count: usize = 0;
        let mut prev_vocabulary_index: Option<usize> = None;
        let mut is_prev_vocabulary: bool = false;

        // 要求キーストローク回数を満たすまで以下を繰り返す
        // 1. 語彙リストから語彙を選ぶ
        // 2. 語彙をパースしてチャンク列を構成する（キーストロークの付与はまだしない）
        // 3. チャンク列に語彙のチャンク列を追加する
        // 4. 表示用の文字列・綴りを構築する
        while min_key_stroke_count < self.key_stroke_threshold.get() {
            // 1
            // 直前に追加した語彙が語彙リストから選んだ語彙ではなかったり語彙区切りがない場合のみ語彙リストから語彙を選択する
            let vocabulary_entry = if is_prev_vocabulary && separator_vocabulary.is_some() {
                is_prev_vocabulary = false;
                separator_vocabulary.as_ref().unwrap()
            } else {
                is_prev_vocabulary = true;

                let vocabulary_index = self
                    .vocabulary_order
                    .next_vocabulary_entry_index(&prev_vocabulary_index, self.vocabulary_entries);

                prev_vocabulary_index.replace(vocabulary_index);

                self.vocabulary_entries.get(vocabulary_index).unwrap()
            };

            // 2
            // 語彙区切りによっては語彙ごとにキーストロークを付与してはいけないケースがあるためまだ付与しない
            // 例えば語彙区切りがない場合には語彙の末尾のキーストロークは次の語彙の先頭チャンクに依存する
            let chunks = vocabulary_entry.construct_chunks();

            // 3
            for chunk in chunks {
                // チャンクのキーストロークの取りうる最小値なのでもし大きかったとしても後で制限する際に削られる
                min_key_stroke_count += chunk.estimate_min_key_stroke_count();

                query_chunks.push(chunk);

                if min_key_stroke_count >= self.key_stroke_threshold.get() {
                    break;
                }
            }

            // 4
            vocabulary_entry
                .spells()
                .iter()
                .enumerate()
                .for_each(|(view_i, spell)| {
                    spell.chars().for_each(|_| {
                        view_position_of_spell.push(query_view.chars().count() + view_i);
                    });
                });

            query_view.push_str(vocabulary_entry.view());
            query_spell.push_str(vocabulary_entry.construct_spell_string().as_str());
        }

        // 全ての語彙や語彙区切りが確定してからキーストロークを付与する
        append_key_stroke_to_chunks(&mut query_chunks);

        // キーストロークを付与したので推測ではない実際のキーストローク回数が分かる
        let mut actual_key_stroke_count: usize = 0;
        query_chunks.retain(|chunk| {
            // キーストロークの閾値を最初に超えたチャンクの次のチャンクから取り除く
            if actual_key_stroke_count >= self.key_stroke_threshold.get() {
                false
            } else {
                // 最終的には採用するチャンクの累積キーストローク回数になる
                actual_key_stroke_count += chunk.calc_min_key_stroke_count();
                true
            }
        });

        // 最後のチャンクのみ制限を行う
        let last_chunk = query_chunks.last_mut().unwrap();
        let over_key_stroke_count = actual_key_stroke_count - self.key_stroke_threshold.get();
        last_chunk.strict_key_stroke_count(
            last_chunk.calc_min_key_stroke_count() - over_key_stroke_count,
        );

        Query::new(
            query_view,
            query_spell,
            view_position_of_spell,
            query_chunks,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    view: String,
    spell: SpellString,
    // i番目の要素は綴りのi番目の文字が表示文字列の何番目の文字のものなのかを示している
    view_position_of_spell: Vec<usize>,
    chunks: Vec<Chunk>,
}

impl Query {
    fn new(
        view: String,
        spell: SpellString,
        view_position_of_spell: Vec<usize>,
        chunks: Vec<Chunk>,
    ) -> Self {
        assert_eq!(spell.chars().count(), view_position_of_spell.len());

        Self {
            view,
            spell,
            view_position_of_spell,
            chunks,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{gen_candidate, gen_chunk, gen_vocabulary_entry};

    #[test]
    fn construct_query_1() {
        let vocabularies = vec![gen_vocabulary_entry!("イオン", ["い", "お", "ん"])];

        let qr = QueryRequest::new(
            &vocabularies,
            NonZeroUsize::new(5).unwrap(),
            VocabularySeparator::WhiteSpace,
            VocabularyOrder::InOrder,
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                "イオン ".to_string(),
                "いおん ".to_string().try_into().unwrap(),
                vec![0, 1, 2, 3],
                vec![
                    gen_chunk!("い", vec![gen_candidate!(["i"]), gen_candidate!(["yi"])]),
                    gen_chunk!("お", vec![gen_candidate!(["o"])]),
                    gen_chunk!("ん", vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])]),
                    gen_chunk!(" ", vec![gen_candidate!([" "])]),
                ]
            )
        );
    }

    #[test]
    fn construct_query_2() {
        let vocabularies = vec![gen_vocabulary_entry!("イオン", ["い", "お", "ん"])];

        let qr = QueryRequest::new(
            &vocabularies,
            NonZeroUsize::new(5).unwrap(),
            VocabularySeparator::None,
            VocabularyOrder::InOrder,
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                "イオンイオン".to_string(),
                "いおんいおん".to_string().try_into().unwrap(),
                vec![0, 1, 2, 3, 4, 5],
                vec![
                    gen_chunk!("い", vec![gen_candidate!(["i"]), gen_candidate!(["yi"])]),
                    gen_chunk!("お", vec![gen_candidate!(["o"])]),
                    gen_chunk!("ん", vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])]),
                    gen_chunk!("い", vec![gen_candidate!(["i"]), gen_candidate!(["y"])]),
                ]
            )
        );
    }

    #[test]
    fn construct_query_3() {
        let vocabularies = vec![
            gen_vocabulary_entry!("イオン", ["い", "お", "ん"]),
            gen_vocabulary_entry!("買っ", ["か", "っ"]),
            gen_vocabulary_entry!("た", ["た"]),
        ];

        let qr = QueryRequest::new(
            &vocabularies,
            NonZeroUsize::new(10).unwrap(),
            VocabularySeparator::None,
            VocabularyOrder::InOrder,
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                "イオン買ったイオン".to_string(),
                "いおんかったいおん".to_string().try_into().unwrap(),
                vec![0, 1, 2, 3, 4, 5, 6, 7, 8],
                vec![
                    gen_chunk!("い", vec![gen_candidate!(["i"]), gen_candidate!(["yi"])]),
                    gen_chunk!("お", vec![gen_candidate!(["o"])]),
                    gen_chunk!(
                        "ん",
                        vec![
                            gen_candidate!(["n"]),
                            gen_candidate!(["nn"]),
                            gen_candidate!(["xn"])
                        ]
                    ),
                    gen_chunk!("か", vec![gen_candidate!(["ka"]), gen_candidate!(["ca"])]),
                    gen_chunk!(
                        "っ",
                        vec![
                            gen_candidate!(["t"], 't'),
                            gen_candidate!(["ltu"]),
                            gen_candidate!(["xtu"]),
                            gen_candidate!(["ltsu"]),
                        ]
                    ),
                    gen_chunk!("た", vec![gen_candidate!(["ta"])]),
                    gen_chunk!("い", vec![gen_candidate!(["i"]), gen_candidate!(["yi"])]),
                    gen_chunk!("お", vec![gen_candidate!(["o"])]),
                ]
            )
        );
    }
}
