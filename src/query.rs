use rand::random;
use std::num::NonZeroUsize;

use crate::{
    chunk::{append_key_stroke_to_chunks, Chunk},
    vocabulary::{VocabularyEntry, VocabularyInfo},
};

// クエリを構成する語彙の量指定
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VocabularyQuantifier {
    // キーストローク数での指定
    KeyStroke(NonZeroUsize),
    // 語彙数での指定
    Vocabulary(NonZeroUsize),
}

// 問題文を構成する各語彙の間に入れる語彙
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
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
#[derive(Clone)]
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

#[derive(Clone)]
pub struct QueryRequest<'vocabulary, 'order_function> {
    vocabulary_entries: &'vocabulary Vec<VocabularyEntry>,
    vocabulary_quantifier: VocabularyQuantifier,
    vocabulary_separator: VocabularySeparator,
    vocabulary_order: VocabularyOrder<'order_function>,
}

impl<'vocabulary, 'order_function> QueryRequest<'vocabulary, 'order_function> {
    pub fn new(
        vocabulary_entries: &'vocabulary Vec<VocabularyEntry>,
        vocabulary_quantifier: VocabularyQuantifier,
        vocabulary_separator: VocabularySeparator,
        vocabulary_order: VocabularyOrder<'order_function>,
    ) -> Self {
        Self {
            vocabulary_entries,
            vocabulary_quantifier,
            vocabulary_separator,
            vocabulary_order,
        }
    }

    pub fn construct_query(&self) -> Query {
        // 語彙リストから選んだ語彙の区切りとして使う語彙
        let separator_vocabulary = if self.vocabulary_separator.is_none() {
            None
        } else {
            Some(self.vocabulary_separator.generate_separator_vocabulary())
        };

        let next_vocabulary_generator = NextVocabularyGenerator::new(
            self.vocabulary_entries,
            &separator_vocabulary,
            &self.vocabulary_order,
        );

        match self.vocabulary_quantifier {
            VocabularyQuantifier::KeyStroke(key_stroke_threshold) => {
                Self::construct_query_with_key_stroke_striction(
                    key_stroke_threshold,
                    next_vocabulary_generator,
                )
            }
            VocabularyQuantifier::Vocabulary(vocabulary_count) => {
                Self::construct_query_with_vocabulary_count(
                    vocabulary_count,
                    next_vocabulary_generator,
                )
            }
        }
    }

    fn construct_query_with_key_stroke_striction(
        key_stroke_threshold: NonZeroUsize,
        mut next_vocabulary_generator: NextVocabularyGenerator,
    ) -> Query {
        let mut query_chunks = Vec::<Chunk>::new();
        let mut query_vocabulary_infos = Vec::<VocabularyInfo>::new();

        let mut min_key_stroke_count: usize = 0;

        // 要求キーストローク回数を満たすまで以下を繰り返す
        // 1. 語彙リストから語彙を選ぶ
        // 2. 語彙をパースしてチャンク列を構成する（キーストロークの付与はまだしない）
        // 3. チャンク列に語彙のチャンク列を追加する
        //
        // キーストロークによる制限は最後にまとめて行う
        while min_key_stroke_count < key_stroke_threshold.get() {
            // 1
            let vocabulary_entry = next_vocabulary_generator.next().unwrap();

            // 2
            // 語彙区切りによっては語彙ごとにキーストロークを付与してはいけないケースがあるためまだ付与しない
            // 例えば語彙区切りがない場合には語彙の末尾のキーストロークは次の語彙の先頭チャンクに依存する
            let chunks = vocabulary_entry.construct_chunks();

            let chunk_count = chunks.len().try_into().unwrap();
            query_vocabulary_infos.push(vocabulary_entry.construct_vocabulary_info(chunk_count));

            // 3
            for chunk in chunks {
                // チャンクのキーストロークの取りうる最小値なのでもし大きかったとしても後で制限する際に削られる
                min_key_stroke_count += chunk.estimate_min_key_stroke_count();

                query_chunks.push(chunk);
            }
        }

        // 全ての語彙や語彙区切りが確定してからキーストロークを付与する
        append_key_stroke_to_chunks(&mut query_chunks);

        // キーストロークを付与したので推測ではない実際のキーストローク回数が分かる
        let mut actual_key_stroke_count: usize = 0;
        query_chunks.retain(|chunk| {
            // キーストロークの閾値を最初に超えたチャンクの次のチャンクから取り除く
            if actual_key_stroke_count >= key_stroke_threshold.get() {
                false
            } else {
                // 最終的には採用するチャンクの累積キーストローク回数になる
                actual_key_stroke_count += chunk.calc_min_key_stroke_count();
                true
            }
        });

        // 最後のチャンクのみ制限を行う
        let last_chunk = query_chunks.last_mut().unwrap();
        let over_key_stroke_count = actual_key_stroke_count - key_stroke_threshold.get();
        last_chunk.strict_key_stroke_count(
            last_chunk.calc_min_key_stroke_count() - over_key_stroke_count,
        );

        // チャンクの削除によって語彙も削除される可能性がある
        let total_chunk_count = query_chunks.len();
        let mut chunk_count = 0;
        let mut chunk_count_over = 0;

        query_vocabulary_infos.retain(|vocabulary_info| {
            if chunk_count >= total_chunk_count {
                false
            } else {
                chunk_count += vocabulary_info.chunk_count().get();
                if chunk_count >= total_chunk_count {
                    chunk_count_over = chunk_count - total_chunk_count;
                }
                true
            }
        });

        // 最後の語彙はチャンク削除によってチャンク数が減っている可能性がある
        let last_vacabulary_info = query_vocabulary_infos.last_mut().unwrap();
        last_vacabulary_info.reset_chunk_count(
            (last_vacabulary_info.chunk_count().get() - chunk_count_over)
                .try_into()
                .unwrap(),
        );

        Query::new(query_vocabulary_infos, query_chunks)
    }

    fn construct_query_with_vocabulary_count(
        vocabulary_count: NonZeroUsize,
        mut next_vocabulary_generator: NextVocabularyGenerator,
    ) -> Query {
        let mut query_chunks = Vec::<Chunk>::new();
        let mut query_vocabulary_infos = Vec::<VocabularyInfo>::new();

        // 要求語彙数を満たすまで以下を繰り返す
        // 1. 語彙リストから語彙を選ぶ
        // 2. 語彙をパースしてチャンク列を構成する（キーストロークの付与はまだしない）
        // 3. チャンク列に語彙のチャンク列を追加する
        let mut current_vocabulary_count = 0;
        while current_vocabulary_count < vocabulary_count.get() {
            // 1
            let vocabulary_entry = next_vocabulary_generator.next().unwrap();

            // 2
            // 語彙区切りによっては語彙ごとにキーストロークを付与してはいけないケースがあるためまだ付与しない
            // 例えば語彙区切りがない場合には語彙の末尾のキーストロークは次の語彙の先頭チャンクに依存する
            let chunks = vocabulary_entry.construct_chunks();

            let chunk_count = chunks.len().try_into().unwrap();
            query_vocabulary_infos.push(vocabulary_entry.construct_vocabulary_info(chunk_count));

            // 3
            for chunk in chunks {
                query_chunks.push(chunk);
            }

            current_vocabulary_count += 1;
        }

        // 全ての語彙や語彙区切りが確定してからキーストロークを付与する
        append_key_stroke_to_chunks(&mut query_chunks);

        Query::new(query_vocabulary_infos, query_chunks)
    }
}

// 次の語彙を生成するイテレータ
struct NextVocabularyGenerator<'this, 'vocabulary, 'order_function> {
    vocabulary_entries: &'vocabulary Vec<VocabularyEntry>,
    is_prev_vocabulary: bool,
    prev_vocabulary_index: Option<usize>,
    separator_vocabulary: &'vocabulary Option<VocabularyEntry>,
    vocabulary_order: &'this VocabularyOrder<'order_function>,
}

impl<'this, 'vocabulary, 'order_function>
    NextVocabularyGenerator<'this, 'vocabulary, 'order_function>
{
    fn new(
        vocabulary_entries: &'vocabulary Vec<VocabularyEntry>,
        separator_vocabulary: &'vocabulary Option<VocabularyEntry>,
        vocabulary_order: &'this VocabularyOrder<'order_function>,
    ) -> Self {
        Self {
            vocabulary_entries,
            is_prev_vocabulary: false,
            prev_vocabulary_index: None,
            separator_vocabulary,
            vocabulary_order,
        }
    }
}

impl<'this, 'vocabulary, 'order_function> Iterator
    for NextVocabularyGenerator<'this, 'vocabulary, 'order_function>
{
    type Item = &'vocabulary VocabularyEntry;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            if self.is_prev_vocabulary && self.separator_vocabulary.is_some() {
                self.is_prev_vocabulary = false;
                self.separator_vocabulary.as_ref().unwrap()
            // 直前に追加した語彙が語彙リストから選んだ語彙ではなかったり語彙区切りがない場合のみ語彙リストから語彙を選択する
            } else {
                self.is_prev_vocabulary = true;

                let vocabulary_index = self.vocabulary_order.next_vocabulary_entry_index(
                    &self.prev_vocabulary_index,
                    self.vocabulary_entries,
                );

                self.prev_vocabulary_index.replace(vocabulary_index);

                self.vocabulary_entries.get(vocabulary_index).unwrap()
            },
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Query {
    vocabulary_infos: Vec<VocabularyInfo>,
    chunks: Vec<Chunk>,
}

impl Query {
    fn new(vocabulary_infos: Vec<VocabularyInfo>, chunks: Vec<Chunk>) -> Self {
        Self {
            vocabulary_infos,
            chunks,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{gen_candidate, gen_chunk, gen_vocabulary_entry, gen_vocabulary_info};

    #[test]
    fn construct_query_1() {
        let vocabularies = vec![gen_vocabulary_entry!("イオン", ["い", "お", "ん"])];

        let qr = QueryRequest::new(
            &vocabularies,
            VocabularyQuantifier::KeyStroke(NonZeroUsize::new(5).unwrap()),
            VocabularySeparator::WhiteSpace,
            VocabularyOrder::InOrder,
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                vec![
                    gen_vocabulary_info!("イオン", "いおん", vec![0, 1, 2], 3),
                    gen_vocabulary_info!(" ", " ", vec![0], 1)
                ],
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
            VocabularyQuantifier::KeyStroke(NonZeroUsize::new(5).unwrap()),
            VocabularySeparator::None,
            VocabularyOrder::InOrder,
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                vec![
                    gen_vocabulary_info!("イオン", "いおん", vec![0, 1, 2], 3),
                    gen_vocabulary_info!("イオン", "いおん", vec![0, 1, 2], 1)
                ],
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
            VocabularyQuantifier::KeyStroke(NonZeroUsize::new(10).unwrap()),
            VocabularySeparator::None,
            VocabularyOrder::InOrder,
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                vec![
                    gen_vocabulary_info!("イオン", "いおん", vec![0, 1, 2], 3),
                    gen_vocabulary_info!("買っ", "かっ", vec![0, 1], 2),
                    gen_vocabulary_info!("た", "た", vec![0], 1),
                    gen_vocabulary_info!("イオン", "いおん", vec![0, 1, 2], 2),
                ],
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

    #[test]
    fn construct_query_4() {
        let vocabularies = vec![
            gen_vocabulary_entry!("1", ["1"]),
            gen_vocabulary_entry!("2", ["2"]),
        ];

        let qr = QueryRequest::new(
            &vocabularies,
            VocabularyQuantifier::KeyStroke(NonZeroUsize::new(3).unwrap()),
            VocabularySeparator::WhiteSpace,
            VocabularyOrder::Arbitrary(&|prev_vocabulary_index, vocabulary_entries| {
                if prev_vocabulary_index.is_none() {
                    vocabulary_entries.len() - 1
                } else if prev_vocabulary_index.is_some()
                    && *prev_vocabulary_index.as_ref().unwrap() == 0
                {
                    vocabulary_entries.len() - 1
                } else {
                    prev_vocabulary_index.as_ref().unwrap() - 1
                }
            }),
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                vec![
                    gen_vocabulary_info!("2", "2", vec![0], 1),
                    gen_vocabulary_info!(" ", " ", vec![0], 1),
                    gen_vocabulary_info!("1", "1", vec![0], 1),
                ],
                vec![
                    gen_chunk!("2", vec![gen_candidate!(["2"])]),
                    gen_chunk!(" ", vec![gen_candidate!([" "])]),
                    gen_chunk!("1", vec![gen_candidate!(["1"])]),
                ]
            )
        );
    }

    #[test]
    fn construct_query_5() {
        let vocabularies = vec![gen_vocabulary_entry!("イオン", ["い", "お", "ん"])];

        let qr = QueryRequest::new(
            &vocabularies,
            VocabularyQuantifier::Vocabulary(NonZeroUsize::new(2).unwrap()),
            VocabularySeparator::WhiteSpace,
            VocabularyOrder::InOrder,
        );

        let query = qr.construct_query();

        assert_eq!(
            query,
            Query::new(
                vec![
                    gen_vocabulary_info!("イオン", "いおん", vec![0, 1, 2], 3),
                    gen_vocabulary_info!(" ", " ", vec![0], 1)
                ],
                vec![
                    gen_chunk!("い", vec![gen_candidate!(["i"]), gen_candidate!(["yi"])]),
                    gen_chunk!("お", vec![gen_candidate!(["o"])]),
                    gen_chunk!("ん", vec![gen_candidate!(["nn"]), gen_candidate!(["xn"])]),
                    gen_chunk!(" ", vec![gen_candidate!([" "])]),
                ]
            )
        );
    }
}
