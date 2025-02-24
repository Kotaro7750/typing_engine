use std::collections::HashSet;
use std::num::NonZeroUsize;

use super::key_stroke_candidate::ChunkKeyStrokeCandidate;
use super::Chunk;
use super::ChunkState;
use crate::typing_primitive_types::chunk::ChunkSpell;
use crate::typing_primitive_types::key_stroke::KeyStrokeChar;
use crate::typing_primitive_types::spell::SpellString;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A struct representing a fundamental typing unit.
/// For alphabet, it is just a single character.
/// For Japanese, it can be a single character or a combination of two characters such as "きょ".
pub(crate) struct ChunkUnprocessed {
    spell: ChunkSpell,
    /// Candidates of key strokes to type this chunk.
    /// Ex. For a chunk "きょ", there are key strokes like "kyo" and "kilyo".
    key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
    /// A key stroke candidate that is the shortest when typed.
    /// This is determined when key strokes are assigned, so it may not be possible to type this
    /// candidate depending on the actual key stroke sequence.
    ideal_candidate: ChunkKeyStrokeCandidate,
}

impl ChunkUnprocessed {
    pub fn new(
        spell: SpellString,
        key_stroke_candidates: Vec<ChunkKeyStrokeCandidate>,
        ideal_candidate: ChunkKeyStrokeCandidate,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            key_stroke_candidates,
            ideal_candidate,
        }
    }

    pub(crate) fn into_inflight(mut self) -> Chunk {
        self.all_key_stroke_candidates_mut()
            .iter_mut()
            .for_each(|candidate| {
                candidate.advance_cursor();
            });

        Chunk::new(
            self.spell.into(),
            self.key_stroke_candidates,
            self.ideal_candidate,
            ChunkState::Inflight,
            Some(vec![]),
        )
    }

    /// Returns the spell of this chunk.
    pub(crate) fn spell(&self) -> &ChunkSpell {
        &self.spell
    }

    fn all_key_stroke_candidates_mut(&mut self) -> &mut [ChunkKeyStrokeCandidate] {
        &mut self.key_stroke_candidates
    }

    pub(crate) fn key_stroke_candidates_mut(&mut self) -> Vec<&mut ChunkKeyStrokeCandidate> {
        self.key_stroke_candidates
            .iter_mut()
            .filter(|candidate| candidate.is_active())
            .collect()
    }

    pub(crate) fn key_stroke_candidates(&self) -> Vec<&ChunkKeyStrokeCandidate> {
        self.key_stroke_candidates
            .iter()
            .filter(|candidate| candidate.is_active())
            .collect()
    }

    /// Returns the ideal key stroke candidate of this chunk.
    pub(crate) fn ideal_key_stroke_candidate(&self) -> &ChunkKeyStrokeCandidate {
        &self.ideal_candidate
    }

    /// Returns the key stroke candidate that is the shortest when typed and satisfies the chunk
    /// head restriction.
    /// When there are multiple candidates with the same key stroke count, the one that appears
    /// earlier is selected.
    pub(crate) fn min_candidate(
        &self,
        chunk_head_striction: Option<KeyStrokeChar>,
    ) -> &ChunkKeyStrokeCandidate {
        let min_candidate = self
            .key_stroke_candidates()
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

        let mut new_key_stroke_candidates = self.key_stroke_candidates.clone();

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

        self.ideal_candidate = new_key_stroke_candidates.first().unwrap().clone();

        self.key_stroke_candidates = new_key_stroke_candidates;
    }

    /// Restrict the candidates of this chunk by the key stroke of chunk head.
    /// Ex. If the chunk_head_striction is "s", the candidates that do not start with "s" are removed.
    pub(crate) fn strict_chunk_head(&mut self, chunk_head_striction: KeyStrokeChar) {
        self.key_stroke_candidates_mut()
            .iter_mut()
            .for_each(|candidate| {
                if candidate.key_stroke_char_at_position(0) != chunk_head_striction {
                    candidate.inactivate();
                }
            });
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroUsize;

    use crate::{gen_candidate, gen_chunk_unprocessed};

    #[test]
    fn strict_key_stroke_count_1() {
        let mut chunk = gen_chunk_unprocessed!(
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
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk_unprocessed!(
                "じょ",
                vec![
                    gen_candidate!(["j"], true, None),
                    gen_candidate!(["z"], true, None),
                ],
                gen_candidate!(["j"], true, None)
            )
        )
    }

    #[test]
    fn strict_key_stroke_count_2() {
        let mut chunk = gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(["n"], true, None, ['j', 'z']),
                gen_candidate!(["nn"], true, None),
                gen_candidate!(["xn"], true, None),
            ],
            gen_candidate!(["n"], true, None, ['j', 'z'])
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk_unprocessed!(
                "ん",
                vec![
                    gen_candidate!(["n"], true, None),
                    gen_candidate!(["x"], true, None)
                ],
                gen_candidate!(["n"], true, None)
            )
        )
    }
}
