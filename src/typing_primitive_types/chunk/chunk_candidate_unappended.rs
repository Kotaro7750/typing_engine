use std::collections::{HashSet, VecDeque};

use super::unprocessed::ChunkUnprocessed;
use crate::typing_primitive_types::chunk::key_stroke_candidate::{
    ChunkKeyStrokeCandidate, DelayedConfirmedCandidateInfo,
};
use crate::typing_primitive_types::chunk::single_n_availability::SingleNAvailability;
use crate::typing_primitive_types::chunk::ChunkSpell;
use crate::typing_primitive_types::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
use crate::typing_primitive_types::spell::SpellString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct representing a chunk before key stroke candidates are appended.
pub(crate) struct ChunkCandidateUnappended {
    /// The spell of this chunk.
    spell: ChunkSpell,
}

impl ChunkCandidateUnappended {
    pub(crate) fn new(spell: SpellString) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
        }
    }

    /// Returns the spell of this chunk.
    fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    /// Returns the estimated minimum number of key strokes required to type this chunk.
    /// This is just an estimate because actual key strokes are not assigned yet.
    pub(crate) fn estimate_min_key_stroke_count(&self) -> usize {
        // Basically, estimation is done by using the number of key strokes in the conversion
        // dictionary because the minimum number of key strokes for a 2-character chunk is to type
        // 2 characters together.
        // "っ" is counted as 1 key stroke because it can be typed in 1 key stroke depending on the
        // next chunk.
        match &self.spell {
            ChunkSpell::DisplayableAscii(_) => 1,
            ChunkSpell::SingleChar(spell_string) | ChunkSpell::DoubleChar(spell_string) => {
                if spell_string.as_str() == "っ" {
                    1
                } else {
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get(spell_string.as_str())
                        .unwrap()
                        .iter()
                        .map(|key_stroke_str| key_stroke_str.chars().count())
                        .min()
                        .unwrap()
                }
            }
        }
    }

    /// Append key stroke candidates and generate ChunkIdealCandidateUnappended.
    fn append_candidate(
        &self,
        key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
    ) -> ChunkIdealCandidateUnappended {
        ChunkIdealCandidateUnappended {
            spell: self.spell.clone(),
            key_stroke_candidates,
        }
    }
}

/// A intermediate struct representing a chunk before key stroke candidates are appended but ideal
/// candidate is not appended yet.
struct ChunkIdealCandidateUnappended {
    /// The spell of this chunk.
    spell: ChunkSpell,
    /// The key stroke candidates of this chunk.
    key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
}

impl ChunkIdealCandidateUnappended {
    fn key_stroke_candidates(&self) -> &[ChunkKeyStrokeCandidate] {
        &self.key_stroke_candidates
    }

    /// Returns the key stroke candidate that is the shortest when typed and satisfies the chunk
    /// head restriction.
    /// When there are multiple candidates with the same key stroke count, the one that appears
    /// earlier is selected.
    fn min_candidate(
        &self,
        chunk_head_striction: Option<KeyStrokeChar>,
    ) -> &ChunkKeyStrokeCandidate {
        let min_candidate = self
            .key_stroke_candidates()
            .iter()
            .filter(|candidate| {
                if let Some(chunk_head_striction) = &chunk_head_striction {
                    &candidate.key_stroke_char_at_position(0) == chunk_head_striction
                } else {
                    true
                }
            })
            .reduce(|min_candidate, candidate| {
                if candidate.calc_key_stroke_count() < min_candidate.calc_key_stroke_count() {
                    candidate
                } else {
                    min_candidate
                }
            });

        assert!(min_candidate.is_some());

        min_candidate.as_ref().unwrap()
    }
}

// 綴りのみの不完全なチャンク列にキーストローク候補を追加する
pub(crate) fn append_key_stroke_to_chunks(
    chunks_key_stroke_unappended: &[ChunkCandidateUnappended],
) -> Vec<ChunkUnprocessed> {
    let mut chunks_ideal_candidate_unappended = VecDeque::<ChunkIdealCandidateUnappended>::new();

    // Because key stroke candidate of chunk are depend on the next chunk, we need to process
    // reverse order.

    // Information of the next chunk.
    // Due to reverse order processing, the next chunk is the previous processed chunk.
    let mut next_chunk_spell: Option<ChunkSpell> = None;
    // The head key strokes of the next chunk.
    let mut next_chunk_head_key_strokes: Option<Vec<KeyStrokeChar>> = None;
    // The key strokes that can represent "っ" by repeating key strokes.
    // In other words, the key strokes of the next chunk head consonant.
    // ex. If the next chunk is "た", it is [t].
    let mut key_strokes_can_represent_ltu_by_repeat: Option<Vec<KeyStrokeChar>> = None;

    for chunk_key_stroke_unappended in chunks_key_stroke_unappended.iter().rev() {
        let mut key_stroke_candidates = Vec::<ChunkKeyStrokeCandidate>::new();

        // Generate key stroke candidates based on the spell of the chunk.
        match chunk_key_stroke_unappended.spell() {
            // 表示可能なASCIIで構成されるチャンクならそのままキーストロークにする
            ChunkSpell::DisplayableAscii(spell_string) => {
                key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                    vec![String::from(spell_string.clone()).try_into().unwrap()],
                    None,
                    None,
                    true,
                    None,
                ));
            }
            ChunkSpell::SingleChar(spell_string) => match spell_string.chars().as_str() {
                "ん" => {
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get("ん")
                        .unwrap()
                        .iter()
                        // 「n」というキーストロークは次のチャンクによっては使えない
                        .filter_map(|key_stroke| match *key_stroke {
                            "n" => {
                                let single_n_avail =
                                    SingleNAvailability::check_single_n_availability(
                                        &next_chunk_spell,
                                        next_chunk_head_key_strokes.as_ref(),
                                    );

                                match single_n_avail {
                                    SingleNAvailability::All(avail_as_next_key_strokes) => {
                                        Some((key_stroke, None, Some(avail_as_next_key_strokes)))
                                    }
                                    SingleNAvailability::Partial(avail_as_next_key_strokes) => {
                                        Some((
                                            key_stroke,
                                            Some(avail_as_next_key_strokes[0].clone()),
                                            Some(avail_as_next_key_strokes),
                                        ))
                                    }
                                    SingleNAvailability::Cannot => None,
                                }
                            }
                            _ => Some((key_stroke, None, None)),
                        })
                        .for_each(
                            |(
                                key_stroke,
                                next_chunk_head_constraint,
                                avail_as_next_key_strokes,
                            )| {
                                key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                    vec![key_stroke.to_string().try_into().unwrap()],
                                    next_chunk_head_constraint,
                                    avail_as_next_key_strokes
                                        .map(DelayedConfirmedCandidateInfo::new),
                                    true,
                                    None,
                                ))
                            },
                        );
                }
                // 「っ」は単独で打つ以外にも次のチャンクの子音で済ませる(「った」なら「tta」)ことができる
                "っ" => {
                    // 「ltu」「ltsu」「xtu」は任意の状況で次のチャンクへの制限なしに打てる
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get("っ")
                        .unwrap()
                        .iter()
                        .for_each(|key_stroke| {
                            key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![key_stroke.to_string().try_into().unwrap()],
                                None,
                                None,
                                true,
                                None,
                            ))
                        });

                    // 子音の連続で打つ場合には次のチャンクへの制限をする
                    if let Some(ref key_strokes_can_represent_ltu_by_repeat) =
                        key_strokes_can_represent_ltu_by_repeat
                    {
                        key_strokes_can_represent_ltu_by_repeat
                            .iter()
                            .for_each(|key_stroke| match char::from(key_stroke.clone()) {
                                'l' | 'x' => {
                                    key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                        vec![char::from(key_stroke.clone())
                                            .to_string()
                                            .try_into()
                                            .unwrap()],
                                        Some(key_stroke.clone()),
                                        // 次のチャンクへの制限があるときには遅延確定候補を確定できるのはその制限だけである
                                        Some(DelayedConfirmedCandidateInfo::new(
                                            next_chunk_head_key_strokes
                                                .as_ref()
                                                .map_or(&vec![], |v| v)
                                                .iter()
                                                .filter(|ks| *ks == key_stroke)
                                                .cloned()
                                                .collect(),
                                        )),
                                        true,
                                        None,
                                    ))
                                }
                                _ => key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                    vec![char::from(key_stroke.clone())
                                        .to_string()
                                        .try_into()
                                        .unwrap()],
                                    Some(key_stroke.clone()),
                                    None,
                                    true,
                                    None,
                                )),
                            });
                    }
                }
                _ => {
                    CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                        .get(spell_string.as_str())
                        .unwrap()
                        .iter()
                        .for_each(|key_stroke| {
                            key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                vec![key_stroke.to_string().try_into().unwrap()],
                                None,
                                None,
                                true,
                                None,
                            ));
                        });
                }
            },
            // 2文字のチャンクはまとめて入力する場合と1文字ずつ入力する場合がある
            ChunkSpell::DoubleChar(spell_string) => {
                // まとめて入力できるキーストローク
                CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                    .get(spell_string.as_str())
                    .unwrap()
                    .iter()
                    .for_each(|key_stroke| {
                        key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                            vec![key_stroke.to_string().try_into().unwrap()],
                            None,
                            None,
                            true,
                            None,
                        ));
                    });

                let (first_spell_string, second_spell_string) =
                    chunk_key_stroke_unappended.spell.split_double_char();

                // 1文字ずつのキーストローク
                CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                    .get(first_spell_string.as_str())
                    .unwrap()
                    .iter()
                    .for_each(|first_key_stroke| {
                        CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY
                            .get(second_spell_string.as_str())
                            .unwrap()
                            .iter()
                            .for_each(|second_key_stroke| {
                                key_stroke_candidates.push(ChunkKeyStrokeCandidate::new(
                                    vec![
                                        first_key_stroke.to_string().try_into().unwrap(),
                                        second_key_stroke.to_string().try_into().unwrap(),
                                    ],
                                    None,
                                    None,
                                    true,
                                    None,
                                ));
                            });
                    });
            }
        }

        // Select the candidate with the fewest key strokes as the first candidate.
        key_stroke_candidates.sort_by(|a, b| {
            a.calc_key_stroke_count()
                .partial_cmp(&(b.calc_key_stroke_count()))
                .unwrap()
        });

        // Update the "next" chunk information for the "previous" chunk.

        next_chunk_spell.replace(chunk_key_stroke_unappended.spell.clone());

        // Update the head key strokes of the "next" chunk.
        next_chunk_head_key_strokes.replace(vec![]);
        let mut next_chunk_head_key_strokes_set = HashSet::<KeyStrokeChar>::new();
        key_stroke_candidates
            .iter()
            .for_each(|key_stroke_candidate| {
                let first_char = key_stroke_candidate.key_stroke_char_at_position(0);
                if next_chunk_head_key_strokes_set.insert(first_char.clone()) {
                    next_chunk_head_key_strokes
                        .as_mut()
                        .unwrap()
                        .push(first_char);
                }
            });

        key_strokes_can_represent_ltu_by_repeat.replace(
            next_chunk_head_key_strokes
                .as_ref()
                .unwrap()
                .iter()
                .filter(|ksc| {
                    match &chunk_key_stroke_unappended.spell {
                        ChunkSpell::SingleChar(_) | ChunkSpell::DoubleChar(_) =>
                        // 直後のチャンクの先頭が「n」を除く子音だった場合に「っ」を子音の連続で表すことができる
                        {
                            **ksc != 'a'
                                && **ksc != 'i'
                                && **ksc != 'u'
                                && **ksc != 'e'
                                && **ksc != 'o'
                                && **ksc != 'n'
                        }
                        // 直後のチャンクがASCIIだったら子音の連続で表すことはできない
                        ChunkSpell::DisplayableAscii(_) => false,
                    }
                })
                .cloned()
                .collect(),
        );

        chunks_ideal_candidate_unappended
            .push_front(chunk_key_stroke_unappended.append_candidate(key_stroke_candidates));
    }

    append_ideal_candidates_to_chunks(chunks_ideal_candidate_unappended.into())
}

/// 理想的なキーストローク候補をチャンク列に付与する
/// 候補が削減されていないことを前提とする
fn append_ideal_candidates_to_chunks(
    chunks_ideal_candidate_unappended: Vec<ChunkIdealCandidateUnappended>,
) -> Vec<ChunkUnprocessed> {
    // 本来なら理想的なキーストローク候補は全探索によって付与されるべきであるが計算量の観点から前のチャンクから貪欲に行うことで付与している
    // このことによって理想的ではないキーストローク候補が付与されてしまう可能性は以下の理由からないと言える
    //
    // チャンク列を処理していったときに次チャンクへの制限がない場合にはチャンク内で最短となる候補が理想的である
    // 次チャンクへの制限を持つ候補があるチャンクに遭遇したときにはそのチャンク内で最短となる候補が理想的であり
    // もしその候補が次チャンクへの制限があった場合には次のチャンクで選択の対象とする候補は制限によって削減する必要がある
    //
    // 次チャンクへの制限を持つ候補があるチャンクの次のチャンクでは制限によって削減される候補群（A）とそうでない候補（B）がある
    // このときAの最短キーストローク数がBの最短キーストローク数と比べて「制限を持つ候補によって短縮することのできるキーストローク数」分より大きい場合には前から貪欲にやってはならない
    //
    // XXX 現在の実装では「ん」には制限を持つ候補はない
    // しかし次チャンクへの制限を持つ候補がある「っ」「ん」の次のチャンクでAとB両方の候補を持つのはそれぞれ「い(AがiでBがyi)」「う(AがuでBがwuなど)」と「う」だけであり
    // これらのAとBの最短キーストローク数の差は制限を持つ候補による短縮分以下である

    let mut next_chunk_head_constraint: Option<KeyStrokeChar> = None;

    chunks_ideal_candidate_unappended
        .into_iter()
        .map(|chunk_ideal_candidate_unappended| {
            let ideal_candidate = chunk_ideal_candidate_unappended
                .min_candidate(next_chunk_head_constraint.clone())
                .clone();
            next_chunk_head_constraint = ideal_candidate.next_chunk_head_constraint().clone();

            ChunkUnprocessed::new(
                chunk_ideal_candidate_unappended.spell.into(),
                chunk_ideal_candidate_unappended.key_stroke_candidates,
                ideal_candidate,
            )
        })
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{gen_candidate, gen_chunk_candidate_unappended, gen_chunk_unprocessed};

    #[test]
    fn append_key_stroke_to_chunks_1() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("じょ"),
            gen_chunk_candidate_unappended!("ん"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "じょ",
                    vec![
                        gen_candidate!(["jo"], true, None),
                        gen_candidate!(["zyo"], true, None),
                        gen_candidate!(["jyo"], true, None),
                        gen_candidate!(["zi", "lyo"], true, None),
                        gen_candidate!(["zi", "xyo"], true, None),
                        gen_candidate!(["ji", "lyo"], true, None),
                        gen_candidate!(["ji", "xyo"], true, None),
                    ],
                    gen_candidate!(["jo"], true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None),
                    ],
                    gen_candidate!(["nn"], true, None)
                )
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_2() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("う"),
            gen_chunk_candidate_unappended!("っ"),
            gen_chunk_candidate_unappended!("う"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, None),
                        gen_candidate!(["wu"], true, None),
                        gen_candidate!(["whu"], true, None)
                    ],
                    gen_candidate!(["u"], true, None)
                ),
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(["w"], true, None, 'w'),
                        gen_candidate!(["ltu"], true, None),
                        gen_candidate!(["xtu"], true, None),
                        gen_candidate!(["ltsu"], true, None)
                    ],
                    gen_candidate!(["w"], true, None, 'w')
                ),
                gen_chunk_unprocessed!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, None),
                        gen_candidate!(["wu"], true, None),
                        gen_candidate!(["whu"], true, None)
                    ],
                    gen_candidate!(["wu"], true, None)
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_3() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("か"),
            gen_chunk_candidate_unappended!("ん"),
            gen_chunk_candidate_unappended!("じ"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, None),
                        gen_candidate!(["ca"], true, None)
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['z', 'j']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['z', 'j'])
                ),
                gen_chunk_unprocessed!(
                    "じ",
                    vec![
                        gen_candidate!(["zi"], true, None),
                        gen_candidate!(["ji"], true, None)
                    ],
                    gen_candidate!(["zi"], true, None)
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_4() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("B"),
            gen_chunk_candidate_unappended!("i"),
            gen_chunk_candidate_unappended!("g"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "B",
                    vec![gen_candidate!(["B"], true, None)],
                    gen_candidate!(["B"], true, None)
                ),
                gen_chunk_unprocessed!(
                    "i",
                    vec![gen_candidate!(["i"], true, None)],
                    gen_candidate!(["i"], true, None)
                ),
                gen_chunk_unprocessed!(
                    "g",
                    vec![gen_candidate!(["g"], true, None)],
                    gen_candidate!(["g"], true, None)
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_5() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("っ"),
            gen_chunk_candidate_unappended!("っ"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(["l"], true, None, 'l', ['l']),
                        gen_candidate!(["x"], true, None, 'x', ['x']),
                        gen_candidate!(["ltu"], true, None),
                        gen_candidate!(["xtu"], true, None),
                        gen_candidate!(["ltsu"], true, None),
                    ],
                    gen_candidate!(["l"], true, None, 'l', ['l'])
                ),
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(["ltu"], true, None),
                        gen_candidate!(["xtu"], true, None),
                        gen_candidate!(["ltsu"], true, None),
                    ],
                    gen_candidate!(["ltu"], true, None)
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_6() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("っ"),
            gen_chunk_candidate_unappended!("か"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "っ",
                    vec![
                        gen_candidate!(["k"], true, None, 'k'),
                        gen_candidate!(["c"], true, None, 'c'),
                        gen_candidate!(["ltu"], true, None),
                        gen_candidate!(["xtu"], true, None),
                        gen_candidate!(["ltsu"], true, None),
                    ],
                    gen_candidate!(["k"], true, None, 'k')
                ),
                gen_chunk_unprocessed!(
                    "か",
                    vec![
                        gen_candidate!(["ka"], true, None),
                        gen_candidate!(["ca"], true, None),
                    ],
                    gen_candidate!(["ka"], true, None)
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_7() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("い"),
            gen_chunk_candidate_unappended!("ん"),
            gen_chunk_candidate_unappended!("しょ"),
            gen_chunk_candidate_unappended!("う"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "い",
                    vec![
                        gen_candidate!(["i"], true, None),
                        gen_candidate!(["yi"], true, None),
                    ],
                    gen_candidate!(["i"], true, None)
                ),
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, ['s', 'c']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None)
                    ],
                    gen_candidate!(["n"], true, None, ['s', 'c'])
                ),
                gen_chunk_unprocessed!(
                    "しょ",
                    vec![
                        gen_candidate!(["syo"], true, None),
                        gen_candidate!(["sho"], true, None),
                        gen_candidate!(["si", "lyo"], true, None),
                        gen_candidate!(["si", "xyo"], true, None),
                        gen_candidate!(["ci", "lyo"], true, None),
                        gen_candidate!(["ci", "xyo"], true, None),
                        gen_candidate!(["shi", "lyo"], true, None),
                        gen_candidate!(["shi", "xyo"], true, None),
                    ],
                    gen_candidate!(["syo"], true, None)
                ),
                gen_chunk_unprocessed!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, None),
                        gen_candidate!(["wu"], true, None),
                        gen_candidate!(["whu"], true, None)
                    ],
                    gen_candidate!(["u"], true, None)
                ),
            ]
        );
    }

    #[test]
    fn append_key_stroke_to_chunks_8() {
        let mut chunks = vec![
            gen_chunk_candidate_unappended!("ん"),
            gen_chunk_candidate_unappended!("う"),
        ];

        let chunks = append_key_stroke_to_chunks(&mut chunks);

        assert_eq!(
            chunks,
            vec![
                gen_chunk_unprocessed!(
                    "ん",
                    vec![
                        gen_candidate!(["n"], true, None, 'w', ['w']),
                        gen_candidate!(["nn"], true, None),
                        gen_candidate!(["xn"], true, None),
                    ],
                    gen_candidate!(["n"], true, None, 'w', ['w'])
                ),
                gen_chunk_unprocessed!(
                    "う",
                    vec![
                        gen_candidate!(["u"], true, None),
                        gen_candidate!(["wu"], true, None),
                        gen_candidate!(["whu"], true, None)
                    ],
                    gen_candidate!(["wu"], true, None)
                ),
            ]
        );
    }
}
