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
    ideal_key_stroke: Vec<usize>,
    key_stroke: Vec<usize>,
    base: BaseTarget,
}

impl MultiTargetDeltaConverter {
    pub(crate) fn new(
        spell: usize,
        ideal_key_stroke: Vec<usize>,
        key_stroke: Vec<usize>,
        base: BaseTarget,
    ) -> Self {
        assert!(spell == 1 || spell == 2);
        assert!(ideal_key_stroke.len() == 1 || (ideal_key_stroke.len() == 2 && spell == 2));
        assert!(key_stroke.len() == 1 || (key_stroke.len() == 2 && spell == 2));

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
                    convert_key_stroke_delta_to_spell_delta(
                        &self.ideal_key_stroke,
                        self.spell,
                        *ideal_key_stroke_delta,
                    )
                })
                .collect(),
            BaseTarget::KeyStroke => base_deltas
                .iter()
                .map(|key_stroke_delta| {
                    convert_key_stroke_delta_to_spell_delta(
                        &self.key_stroke,
                        self.spell,
                        *key_stroke_delta,
                    )
                })
                .collect(),
        }
    }

    /// 基準の位置は理想的なキーストローク系列でいうとどこか
    pub(crate) fn ideal_key_stroke_delta(&self, base_deltas: Vec<usize>) -> Vec<usize> {
        match self.base {
            BaseTarget::Chunk => base_deltas
                .iter()
                .map(|_| self.ideal_key_stroke.iter().fold(0, |acc, &e| acc + e))
                .collect(),
            BaseTarget::Spell => base_deltas
                .iter()
                .map(|spell_delta| {
                    convert_spell_delta_to_key_stroke_delta(&self.ideal_key_stroke, *spell_delta)
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
                .map(|_| self.key_stroke.iter().fold(0, |acc, &e| acc + e))
                .collect(),
            BaseTarget::Spell => base_deltas
                .iter()
                .map(|spell_delta| {
                    convert_spell_delta_to_key_stroke_delta(&self.key_stroke, *spell_delta)
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

/// 対象間の数の違いを考慮して位置の変換をする
fn convert_by_weighted_count(from_count: usize, to_count: usize, from_delta: usize) -> usize {
    // ceil(a/b)は (a+b-1)/b とできる
    ((from_delta * to_count) + from_count - 1) / from_count
}

/// カウントを要素として持つスライスvのindexまでの要素の累積和を取る
fn accumulation(v: &[usize], index: usize) -> usize {
    let mut accum = 0;
    for i in 0..=index {
        accum += v[i];
    }

    accum
}

// 理想的なキーストローク・キーストロークの位置を綴りの位置に変換する
fn convert_key_stroke_delta_to_spell_delta(
    count_of_spell_elements: &[usize],
    spell: usize,
    key_stroke_delta: usize,
) -> usize {
    let spell_elements_index =
        spell_elements_index_of_key_stroke_delta(count_of_spell_elements, key_stroke_delta);

    let in_spell_element_key_stroke_delta = if spell_elements_index == 0 {
        key_stroke_delta
    } else {
        assert!(spell_elements_index > 0);
        key_stroke_delta - accumulation(count_of_spell_elements, spell_elements_index - 1)
    };

    let effective_spell_count = if count_of_spell_elements.len() == 1 {
        spell
    } else {
        1
    };

    convert_by_weighted_count(
        count_of_spell_elements[spell_elements_index],
        effective_spell_count,
        in_spell_element_key_stroke_delta,
    ) + spell_elements_index
}

// 綴りの位置を理想的なキーストローク・キーストロークの位置に変換する
fn convert_spell_delta_to_key_stroke_delta(
    count_of_spell_elements: &[usize],
    spell_delta: usize,
) -> usize {
    if count_of_spell_elements.len() == 2 {
        accumulation(&count_of_spell_elements, spell_delta - 1)
    } else {
        count_of_spell_elements[0]
    }
}

fn convert_between_key_stroke_delta(
    from_count_of_spell_elements: &[usize],
    to_count_of_spell_elements: &[usize],
    spell: usize,
    from_delta: usize,
) -> usize {
    let pseudo_from_cose =
        construct_pseudo_count_of_spell_elements(from_count_of_spell_elements, spell);

    let pseudo_to_cose =
        construct_pseudo_count_of_spell_elements(to_count_of_spell_elements, spell);

    let i = spell_elements_index_of_key_stroke_delta(&pseudo_from_cose, from_delta);

    let in_spell_element_from_delta = if i == 0 {
        from_delta
    } else {
        assert!(i > 0);
        from_delta - accumulation(&pseudo_from_cose, i - 1)
    };

    let delta = convert_by_weighted_count(
        pseudo_from_cose[i],
        pseudo_to_cose[i],
        in_spell_element_from_delta,
    );

    if i > 0 {
        delta + accumulation(&pseudo_to_cose, i - 1)
    } else {
        delta
    }
}

/// 理想的なキーストローク・キーストロークが綴り要素の何番目に属するか
fn spell_elements_index_of_key_stroke_delta(
    count_of_spell_elements: &[usize],
    key_stroke_delta: usize,
) -> usize {
    let mut accum = 0;
    for (i, count) in count_of_spell_elements.iter().enumerate() {
        accum += count;
        if key_stroke_delta <= accum {
            return i;
        }
    }

    // 綴り要素のどれかにキーストロークは属するはずである
    unreachable!();
}

/// 綴りのどの位置に属すかという観点で擬似的な綴り要素ごとの個数を構築する
/// ex. 「きょ」という綴りの「kyo」というキーストロークの綴り要素は1つだが
/// 位置変換という文脈ではkは0番目に属しyoは1番目に属する
fn construct_pseudo_count_of_spell_elements(
    count_of_spell_elements: &[usize],
    spell: usize,
) -> Vec<usize> {
    if count_of_spell_elements.len() == spell {
        return count_of_spell_elements.to_vec();
    }

    assert_eq!(count_of_spell_elements.len(), 1);
    assert_eq!(spell, 2);

    let mut v = vec![0; spell];

    for i in 1..=count_of_spell_elements[0] {
        let spell_delta =
            convert_key_stroke_delta_to_spell_delta(count_of_spell_elements, spell, i);
        v[spell_delta - 1] += 1;
    }

    v
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn multi_target_delta_converter_1() {
        let m = MultiTargetDeltaConverter::new(2, vec![3], vec![2, 3], BaseTarget::Chunk);

        assert_eq!(m.chunk_delta(vec![1]), vec![1]);
        assert_eq!(m.spell_delta(vec![1]), vec![2]);
        assert_eq!(m.ideal_key_stroke_delta(vec![1]), vec![3]);
        assert_eq!(m.key_stroke_delta(vec![1]), vec![5]);
    }

    #[test]
    fn multi_target_delta_converter_2() {
        let m = MultiTargetDeltaConverter::new(2, vec![3], vec![2, 3], BaseTarget::Spell);

        assert_eq!(m.chunk_delta(vec![1, 2]), vec![1, 1]);

        assert_eq!(m.spell_delta(vec![1, 2]), vec![1, 2]);
        assert_eq!(m.ideal_key_stroke_delta(vec![1, 2]), vec![3, 3]);
        assert_eq!(m.key_stroke_delta(vec![1, 2]), vec![2, 5]);
    }

    #[test]
    fn multi_target_delta_converter_3() {
        let m = MultiTargetDeltaConverter::new(2, vec![3], vec![2, 3], BaseTarget::IdealKeyStroke);

        assert_eq!(m.chunk_delta(vec![1, 2, 3]), vec![1, 1, 1]);
        assert_eq!(m.spell_delta(vec![1, 2, 3]), vec![1, 2, 2]);
        assert_eq!(m.ideal_key_stroke_delta(vec![1, 2, 3]), vec![1, 2, 3]);
        assert_eq!(m.key_stroke_delta(vec![1, 2, 3]), vec![2, 4, 5]);
    }

    #[test]
    fn multi_target_delta_converter_4() {
        let m = MultiTargetDeltaConverter::new(2, vec![3], vec![2, 3], BaseTarget::KeyStroke);

        assert_eq!(m.chunk_delta(vec![1, 2, 3, 4, 5]), vec![1, 1, 1, 1, 1]);
        assert_eq!(m.spell_delta(vec![1, 2, 3, 4, 5]), vec![1, 1, 2, 2, 2]);
        assert_eq!(
            m.ideal_key_stroke_delta(vec![1, 2, 3, 4, 5]),
            vec![1, 1, 2, 3, 3]
        );
        assert_eq!(m.key_stroke_delta(vec![1, 2, 3, 4, 5]), vec![1, 2, 3, 4, 5]);
    }
}
