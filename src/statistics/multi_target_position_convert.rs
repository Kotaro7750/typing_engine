use crate::chunk::KeyStrokeElementCount;
use crate::utility::convert_by_weighted_count;

enum BaseTarget {
    Chunk,
    Spell,
    IdealKeyStroke,
    KeyStroke,
}

/// チャンク・チャンク内の綴り・理想的なキーストローク系列・キーストローク系列間の位置の変換を行う
pub(crate) struct MultiTargetDeltaConverter {
    spell: usize,
    // Vecになっているのは綴り要素のそれぞれに対応させるため
    ideal_key_stroke: KeyStrokeElementCount,
    key_stroke: KeyStrokeElementCount,
    base: BaseTarget,
}

impl MultiTargetDeltaConverter {
    pub(crate) fn new(
        spell: usize,
        ideal_key_stroke: KeyStrokeElementCount,
        key_stroke: KeyStrokeElementCount,
        base: BaseTarget,
    ) -> Self {
        assert!(spell == 1 || spell == 2);
        assert!(!key_stroke.is_double() || (key_stroke.is_double() && spell == 2));
        assert!(!ideal_key_stroke.is_double() || (ideal_key_stroke.is_double() && spell == 2));

        Self {
            spell,
            ideal_key_stroke,
            key_stroke,
            base,
        }
    }

    /// 基準の位置はチャンクでいうとどこか
    pub(crate) fn chunk_delta(&self, base_deltas: Vec<usize>) -> Vec<usize> {
        // 他の対象のどこに位置があったとしてもそのチャンク末であることには変わりない
        base_deltas.iter().map(|_| 1).collect()
    }

    /// 基準の位置は綴りでいうとどこか
    pub(crate) fn spell_delta(&self, base_deltas: Vec<usize>) -> Vec<usize> {
        match self.base {
            BaseTarget::Chunk => base_deltas.iter().map(|_| self.spell).collect(),
            BaseTarget::Spell => base_deltas,
            BaseTarget::IdealKeyStroke => base_deltas
                .iter()
                .map(|ideal_key_stroke_delta| {
                    self.ideal_key_stroke
                        .convert_key_stroke_delta_to_spell_delta(
                            self.spell,
                            *ideal_key_stroke_delta,
                        )
                })
                .collect(),
            BaseTarget::KeyStroke => base_deltas
                .iter()
                .map(|key_stroke_delta| {
                    self.key_stroke
                        .convert_key_stroke_delta_to_spell_delta(self.spell, *key_stroke_delta)
                })
                .collect(),
        }
    }

    /// 基準の位置は理想的なキーストローク系列でいうとどこか
    pub(crate) fn ideal_key_stroke_delta(&self, base_deltas: Vec<usize>) -> Vec<usize> {
        match self.base {
            BaseTarget::Chunk => base_deltas
                .iter()
                .map(|_| self.ideal_key_stroke.whole_count())
                .collect(),
            BaseTarget::Spell => base_deltas
                .iter()
                .map(|spell_delta| {
                    self.ideal_key_stroke
                        .convert_spell_delta_to_key_stroke_delta(*spell_delta)
                })
                .collect(),
            BaseTarget::IdealKeyStroke => base_deltas,
            BaseTarget::KeyStroke => base_deltas
                .iter()
                .map(|key_stroke_delta| {
                    convert_between_key_stroke_delta(
                        &self.key_stroke,
                        &self.ideal_key_stroke,
                        self.spell,
                        *key_stroke_delta,
                    )
                })
                .collect(),
        }
    }

    /// 基準の位置はキーストローク系列でいうとどこか
    pub(crate) fn key_stroke_delta(&self, base_deltas: Vec<usize>) -> Vec<usize> {
        match self.base {
            BaseTarget::Chunk => base_deltas
                .iter()
                .map(|_| self.key_stroke.whole_count())
                .collect(),
            BaseTarget::Spell => base_deltas
                .iter()
                .map(|spell_delta| {
                    self.key_stroke
                        .convert_spell_delta_to_key_stroke_delta(*spell_delta)
                })
                .collect(),
            BaseTarget::IdealKeyStroke => base_deltas
                .iter()
                .map(|key_stroke_delta| {
                    convert_between_key_stroke_delta(
                        &self.ideal_key_stroke,
                        &self.key_stroke,
                        self.spell,
                        *key_stroke_delta,
                    )
                })
                .collect(),
            BaseTarget::KeyStroke => base_deltas,
        }
    }
}

fn convert_between_key_stroke_delta(
    from_count_of_spell_elements: &KeyStrokeElementCount,
    to_count_of_spell_elements: &KeyStrokeElementCount,
    spell: usize,
    from_delta: usize,
) -> usize {
    let pseudo_from_cose =
        from_count_of_spell_elements.construct_pseudo_count_of_spell_elements(spell);

    let pseudo_to_cose = to_count_of_spell_elements.construct_pseudo_count_of_spell_elements(spell);

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn multi_target_delta_converter_1() {
        let m = MultiTargetDeltaConverter::new(
            2,
            KeyStrokeElementCount::new(&vec![3]),
            KeyStrokeElementCount::new(&vec![2, 3]),
            BaseTarget::Chunk,
        );

        assert_eq!(m.chunk_delta(vec![1]), vec![1]);
        assert_eq!(m.spell_delta(vec![1]), vec![2]);
        assert_eq!(m.ideal_key_stroke_delta(vec![1]), vec![3]);
        assert_eq!(m.key_stroke_delta(vec![1]), vec![5]);
    }

    #[test]
    fn multi_target_delta_converter_2() {
        let m = MultiTargetDeltaConverter::new(
            2,
            KeyStrokeElementCount::new(&vec![3]),
            KeyStrokeElementCount::new(&vec![2, 3]),
            BaseTarget::Spell,
        );

        assert_eq!(m.chunk_delta(vec![1, 2]), vec![1, 1]);

        assert_eq!(m.spell_delta(vec![1, 2]), vec![1, 2]);
        assert_eq!(m.ideal_key_stroke_delta(vec![1, 2]), vec![3, 3]);
        assert_eq!(m.key_stroke_delta(vec![1, 2]), vec![2, 5]);
    }

    #[test]
    fn multi_target_delta_converter_3() {
        let m = MultiTargetDeltaConverter::new(
            2,
            KeyStrokeElementCount::new(&vec![3]),
            KeyStrokeElementCount::new(&vec![2, 3]),
            BaseTarget::IdealKeyStroke,
        );

        assert_eq!(m.chunk_delta(vec![1, 2, 3]), vec![1, 1, 1]);
        assert_eq!(m.spell_delta(vec![1, 2, 3]), vec![1, 2, 2]);
        assert_eq!(m.ideal_key_stroke_delta(vec![1, 2, 3]), vec![1, 2, 3]);
        assert_eq!(m.key_stroke_delta(vec![1, 2, 3]), vec![2, 4, 5]);
    }

    #[test]
    fn multi_target_delta_converter_4() {
        let m = MultiTargetDeltaConverter::new(
            2,
            KeyStrokeElementCount::new(&vec![3]),
            KeyStrokeElementCount::new(&vec![2, 3]),
            BaseTarget::KeyStroke,
        );

        assert_eq!(m.chunk_delta(vec![1, 2, 3, 4, 5]), vec![1, 1, 1, 1, 1]);
        assert_eq!(m.spell_delta(vec![1, 2, 3, 4, 5]), vec![1, 1, 2, 2, 2]);
        assert_eq!(
            m.ideal_key_stroke_delta(vec![1, 2, 3, 4, 5]),
            vec![1, 1, 2, 3, 3]
        );
        assert_eq!(m.key_stroke_delta(vec![1, 2, 3, 4, 5]), vec![1, 2, 3, 4, 5]);
    }
}
