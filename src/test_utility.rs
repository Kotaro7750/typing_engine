#[macro_export]
macro_rules! gen_unprocessed_chunk {
    ($chunk_spell:literal) => {
        crate::chunk::Chunk::new($chunk_spell.to_string().try_into().unwrap(), None, None)
    };
}

#[macro_export]
macro_rules! gen_chunk {
    (
            $chunk_spell:literal,
            $key_stroke_candidates:expr
            $(,$ideal_candidate:expr)?
        ) => {
        {
            let _ideal_candidate: Option<crate::chunk::ChunkKeyStrokeCandidate> = None;
            $(let _ideal_candidate = Some($ideal_candidate);)?

            crate::chunk::Chunk::new(
                $chunk_spell.to_string().try_into().unwrap(),
                Some($key_stroke_candidates),
                _ideal_candidate
            )
        }
    };
}

#[macro_export]
macro_rules! gen_vocabulary_spell {
    ([$($spell:literal),*]) => {
        crate::vocabulary::VocabularySpell::Normal(vec![
            $(
                String::from($spell).try_into().unwrap(),
            )*
        ])
    };
    ($spell:literal) => {
        crate::vocabulary::VocabularySpell::Compound(String::from($spell).try_into().unwrap())
    };
}

#[macro_export]
macro_rules! gen_vocabulary_entry {
        (
            $vs:literal,
            [
                $(
                    (
                        $spell:literal
                        $(,$view_count:literal)?
                    )
                ),*
            ]) => {
            crate::vocabulary::VocabularyEntry::new( String::from($vs),
                vec![
                    $(
                        {
                            let _vse = crate::vocabulary::VocabularySpellElement::Normal(String::from($spell).try_into().unwrap());
                            $(let _vse = crate::vocabulary::VocabularySpellElement::Compound((String::from($spell).try_into().unwrap(),std::num::NonZeroUsize::new($view_count).unwrap()));)?
                            _vse
                        },
                    )*
                ]
            ).unwrap()
        };
    }

#[macro_export]
macro_rules! gen_view_position {
    ($position:literal) => {
        crate::vocabulary::ViewPosition::Normal($position)
    };
    ([$($position:literal),*]) => {
        crate::vocabulary::ViewPosition::Compound(vec![
            $(
                $position
            )*
        ])
    };
}

#[macro_export]
macro_rules! gen_vocabulary_info {
    ($view:literal,$spell:literal,$vpos:expr,$chunk_count:literal) => {
        crate::vocabulary::VocabularyInfo::new(
            String::from($view),
            String::from($spell).try_into().unwrap(),
            $vpos,
            $chunk_count.try_into().unwrap(),
        )
    };
}

#[macro_export]
macro_rules! gen_candidate {
        ([$($key_stroke:literal),*]$(, $constraint:literal)?$(, [$($delayed:literal),*])?) => {
            {
                let _constraint: Option<crate::key_stroke::KeyStrokeChar> = None;
                $(let _constraint = Some($constraint.try_into().unwrap());)?

                let _delayed: Option<crate::chunk::DelayedConfirmedCandidateInfo> = None;
                $(let _delayed = Some(crate::chunk::DelayedConfirmedCandidateInfo::new(vec![$($delayed.try_into().unwrap()),*]));)?
                crate::chunk::ChunkKeyStrokeCandidate::new(vec![$($key_stroke.to_string().try_into().unwrap()),*],_constraint,_delayed)
            }
        };
    }
