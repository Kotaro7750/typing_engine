#[macro_export]
macro_rules! gen_chunk_candidate_unappended {
    ($chunk_spell:literal) => {
        crate::typing_primitive_types::chunk::chunk_candidate_unappended::ChunkCandidateUnappended::new(
            $chunk_spell.to_string().try_into().unwrap()
        )
    };
}

#[macro_export]
macro_rules! gen_unprocessed_chunk {
    ($chunk_spell:literal) => {
        crate::typing_primitive_types::chunk::Chunk::new(
            $chunk_spell.to_string().try_into().unwrap(),
            None,
            None,
            crate::typing_primitive_types::chunk::ChunkState::Unprocessed,
            None,
        )
    };
}

#[macro_export]
macro_rules! gen_chunk {
    (
            $chunk_spell:literal,
            $key_stroke_candidates:expr,
            $state:expr
            $(,$ideal_candidate:expr)?
            $(,[$($actual_key_stroke:expr),*])?
        ) => {
        {
            let _ideal_candidate: Option<crate::typing_primitive_types::chunk::key_stroke_candidate::ChunkKeyStrokeCandidate> = None;
            $(let _ideal_candidate = Some($ideal_candidate);)?

            let _actual_key_stroke: Option<Vec<crate::typing_primitive_types::key_stroke::ActualKeyStroke>> = None;
            $(let _actual_key_stroke = Some(vec![$($actual_key_stroke.try_into().unwrap()),*]);)?

            crate::typing_primitive_types::chunk::Chunk::new(
                $chunk_spell.to_string().try_into().unwrap(),
                Some($key_stroke_candidates),
                _ideal_candidate,
                $state,
                _actual_key_stroke,
            )
        }
    };
}

#[macro_export]
macro_rules! gen_vocabulary_spell {
    ([$($spell:literal),*]) => {
        crate::typing_primitive_types::vocabulary::VocabularySpell::Normal(vec![
            $(
                String::from($spell).try_into().unwrap(),
            )*
        ])
    };
    ($spell:literal) => {
        crate::typing_primitive_types::vocabulary::VocabularySpell::Compound(String::from($spell).try_into().unwrap())
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
            crate::typing_primitive_types::vocabulary::VocabularyEntry::new( String::from($vs),
                vec![
                    $(
                        {
                            let _vse = crate::typing_primitive_types::vocabulary::VocabularySpellElement::Normal(String::from($spell).try_into().unwrap());
                            $(let _vse = crate::typing_primitive_types::vocabulary::VocabularySpellElement::Compound((String::from($spell).try_into().unwrap(),std::num::NonZeroUsize::new($view_count).unwrap()));)?
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
        crate::typing_primitive_types::vocabulary::ViewPosition::Normal($position)
    };
    ([$($position:literal),*]) => {
        crate::typing_primitive_types::vocabulary::ViewPosition::Compound(vec![
            $(
                $position
            )*
        ])
    };
}

#[macro_export]
macro_rules! gen_vocabulary_info {
    ($view:literal,$spell:literal,$vpos:expr,$chunk_count:literal) => {
        crate::typing_primitive_types::vocabulary::VocabularyInfo::new(
            String::from($view),
            String::from($spell).try_into().unwrap(),
            $vpos,
            $chunk_count.try_into().unwrap(),
        )
    };
}

#[macro_export]
macro_rules! gen_candidate {
        ([$($key_stroke:literal),*], $is_active:literal, $cursor_position:expr$(, $constraint:literal)?$(, [$($delayed:literal),*])?) => {
            {
                let _constraint: Option<crate::typing_primitive_types::key_stroke::KeyStrokeChar> = None;
                $(let _constraint = Some($constraint.try_into().unwrap());)?

                let _delayed: Option<crate::typing_primitive_types::chunk::key_stroke_candidate::DelayedConfirmedCandidateInfo> = None;
                $(let _delayed = Some(crate::typing_primitive_types::chunk::key_stroke_candidate::DelayedConfirmedCandidateInfo::new(vec![$($delayed.try_into().unwrap()),*]));)?

                let _is_active = $is_active.try_into().unwrap();

                let _cursor_position = $cursor_position.try_into().unwrap();

                crate::typing_primitive_types::chunk::key_stroke_candidate::ChunkKeyStrokeCandidate::new(
                    vec![$($key_stroke.to_string().try_into().unwrap()),*],
                    _constraint,
                    _delayed,
                    _is_active,
                    _cursor_position,
                )
            }
        };
    }
