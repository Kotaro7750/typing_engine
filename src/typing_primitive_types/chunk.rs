use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::typing_primitive_types::chunk_key_stroke_dictionary::CHUNK_SPELL_TO_KEY_STROKE_DICTIONARY;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
use crate::typing_primitive_types::spell::SpellString;
use key_stroke_candidate::{ChunkKeyStrokeCandidate, DelayedConfirmedCandidateInfo};
use single_n_availability::SingleNAvailability;

pub(crate) mod confirmed;
pub(crate) mod has_actual_key_strokes;
pub(crate) mod key_stroke_candidate;
mod single_n_availability;
pub(crate) mod typed;

#[cfg(test)]
mod test;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// An enum representing possible spell of a chunk.
pub(crate) enum ChunkSpell {
    DisplayableAscii(SpellString),
    SingleChar(SpellString),
    DoubleChar(SpellString),
}

impl ChunkSpell {
    fn new(ss: SpellString) -> Self {
        if ss.contains_displayable_ascii() {
            assert!(ss.chars().count() == 1);
            Self::DisplayableAscii(ss)
        } else {
            match ss.chars().count() {
                1 => Self::SingleChar(ss),
                2 => Self::DoubleChar(ss),
                _ => unreachable!("ChunkSpell's length must be 1 or 2"),
            }
        }
    }

    /// Split ChunkSpell::DoubleChar into two spells.
    fn split_double_char(&self) -> (SpellString, SpellString) {
        match self {
            Self::DoubleChar(spell_string) => (
                spell_string
                    .chars()
                    .next()
                    .unwrap()
                    .to_string()
                    .try_into()
                    .unwrap(),
                spell_string
                    .chars()
                    .nth(1)
                    .unwrap()
                    .to_string()
                    .try_into()
                    .unwrap(),
            ),
            _ => panic!("cannot split this ChunkSpell type"),
        }
    }

    /// Returns the number of characters in this spell.
    pub(crate) fn count(&self) -> usize {
        match self {
            ChunkSpell::DoubleChar(_) => 2,
            _ => 1,
        }
    }
}

impl AsRef<SpellString> for ChunkSpell {
    fn as_ref(&self) -> &SpellString {
        match self {
            ChunkSpell::DisplayableAscii(ss)
            | ChunkSpell::SingleChar(ss)
            | ChunkSpell::DoubleChar(ss) => ss,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct representing a fundamental typing unit.
/// For alphabet, it is just a single character.
/// For Japanese, it can be a single character or a combination of two characters such as "きょ".
pub struct Chunk {
    spell: ChunkSpell,
    /// Candidates of key strokes to type this chunk.
    /// Ex. For a chunk "きょ", there are key strokes like "kyo" and "kilyo".
    key_stroke_candidates: Option<Vec<ChunkKeyStrokeCandidate>>,
    /// A key stroke candidate that is the shortest when typed.
    /// This is determined when key strokes are assigned, so it may not be possible to type this
    /// candidate depending on the actual key stroke sequence.
    ideal_candidate: Option<ChunkKeyStrokeCandidate>,
}

impl Chunk {
    pub fn new(
        spell: SpellString,
        key_stroke_candidates: Option<Vec<ChunkKeyStrokeCandidate>>,
        ideal_candidate: Option<ChunkKeyStrokeCandidate>,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            key_stroke_candidates,
            ideal_candidate,
        }
    }

    /// Returns the spell of this chunk.
    pub(crate) fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    /// Returns key stroke candidates of this chunk.
    pub(crate) fn all_key_stroke_candidates(&self) -> &Option<Vec<ChunkKeyStrokeCandidate>> {
        &self.key_stroke_candidates
    }

    fn all_key_stroke_candidates_mut(&mut self) -> &mut Option<Vec<ChunkKeyStrokeCandidate>> {
        &mut self.key_stroke_candidates
    }

    pub(crate) fn key_stroke_candidates_mut(
        &mut self,
    ) -> Option<Vec<&mut ChunkKeyStrokeCandidate>> {
        self.key_stroke_candidates.as_mut().map(|candidates| {
            candidates
                .as_mut_slice()
                .iter_mut()
                .filter(|candidate| candidate.is_active())
                .collect()
        })
    }

    pub(crate) fn key_stroke_candidates(&self) -> Option<Vec<&ChunkKeyStrokeCandidate>> {
        self.key_stroke_candidates.as_ref().map(|candidates| {
            candidates
                .as_slice()
                .iter()
                .filter(|candidate| candidate.is_active())
                .collect()
        })
    }

    pub(crate) fn change_state_to_typed(&mut self) {
        self.all_key_stroke_candidates_mut()
            .as_mut()
            .unwrap()
            .iter_mut()
            .for_each(|candidate| {
                candidate.advance_cursor();
            });
    }

    /// Returns the ideal key stroke candidate of this chunk.
    pub(crate) fn ideal_key_stroke_candidate(&self) -> &Option<ChunkKeyStrokeCandidate> {
        &self.ideal_candidate
    }

    /// Returns the estimated minimum number of key strokes required to type this chunk.
    /// This is just an estimate because actual key strokes are not assigned yet.
    pub fn estimate_min_key_stroke_count(&self) -> usize {
        assert!(self.key_stroke_candidates().is_none());

        // ここで推測するのはあくまでも最小なので基本的には変換辞書から引いたものをそのまま使う
        // これは，2文字のチャンクの最小キーストロークは2文字をまとめて打つものだからである
        // 「っ」は次のチャンクによっては1回のキーストロークで打てるため1回としてカウントする
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

    /// Returns the key stroke candidate that is the shortest when typed and satisfies the chunk
    /// head restriction.
    /// When there are multiple candidates with the same key stroke count, the one that appears
    /// earlier is selected.
    pub(crate) fn min_candidate(
        &self,
        chunk_head_striction: Option<KeyStrokeChar>,
    ) -> &ChunkKeyStrokeCandidate {
        assert!(self.key_stroke_candidates().is_some());

        let min_candidate = self
            .key_stroke_candidates()
            .unwrap()
            .into_iter()
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

    /// Calculate the minimum number of key strokes required to type this chunk.
    /// This is calculated by selecting the shortest key stroke candidate.
    pub fn calc_min_key_stroke_count(&self) -> usize {
        self.min_candidate(None).calc_key_stroke_count()
    }

    /// 最後のチャンクに使うことを想定している
    /// Restrict the candidates of this chunk by the number of key strokes.
    /// Ex. When chunk is "し", there are candidates like "si", "shi", "ci", but when restricted to
    /// 1 key stroke, candidates becomes "s" and "c".
    ///
    /// This is assumed to be used for the last chunk.
    pub(crate) fn strict_key_stroke_count(&mut self, key_stroke_count_striction: NonZeroUsize) {
        // 制限によって必要キーストローク数が増えてはいけない
        assert!(key_stroke_count_striction.get() <= self.calc_min_key_stroke_count());

        let mut new_key_stroke_candidates = self.key_stroke_candidates.as_ref().unwrap().clone();

        new_key_stroke_candidates
            .iter_mut()
            // 変更するのは基本的には制限よりも長い候補のみでいい
            // 遅延確定候補は制限と同じタイプ数であっても通常の候補にする必要がある
            // 通常の候補にしないと制限だけタイプしても確定しなくなってしまう
            .filter(|candidate| {
                candidate.calc_key_stroke_count() > key_stroke_count_striction.get()
                    || candidate.is_delayed_confirmed_candidate()
            })
            .for_each(|candidate| candidate.strict_key_stroke_count(key_stroke_count_striction));

        // 制限の結果重複するキーストロークが生じる可能性があるので縮退させる
        let mut exists_in_candidates: HashSet<String> = HashSet::new();
        new_key_stroke_candidates.retain(|candidate| {
            let whole_key_stroke = candidate.whole_key_stroke().to_string();
            if exists_in_candidates.contains(&whole_key_stroke) {
                false
            } else {
                exists_in_candidates.insert(whole_key_stroke);
                true
            }
        });

        self.ideal_candidate
            .replace(new_key_stroke_candidates.get(0).unwrap().clone());

        self.key_stroke_candidates
            .replace(new_key_stroke_candidates);
    }

    /// Restrict the candidates of this chunk by the key stroke of chunk head.
    /// Ex. If the chunk_head_striction is "s", the candidates that do not start with "s" are removed.
    pub(crate) fn strict_chunk_head(&mut self, chunk_head_striction: KeyStrokeChar) {
        self.key_stroke_candidates_mut()
            .as_mut()
            .unwrap()
            .iter_mut()
            .for_each(|candidate| {
                if candidate.key_stroke_char_at_position(0) != chunk_head_striction {
                    candidate.inactivate();
                }
            });
    }

    /// Reduce the candidates of this chunk.
    /// Retain only the candidates whose index is true in the retain_vector.
    pub(crate) fn reduce_candidate(&mut self, retain_vector: &[bool]) {
        self.all_key_stroke_candidates_mut()
            .as_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
            .filter(|(i, _)| !retain_vector[*i])
            .for_each(|(_, candidate)| {
                candidate.inactivate();
            });
    }
}

// 綴りのみの不完全なチャンク列にキーストローク候補を追加する
pub fn append_key_stroke_to_chunks(chunks: &mut [Chunk]) {
    let mut next_chunk_spell: Option<ChunkSpell> = None;

    // 次のチャンク先頭のキーストローク
    let mut next_chunk_head_key_strokes: Option<Vec<KeyStrokeChar>> = None;

    // このチャンクが「っ」としたときにキーストロークの連続によって表現できるキーストローク群
    // 次のチャンク先頭の子音などのキーストロークともいえる
    // ex. 次のチャンクが「た」だったときには [t] となる
    let mut key_strokes_can_represent_ltu_by_repeat: Option<Vec<KeyStrokeChar>> = None;

    // キーストローク候補は次のチャンクに依存するので後ろから走査する
    for chunk in chunks.iter_mut().rev() {
        assert!(chunk.key_stroke_candidates().is_none());

        let mut key_stroke_candidates = Vec::<ChunkKeyStrokeCandidate>::new();

        match &chunk.spell {
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

                let (first_spell_string, second_spell_string) = chunk.spell.split_double_char();

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

        // タイプ数が少ないキーストロークを第一候補として選択する
        key_stroke_candidates.sort_by(|a, b| {
            a.calc_key_stroke_count()
                .partial_cmp(&(b.calc_key_stroke_count()))
                .unwrap()
        });

        chunk.key_stroke_candidates.replace(key_stroke_candidates);

        next_chunk_spell.replace(chunk.spell.clone());

        // 次のチャンク先頭のキーストロークを更新する
        next_chunk_head_key_strokes.replace(vec![]);

        let mut already_pushed_next_chunk_head_key_strokes = HashSet::<KeyStrokeChar>::new();
        chunk
            .key_stroke_candidates()
            .unwrap()
            .iter()
            .for_each(|key_stroke_candidate| {
                let first_char = key_stroke_candidate.key_stroke_char_at_position(0);
                if !already_pushed_next_chunk_head_key_strokes.contains(&first_char) {
                    already_pushed_next_chunk_head_key_strokes.insert(first_char.clone());
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
                    match &chunk.spell {
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
    }

    append_ideal_candidates_to_chunks(chunks);
}

/// 理想的なキーストローク候補をチャンク列に付与する
/// 候補が削減されていないことを前提とする
fn append_ideal_candidates_to_chunks(chunks: &mut [Chunk]) {
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

    chunks.iter_mut().for_each(|chunk| {
        let ideal_candidate = chunk.min_candidate(next_chunk_head_constraint.clone());
        next_chunk_head_constraint = ideal_candidate.next_chunk_head_constraint().clone();

        chunk.ideal_candidate = Some(ideal_candidate.clone());
    });
}
