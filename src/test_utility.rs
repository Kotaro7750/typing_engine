#[macro_export]
macro_rules! gen_chunk_candidate_unappended {
    ($chunk_spell:literal) => {
        crate::typing_primitive_types::chunk::candidate_unappended::ChunkCandidateUnappended::new(
            $chunk_spell.to_string().try_into().unwrap(),
        )
    };
}

#[macro_export]
macro_rules! gen_chunk_unprocessed {
    (
            $chunk_spell:literal,
            $key_stroke_candidates:expr,
            $ideal_candidate:expr
        ) => {
        {
            let _ideal_candidate: crate::typing_primitive_types::chunk::key_stroke_candidate::ChunkKeyStrokeCandidate = $ideal_candidate;

            crate::typing_primitive_types::chunk::unprocessed::ChunkUnprocessed::new(
                $chunk_spell.to_string().try_into().unwrap(),
                $key_stroke_candidates,
                _ideal_candidate,
            )
        }
    };
}

#[macro_export]
macro_rules! gen_chunk_inflight {
    (
            $chunk_spell:literal,
            $key_stroke_candidates:expr,
            $inactive_key_stroke_candidates:expr,
            $ideal_candidate:expr,
            [$($actual_key_stroke:expr),*],
            $key_stroke_cursor_position:literal,
            [$($pending_key_strokes:expr),*]
        ) => {
        {

            let _actual_key_stroke = vec![$($actual_key_stroke.try_into().unwrap()),*];
            let _pending_key_strokes = vec![$($pending_key_strokes.try_into().unwrap()),*];

            crate::typing_primitive_types::chunk::inflight::ChunkInflight::new(
                $chunk_spell.to_string().try_into().unwrap(),
                $key_stroke_candidates,
                $inactive_key_stroke_candidates,
                $ideal_candidate,
                _actual_key_stroke,
                $key_stroke_cursor_position,
                _pending_key_strokes
            )
        }
    };
}

#[macro_export]
macro_rules! gen_chunk_confirmed {
    (
            $chunk_spell:literal,
            $key_stroke_candidates:expr,
            $inactive_key_stroke_candidates:expr,
            $ideal_candidate:expr,
            [$($actual_key_stroke:expr),*]
        ) => {
        {

            let _actual_key_stroke = vec![$($actual_key_stroke.try_into().unwrap()),*];

            crate::typing_primitive_types::chunk::confirmed::ChunkConfirmed::new(
                $chunk_spell.to_string().try_into().unwrap(),
                $key_stroke_candidates,
                $inactive_key_stroke_candidates,
                $ideal_candidate,
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
        ($key_stroke:expr$(, $constraint:literal)?$(, [$($delayed:literal),*])?) => {
            {
                let _constraint: Option<crate::typing_primitive_types::key_stroke::KeyStrokeChar> = None;
                $(let _constraint = Some($constraint.try_into().unwrap());)?

                let _delayed: Option<crate::typing_primitive_types::chunk::key_stroke_candidate::DelayedConfirmedCandidateInfo> = None;
                $(let _delayed = Some(crate::typing_primitive_types::chunk::key_stroke_candidate::DelayedConfirmedCandidateInfo::new(vec![$($delayed.try_into().unwrap()),*]));)?

                crate::typing_primitive_types::chunk::key_stroke_candidate::ChunkKeyStrokeCandidate::new(
                    $key_stroke,
                    _constraint,
                    _delayed,
                )
            }
        };
    }

#[macro_export]
macro_rules! gen_candidate_key_stroke {
    ($key_stroke_string:literal) => {
        crate::typing_primitive_types::chunk::key_stroke_candidate::CandidateKeyStroke::Normal(
            String::from($key_stroke_string).try_into().unwrap(),
        )
    };

    ([$key_stroke_string:literal]) => {
        crate::typing_primitive_types::chunk::key_stroke_candidate::CandidateKeyStroke::Double(
            String::from($key_stroke_string).try_into().unwrap(),
        )
    };

    ([$key_stroke_string1:literal, $key_stroke_string2:literal]) => {
        crate::typing_primitive_types::chunk::key_stroke_candidate::CandidateKeyStroke::DoubleSplitted(
            String::from($key_stroke_string1).try_into().unwrap(),
            String::from($key_stroke_string2).try_into().unwrap(),
        )
    };
}
