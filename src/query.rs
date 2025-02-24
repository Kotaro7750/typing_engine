use rand::random;
use std::num::NonZeroUsize;

use crate::typing_primitive_types::{
    chunk::{
        chunk_candidate_unappended::{append_key_stroke_to_chunks, ChunkCandidateUnappended},
        Chunk,
    },
    vocabulary::{VocabularyEntry, VocabularyInfo, VocabularySpellElement},
};

#[cfg(test)]
mod test;

/// A vocabulary quantifier for constructing query.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VocabularyQuantifier {
    /// Vocabularies are selected to meet key stroke count.
    KeyStroke(NonZeroUsize),
    /// Vocabularies are selected to meet vocabulary count.
    Vocabulary(NonZeroUsize),
}

/// A vocabulary used to separate between vocabularies of query string.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum VocabularySeparator {
    /// No vocabulary is inserted between selected vocabularies.
    None,
    /// Whitespace is inserted between selected vocabularies.
    WhiteSpace,
    /// Any vocabulary is inserted between selected vocabularies.
    Vocabulary(VocabularyEntry),
}

impl VocabularySeparator {
    // 語彙の区切り語彙がないかどうか
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn generate_separator_vocabulary(&self) -> VocabularyEntry {
        match self {
            Self::Vocabulary(v) => v.clone(),
            Self::WhiteSpace => VocabularyEntry::new(
                " ".to_string(),
                vec![VocabularySpellElement::Normal(
                    " ".to_string().try_into().unwrap(),
                )],
            )
            .unwrap(),
            Self::None => unreachable!("this function cannot be called when self is None"),
        }
    }
}

/// An order specifier by which vocabularies are selected from vocabulary list.
pub enum VocabularyOrder {
    /// Vocabularies are selected randomly from vocabulary list.
    Random,
    /// Vocabularies are selected in-order from vocabulary list.
    InOrder,
    /// Vocabularies are selected user-defined order from vocabulary list.
    Arbitrary(Box<dyn Fn(&Option<usize>, &[&VocabularyEntry]) -> usize>),
}

impl VocabularyOrder {
    // 前回の語彙のインデックスと語彙リストから次使う語彙のインデックスを生成する
    fn next_vocabulary_entry_index(
        &self,
        prev_index: &Option<usize>,
        vocabulary_entries: &[&VocabularyEntry],
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

/// A request for constructing query.
pub struct QueryRequest<'vocabulary> {
    vocabulary_entries: Vec<&'vocabulary VocabularyEntry>,
    vocabulary_quantifier: VocabularyQuantifier,
    vocabulary_separator: VocabularySeparator,
    vocabulary_order: VocabularyOrder,
}

impl<'vocabulary> QueryRequest<'vocabulary> {
    /// Construct a new [`QueryRequest`].
    pub fn new(
        vocabulary_entries: &[&'vocabulary VocabularyEntry],
        vocabulary_quantifier: VocabularyQuantifier,
        vocabulary_separator: VocabularySeparator,
        vocabulary_order: VocabularyOrder,
    ) -> Self {
        Self {
            vocabulary_entries: From::<&[&VocabularyEntry]>::from(vocabulary_entries),
            vocabulary_quantifier,
            vocabulary_separator,
            vocabulary_order,
        }
    }

    pub(crate) fn construct_query(&self) -> Query {
        // 語彙リストから選んだ語彙の区切りとして使う語彙
        let separator_vocabulary = if self.vocabulary_separator.is_none() {
            None
        } else {
            Some(self.vocabulary_separator.generate_separator_vocabulary())
        };

        let next_vocabulary_generator = NextVocabularyGenerator::new(
            &self.vocabulary_entries,
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
        let mut query_chunks = Vec::<ChunkCandidateUnappended>::new();
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
        let mut query_chunks = append_key_stroke_to_chunks(&query_chunks);

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
            NonZeroUsize::new(last_chunk.calc_min_key_stroke_count() - over_key_stroke_count)
                .unwrap(),
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
        let mut query_chunks = Vec::<ChunkCandidateUnappended>::new();
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
        let query_chunks = append_key_stroke_to_chunks(&query_chunks);

        Query::new(query_vocabulary_infos, query_chunks)
    }
}

// 次の語彙を生成するイテレータ
struct NextVocabularyGenerator<'this, 'vocabulary> {
    vocabulary_entries: &'this [&'vocabulary VocabularyEntry],
    is_prev_vocabulary: bool,
    prev_vocabulary_index: Option<usize>,
    separator_vocabulary: &'vocabulary Option<VocabularyEntry>,
    vocabulary_order: &'this VocabularyOrder,
}

impl<'this, 'vocabulary> NextVocabularyGenerator<'this, 'vocabulary> {
    fn new(
        vocabulary_entries: &'this [&'vocabulary VocabularyEntry],
        separator_vocabulary: &'vocabulary Option<VocabularyEntry>,
        vocabulary_order: &'this VocabularyOrder,
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

impl<'this, 'vocabulary> Iterator for NextVocabularyGenerator<'this, 'vocabulary> {
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
pub(crate) struct Query {
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

    pub(crate) fn decompose(self) -> (Vec<VocabularyInfo>, Vec<Chunk>) {
        (self.vocabulary_infos, self.chunks)
    }
}
