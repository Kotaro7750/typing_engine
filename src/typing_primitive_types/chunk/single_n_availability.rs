//! Module for checking "n" availability as a key stroke for chunk "ん" (syllabic nasal sound).
use super::ChunkSpell;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;

/// An enum representing the availability of key stroke "n" for chunk "ん" (syllabic nasal sound).
/// Availability depends on the next chunk's spell and head key strokes.
pub(super) enum SingleNAvailability {
    /// Can use "n" for "ん" regardless of the next chunk's head key strokes.
    /// Next chunk's head key strokes are stored in the vector.
    All(Vec<KeyStrokeChar>),
    /// Can use "n" for "ん" depending on the next chunk's head key strokes.
    /// Possible next chunk's head key strokes are stored in the vector.
    Partial(Vec<KeyStrokeChar>),
    /// Cannot use "n" for "ん" at all.
    Cannot,
}

impl SingleNAvailability {
    // 「ん」のキーストロークとして「n」を使っていいか判定する
    pub(super) fn check_single_n_availability(
        next_chunk_spell: &Option<ChunkSpell>,
        next_chunk_head: Option<&Vec<KeyStrokeChar>>,
    ) -> SingleNAvailability {
        // 最後のチャンクの場合には許容しない
        if next_chunk_head.is_none() || next_chunk_spell.is_none() {
            return SingleNAvailability::Cannot;
        }

        if let ChunkSpell::DisplayableAscii(_) = next_chunk_spell.as_ref().unwrap() {
            return SingleNAvailability::Cannot;
        }

        let next_chunk_head = next_chunk_head.unwrap();

        let available_key_stroke_chars: Vec<KeyStrokeChar> = next_chunk_head
            .iter()
            .filter(|ksc| {
                // 次のチャンク先頭のキーストロークが「a」「i」「u」「e」「o」「y」「n」の場合には「n」で「ん」を打てない
                !(**ksc == 'a'
                    || **ksc == 'i'
                    || **ksc == 'u'
                    || **ksc == 'e'
                    || **ksc == 'o'
                    || **ksc == 'y'
                    || **ksc == 'n')
            })
            .cloned()
            .collect();

        if available_key_stroke_chars.is_empty() {
            SingleNAvailability::Cannot
        } else if available_key_stroke_chars.len() == next_chunk_head.len() {
            SingleNAvailability::All(available_key_stroke_chars)
        } else {
            assert_eq!(available_key_stroke_chars.len(), 1);
            SingleNAvailability::Partial(available_key_stroke_chars)
        }
    }
}
