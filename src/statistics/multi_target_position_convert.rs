use crate::typing_primitive_types::chunk::key_stroke_candidate::KeyStrokeElementCount;
use crate::utility::convert_by_weighted_count;

#[cfg(test)]
mod test;

pub(crate) enum BaseTarget {
    Chunk,
    Spell,
    IdealKeyStroke,
    KeyStroke,
}

/// A struct that converts the position between chunk, spell in chunk, ideal key stroke sequence,
/// and key stroke sequence.
pub(crate) struct MultiTargetDeltaConverter {
    /// Conversion base.
    base: BaseTarget,
    /// Spell count in a target chunk.
    spell_count: usize,
    /// Ideal key stroke count in a target chunk.
    ideal_key_stroke_count: KeyStrokeElementCount,
    /// Key stroke count in a target chunk.
    key_stroke_count: KeyStrokeElementCount,
}

impl MultiTargetDeltaConverter {
    pub(crate) fn new(
        spell_count: usize,
        ideal_key_stroke_count: KeyStrokeElementCount,
        key_stroke_count: KeyStrokeElementCount,
        base: BaseTarget,
    ) -> Self {
        assert!(spell_count == 1 || spell_count == 2);
        assert!(
            !key_stroke_count.is_double() || (key_stroke_count.is_double() && spell_count == 2)
        );
        assert!(
            !ideal_key_stroke_count.is_double()
                || (ideal_key_stroke_count.is_double() && spell_count == 2)
        );

        Self {
            spell_count,
            ideal_key_stroke_count,
            key_stroke_count,
            base,
        }
    }

    /// Convert the position delta of base to the position delta of chunk.
    /// Because the unit of this converter is a chunk, the delta is always 1.
    pub(crate) fn chunk_delta(&self, base_deltas: &[usize]) -> Vec<usize> {
        // 他の対象のどこに位置があったとしてもそのチャンク末であることには変わりない
        base_deltas.iter().map(|_| 1).collect()
    }

    /// Convert the position delta of base to the position delta of spell.
    pub(crate) fn spell_delta(&self, base_deltas: &[usize]) -> Vec<usize> {
        match self.base {
            BaseTarget::Chunk => base_deltas.iter().map(|_| self.spell_count).collect(),
            BaseTarget::Spell => base_deltas.to_vec(),
            BaseTarget::IdealKeyStroke => base_deltas
                .iter()
                .map(|ideal_key_stroke_delta| {
                    self.ideal_key_stroke_count
                        .convert_key_stroke_delta_to_spell_delta(
                            self.spell_count,
                            *ideal_key_stroke_delta,
                        )
                })
                .collect(),
            BaseTarget::KeyStroke => base_deltas
                .iter()
                .map(|key_stroke_delta| {
                    self.key_stroke_count
                        .convert_key_stroke_delta_to_spell_delta(
                            self.spell_count,
                            *key_stroke_delta,
                        )
                })
                .collect(),
        }
    }

    /// Convert the position delta of base to the position delta of ideal key strokes.
    pub(crate) fn ideal_key_stroke_delta(&self, base_deltas: &[usize]) -> Vec<usize> {
        match self.base {
            BaseTarget::Chunk => base_deltas
                .iter()
                .map(|_| self.ideal_key_stroke_count.whole_count())
                .collect(),
            BaseTarget::Spell => base_deltas
                .iter()
                .map(|spell_delta| {
                    self.ideal_key_stroke_count
                        .convert_spell_delta_to_key_stroke_delta(self.spell_count, *spell_delta)
                })
                .collect(),
            BaseTarget::IdealKeyStroke => base_deltas.to_vec(),
            BaseTarget::KeyStroke => base_deltas
                .iter()
                .map(|key_stroke_delta| {
                    convert_between_key_stroke_delta(
                        &self.key_stroke_count,
                        &self.ideal_key_stroke_count,
                        self.spell_count,
                        *key_stroke_delta,
                    )
                })
                .collect(),
        }
    }

    /// Convert the position delta of base to the position delta of key strokes.
    pub(crate) fn key_stroke_delta(&self, base_deltas: &[usize]) -> Vec<usize> {
        match self.base {
            BaseTarget::Chunk => base_deltas
                .iter()
                .map(|_| self.key_stroke_count.whole_count())
                .collect(),
            BaseTarget::Spell => base_deltas
                .iter()
                .map(|spell_delta| {
                    self.key_stroke_count
                        .convert_spell_delta_to_key_stroke_delta(self.spell_count, *spell_delta)
                })
                .collect(),
            BaseTarget::IdealKeyStroke => base_deltas
                .iter()
                .map(|key_stroke_delta| {
                    convert_between_key_stroke_delta(
                        &self.ideal_key_stroke_count,
                        &self.key_stroke_count,
                        self.spell_count,
                        *key_stroke_delta,
                    )
                })
                .collect(),
            BaseTarget::KeyStroke => base_deltas.to_vec(),
        }
    }
}

/// Convert the position delta between key strokes and ideal key strokes.
fn convert_between_key_stroke_delta(
    from: &KeyStrokeElementCount,
    to: &KeyStrokeElementCount,
    spell_count: usize,
    from_delta: usize,
) -> usize {
    let pseudo_from_cose = from.construct_pseudo_count_of_spell_elements(spell_count);

    let pseudo_to_cose = to.construct_pseudo_count_of_spell_elements(spell_count);

    let i = pseudo_from_cose.spell_elements_index_of_delta(from_delta);

    let in_spell_element_from_delta = if i == 0 {
        from_delta
    } else {
        assert!(i > 0);
        from_delta - pseudo_from_cose.key_stroke_count_offset(i)
    };

    let delta = convert_by_weighted_count(
        pseudo_from_cose.count_of_spell_elements_index(i),
        pseudo_to_cose.count_of_spell_elements_index(i),
        in_spell_element_from_delta,
    );

    if i > 0 {
        delta + pseudo_to_cose.key_stroke_count_offset(i)
    } else {
        delta
    }
}
