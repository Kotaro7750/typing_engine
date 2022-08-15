use crate::chunk::Chunk;
use crate::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::spell::SpellString;

// 辞書中の各語彙
pub struct VocabularyEntry {
    // 問題文として表示する文字列
    view: String,
    // viewの各文字のそれぞれの綴り
    // ex. 「機能」という語彙に対しては[き,のう]
    spell_list: Vec<SpellString>,
}

impl VocabularyEntry {
    fn new(view: String, spell_list: Vec<SpellString>) -> Option<Self> {
        if view.chars().count() != spell_list.len() {
            None
        } else {
            Some(Self { view, spell_list })
        }
    }

    // 語彙全体の綴りを構築する
    // 表示文字列の各文字に対しての綴りをつなげたもの
    fn construct_spell_string(&self) -> SpellString {
        let mut s = String::new();

        for spell in &self.spell_list {
            s.push_str(spell);
        }

        s.try_into().unwrap()
    }

    // 語彙からチャンク列を構築する
    // この段階ではそれぞれのチャンクに対するキーストローク候補は設定しない
    fn construct_chunks_from_vocabulary_entry(&self) -> Vec<Chunk> {
        let mut chunks = Vec::<Chunk>::new();

        let spell_chars: Vec<char> = self.construct_spell_string().chars().collect();

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

            let spell = if uni.is_ascii_graphic() || uni == ' ' {
                i += 1;
                uni.to_string()
            } else {
                if CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY.contains_key(bi.as_str()) {
                    i += 2;
                    bi
                } else {
                    assert!(
                        CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY.contains_key(uni.to_string().as_str())
                    );
                    i += 1;
                    uni.to_string()
                }
            }
            .try_into()
            .unwrap();

            chunks.push(Chunk::new(spell, None));
        }

        chunks
    }
}

#[cfg(test)]
mod test {
    use super::*;

    macro_rules! chunk {
        ($s:literal) => {
            Chunk::new(String::from($s).try_into().unwrap(), None)
        };
    }

    macro_rules! vocabulary_entry {
        ($vs:literal,[$($spell:literal),*]) => {
            VocabularyEntry::new( String::from($vs),
                vec![
                    $(
                        String::from($spell).try_into().unwrap(),
                    )*
                ]
            ).unwrap()
        };
    }

    macro_rules! equal_check_construct_chunks_from_vocabulary {
        (($vs:literal,[$($spell:literal),*]), [$($s:literal),*]) => {
            let ve = vocabulary_entry!($vs,[$($spell),*]);

            assert_eq!(
                ve.construct_chunks_from_vocabulary_entry(),
                vec![$(chunk!($s)),*]
            );
        }
    }

    #[test]
    fn construct_chunks_from_vocabulary_entry_1() {
        equal_check_construct_chunks_from_vocabulary!(("今日", ["きょ", "う"]), ["きょ", "う"]);
    }

    #[test]
    fn construct_chunks_from_vocabulary_entry_2() {
        equal_check_construct_chunks_from_vocabulary!((" 　", [" ", "　"]), [" ", "　"]);
    }

    #[test]
    fn construct_chunks_from_vocabulary_entry_3() {
        equal_check_construct_chunks_from_vocabulary!(("big", ["b", "i", "g"]), ["b", "i", "g"]);
    }
}
