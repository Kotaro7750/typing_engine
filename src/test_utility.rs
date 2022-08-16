#[macro_export]
macro_rules! gen_unprocessed_chunk {
    ($chunk_spell:literal) => {
        crate::chunk::Chunk::new($chunk_spell.to_string().try_into().unwrap(), None)
    };
}

#[macro_export]
macro_rules! gen_chunk {
    (
            $chunk_spell:literal,
            $key_stroke_candidates:expr
        ) => {
        crate::chunk::Chunk::new(
            $chunk_spell.to_string().try_into().unwrap(),
            Some($key_stroke_candidates),
        )
    };
}

#[macro_export]
macro_rules! gen_vocabulary_entry {
        ($vs:literal,[$($spell:literal),*]) => {
            crate::vocabulary::VocabularyEntry::new( String::from($vs),
                vec![
                    $(
                        String::from($spell).try_into().unwrap(),
                    )*
                ]
            ).unwrap()
        };
    }

#[macro_export]
macro_rules! gen_candidate {
        ([$($key_stroke:literal),*]$(, $constraint:literal)?) => {
            {
                let _constraint: Option<crate::key_stroke::KeyStrokeChar> = None;
                $(let _constraint = Some($constraint.try_into().unwrap());)?
                crate::chunk::ChunkKeyStrokeCandidate::new(vec![$($key_stroke.to_string().try_into().unwrap()),*],_constraint)
            }
        };
    }
