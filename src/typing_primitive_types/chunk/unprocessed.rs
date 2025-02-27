use std::collections::HashSet;
use std::num::NonZeroUsize;

use super::inflight::ChunkInflight;
use super::key_stroke_candidate::{ChunkKeyStrokeCandidate, ChunkKeyStrokeCandidateWithoutCursor};
use super::Chunk;
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
    key_stroke_candidates: Vec<ChunkKeyStrokeCandidateWithoutCursor>,
    /// A key stroke candidate that is the shortest when typed.
    /// This is determined when key strokes are assigned, so it may not be possible to type this
    /// candidate depending on the actual key stroke sequence.
    ideal_candidate: ChunkKeyStrokeCandidateWithoutCursor,
}

impl ChunkUnprocessed {
    pub fn new(
        spell: SpellString,
        key_stroke_candidates: Vec<ChunkKeyStrokeCandidateWithoutCursor>,
        ideal_candidate: ChunkKeyStrokeCandidateWithoutCursor,
    ) -> Self {
        Self {
            spell: ChunkSpell::new(spell),
            key_stroke_candidates,
            ideal_candidate,
        }
    }

    /// Convert this chunk into inflight chunk.
    /// If the head_striction_from_previous is Some, the candidates that do not start with the
    /// key stroke char are inactive by default.
    /// Ex. If the chunk_head_striction is Some("s"), candidates that do not start with "s" are inactive.
    pub(crate) fn into_inflight(
        self,
        head_striction_from_previous: Option<KeyStrokeChar>,
    ) -> ChunkInflight {
        let (active_candidate, inactive_candidate): (
            Vec<ChunkKeyStrokeCandidateWithoutCursor>,
            Vec<ChunkKeyStrokeCandidateWithoutCursor>,
        ) = self
            .key_stroke_candidates
            .into_iter()
            .partition(|candidate| {
                if let Some(head_striction_from_previous) = &head_striction_from_previous {
                    candidate.key_stroke_char_at_position(0) == *head_striction_from_previous
                } else {
                    true
                }
            });

        ChunkInflight::new(
            self.spell.into(),
            active_candidate
                .into_iter()
                .map(|candidate| candidate.into_with_cursor(0))
                .collect(),
            inactive_candidate,
            self.ideal_candidate,
            vec![],
        )
    }

    pub(crate) fn key_stroke_candidates(&self) -> &[ChunkKeyStrokeCandidateWithoutCursor] {
        &self.key_stroke_candidates
    }

    /// Returns the ideal key stroke candidate of this chunk.
    pub(crate) fn ideal_key_stroke_candidate(&self) -> &ChunkKeyStrokeCandidateWithoutCursor {
        &self.ideal_candidate
    }

    /// Returns the key stroke candidate that is the shortest when typed and satisfies the chunk
    /// head restriction.
    /// When there are multiple candidates with the same key stroke count, the one that appears
    /// earlier is selected.
    pub(crate) fn min_candidate(
        &self,
        chunk_head_striction: Option<KeyStrokeChar>,
    ) -> &ChunkKeyStrokeCandidateWithoutCursor {
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
}

impl Chunk for ChunkUnprocessed {
    fn spell(&self) -> &ChunkSpell {
        &self.spell
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZeroUsize;

    use crate::{gen_candidate, gen_candidate_key_stroke, gen_chunk_unprocessed};

    #[test]
    fn strict_key_stroke_count_1() {
        let mut chunk = gen_chunk_unprocessed!(
            "じょ",
            vec![
                gen_candidate!(gen_candidate_key_stroke!(["jo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["zyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["jyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "lyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["zi", "xyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ji", "lyo"]), false),
                gen_candidate!(gen_candidate_key_stroke!(["ji", "xyo"]), false),
            ],
            gen_candidate!(gen_candidate_key_stroke!(["jo"]), false)
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk_unprocessed!(
                "じょ",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!(["j"]), false),
                    gen_candidate!(gen_candidate_key_stroke!(["z"]), false),
                ],
                gen_candidate!(gen_candidate_key_stroke!(["j"]), false)
            )
        )
    }

    #[test]
    fn strict_key_stroke_count_2() {
        let mut chunk = gen_chunk_unprocessed!(
            "ん",
            vec![
                gen_candidate!(gen_candidate_key_stroke!("n"), false, ['j', 'z']),
                gen_candidate!(gen_candidate_key_stroke!("nn"), false),
                gen_candidate!(gen_candidate_key_stroke!("xn"), false),
            ],
            gen_candidate!(gen_candidate_key_stroke!("n"), false, ['j', 'z'])
        );

        chunk.strict_key_stroke_count(NonZeroUsize::new(1).unwrap());

        assert_eq!(
            chunk,
            gen_chunk_unprocessed!(
                "ん",
                vec![
                    gen_candidate!(gen_candidate_key_stroke!("n"), false),
                    gen_candidate!(gen_candidate_key_stroke!("x"), false)
                ],
                gen_candidate!(gen_candidate_key_stroke!("n"), false)
            )
        )
    }
}
