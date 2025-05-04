use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::num::NonZeroUsize;

use crate::typing_primitive_types::spell::SpellString;
use crate::typing_primitive_types::vocabulary::{VocabularyEntry, VocabularySpellElement};

#[derive(Debug, Clone, PartialEq, Eq)]
enum VocabularyParseErrorKind {
    MultipleLines,
    ComponentsCountMisMatch,
    CompoundSymbolMisMatch,
    EmptyCompound,
    ViewAndSpellsCountMisMatch,
    InvalidSpellString(String),
    Internal(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VocabularyParseError {
    kind: VocabularyParseErrorKind,
}

impl VocabularyParseError {
    fn new(kind: VocabularyParseErrorKind) -> Self {
        Self { kind }
    }
}

impl Display for VocabularyParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            VocabularyParseErrorKind::MultipleLines => {
                write!(f, "Multiple lines in input, expected a single line")
            }
            VocabularyParseErrorKind::ComponentsCountMisMatch => write!(
                f,
                "Component count mismatch, expected 2 components separated by ':'"
            ),
            VocabularyParseErrorKind::CompoundSymbolMisMatch => write!(
                f,
                "Compound symbol mismatch, '[' and ']' don't match properly"
            ),
            VocabularyParseErrorKind::EmptyCompound => write!(f, "Empty compound is not allowed"),
            VocabularyParseErrorKind::ViewAndSpellsCountMisMatch => {
                write!(f, "View and spells count mismatch")
            }
            VocabularyParseErrorKind::InvalidSpellString(s) => {
                write!(f, "Invalid spell string: {}", s)
            }
            VocabularyParseErrorKind::Internal(s) => write!(f, "Internal error: {}", s),
        }
    }
}

impl Error for VocabularyParseError {}

/// Parses a single line into a [`VocabularyEntry`](VocabularyEntry).
///
/// Passed line must follow the format:
/// `word:reading1,reading2,...,readingN`
///
/// - The left-hand side of the colon (`:`) is the word to be displayed.
/// - The right-hand side is a comma-separated list of readings corresponding to each character or grouped characters in the word.
/// - Groups of characters that should be treated as a single unit (e.g., 熟字訓) must be enclosed in square brackets, e.g., `[明日]のジョー:あした,の,じ,ょ,ー`.
/// - Katakana words must use Hiragana for their readings.
/// - English words should be spelled out letter by letter in the reading section.
/// - Small kana characters (e.g., "ょ") count as individual units.
/// - Special characters used for formatting (`:`, `[`, `]`, `,`) can be escaped with a backslash (`\`) to be interpreted literally.
/// - A literal backslash (`\`) must be escaped as a double backslash (`\\`).
/// - If the number of readings does not match the number of character units (taking brackets and escapes into account), the line is considered invalid and an error ([`VocabularyParseError`](VocabularyParseError)) is returned.
///
/// ## Examples
///
/// ```
/// use typing_engine::parse_vocabulary_entry;
///
/// // Valid entries:
///
/// // Basic Japanese example
/// let result = parse_vocabulary_entry("頑張る:がん,ば,る");
/// assert!(result.is_ok());
///
/// // Katakana example
/// let result = parse_vocabulary_entry("タイピング:た,い,ぴ,ん,ぐ");
/// assert!(result.is_ok());
///
/// // Example with compound characters
/// let result = parse_vocabulary_entry("[明日]のジョー:あした,の,じ,ょ,ー");
/// assert!(result.is_ok());
///
/// // English example
/// let result = parse_vocabulary_entry("America:A,m,e,r,i,c,a");
/// assert!(result.is_ok());
///
/// // Example with punctuation
/// let result = parse_vocabulary_entry("メロスは激怒した。:め,ろ,す,は,げき,ど,し,た,。");
/// assert!(result.is_ok());
///
/// // Escaped colon
/// let result = parse_vocabulary_entry(r"a\:b:a,\:,b");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().view(), "a:b");
///
/// // Escaped comma in word
/// let result = parse_vocabulary_entry(r"カン,マ:か,ん,\,,ま");
/// //assert!(result.is_ok());
/// assert_eq!(result.unwrap().view(), "カン,マ");
///
/// // Escaped brackets
/// let result = parse_vocabulary_entry(r"\[テスト\]:[,て,す,と,]");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().view(), "[テスト]");
///
/// // Escaped backslash
/// let result = parse_vocabulary_entry(r"\\:\\");
/// assert!(result.is_ok());
/// assert_eq!(result.unwrap().view(), r"\");
///
/// // Invalid entries:
///
/// // Extra reading segment
/// let result = parse_vocabulary_entry("頑張る:が,ん,ば,る");
/// assert!(result.is_err());
///
/// // Reading count mismatch due to grouping
/// let result = parse_vocabulary_entry("[明日]の:あ,した,の");
/// assert!(result.is_err());
/// ```
pub fn parse_vocabulary_entry(line: &str) -> Result<VocabularyEntry, VocabularyParseError> {
    if line.lines().count() > 1 {
        return Err(VocabularyParseError::new(
            VocabularyParseErrorKind::MultipleLines,
        ));
    }

    let elements: Vec<String> = split_by_non_escaped_separator(line, ':');

    if elements.len() != 2 {
        return Err(VocabularyParseError::new(
            VocabularyParseErrorKind::ComponentsCountMisMatch,
        ));
    }

    let view = elements.first().unwrap();
    let spells_str = elements.get(1).unwrap();

    let (view, view_parts_counts) = remove_square_parentheses(view)?;
    let spells = split_by_non_escaped_separator(spells_str, ',');

    // Convert two consecutive backslashes in spells
    let spells: Vec<String> = spells
        .iter()
        .map(|spell| convert_two_backslash_to_single(spell))
        .collect();

    if spells.len() != view_parts_counts.len() {
        return Err(VocabularyParseError::new(
            VocabularyParseErrorKind::ViewAndSpellsCountMisMatch,
        ));
    }

    let spells: Vec<VocabularySpellElement> = construct_spell_strings(&spells)?
        .iter()
        .zip(view_parts_counts)
        .map(|(spell, count)| {
            if count == NonZeroUsize::new(1).unwrap() {
                VocabularySpellElement::Normal(spell.clone())
            } else {
                VocabularySpellElement::Compound((spell.clone(), count))
            }
        })
        .collect();

    let vocabulary_entry = VocabularyEntry::new(view.clone(), spells.clone()).ok_or(
        VocabularyParseError::new(VocabularyParseErrorKind::Internal(format!(
            "Failed to create VocabularyEntry for view: {}, spell: {:?}",
            view, spells
        ))),
    )?;

    Ok(vocabulary_entry)
}

// Convert a slice of strings to a list of spell strings
// Returns error if any of the strings are invalid spell strings
fn construct_spell_strings(strs: &[String]) -> Result<Vec<SpellString>, VocabularyParseError> {
    let mut spell_strings = vec![];
    for str in strs {
        if let Ok(spell_string) = SpellString::try_from(str.to_string()) {
            spell_strings.push(spell_string);
        } else {
            return Err(VocabularyParseError::new(
                VocabularyParseErrorKind::InvalidSpellString(str.clone()),
            ));
        }
    }

    Ok(spell_strings)
}

/// Separate passed line into multiple components with passed separator charactor.
/// Backslashed separators are recognized as separator charactor itself.
/// Backslash not followed after separator is retain.
///
/// Ex. When separator is colon ( : ),
/// a:b:c -> (a,b,c)
/// a<bslash>:b:c -> (a:b, c)
/// a<bslash><bslash>:b:c -> (a<bslash>:b, c)
fn split_by_non_escaped_separator(line: &str, separator: char) -> Vec<String> {
    assert_ne!(separator, '\\');

    let mut separated = Vec::<String>::new();
    let mut component = String::new();

    let mut is_prev_escape = false;

    for char in line.chars() {
        if char == separator {
            // Escaped separator is recognized as separator charactor itself
            if is_prev_escape {
                component.push(char);

                is_prev_escape = false;
            } else {
                separated.push(component.clone());
                component.clear();

                is_prev_escape = false;
            }
        } else if char == '\\' {
            if is_prev_escape {
                component.push(char);
                component.push(char);

                is_prev_escape = false;
            } else {
                is_prev_escape = true;
            }
        } else {
            if is_prev_escape {
                // Backslash not followed after separator is retain
                component.push('\\');
            }

            component.push(char);

            is_prev_escape = false;
        }
    }

    // Remained component should be added
    separated.push(component);

    separated
}

/// Removes square brackets ([]) and constructs a count of characters for each group of enclosed content.
/// Backslash-escaped square brackets and backslashes are treated as literal characters.
/// Returns Err if brackets are nested or don't match properly.
/// Other backslashes are left unchanged.
fn remove_square_parentheses(s: &str) -> Result<(String, Vec<NonZeroUsize>), VocabularyParseError> {
    // Construction is done in 2 stages:
    // 1. Remove square brackets while recording the position (start and end indices in the resulting string) of the enclosed parts
    // 2. Based on the positions of the enclosed parts, determine how many characters are in each compound unit
    let mut string = String::new();
    let mut surround_positions = VecDeque::<(usize, usize)>::new();

    let mut is_prev_escape = false;
    let mut i = 0;
    let mut compound_start_i: Option<usize> = None;

    // 1.
    for char in s.chars() {
        if char == '[' {
            if is_prev_escape {
                string.push(char);

                i += 1;
            } else {
                if compound_start_i.is_some() {
                    return Err(VocabularyParseError::new(
                        VocabularyParseErrorKind::CompoundSymbolMisMatch,
                    ));
                }
                compound_start_i.replace(i);
            }
            is_prev_escape = false;
        } else if char == ']' {
            if is_prev_escape {
                string.push(char);

                i += 1;
            } else {
                if let Some(compound_start_i) = compound_start_i {
                    // Empty compound are not allowed
                    if compound_start_i == i {
                        return Err(VocabularyParseError::new(
                            VocabularyParseErrorKind::EmptyCompound,
                        ));
                    }
                    surround_positions.push_back((compound_start_i, i - 1));
                } else {
                    return Err(VocabularyParseError::new(
                        VocabularyParseErrorKind::CompoundSymbolMisMatch,
                    ));
                }

                compound_start_i.take();
            }

            is_prev_escape = false;
        } else if char == '\\' {
            if is_prev_escape {
                string.push(char);
                i += 1;

                is_prev_escape = false;
            } else {
                is_prev_escape = true;
            }
        } else {
            if is_prev_escape {
                string.push('\\');
                i += 1;
            }

            string.push(char);
            i += 1;

            is_prev_escape = false;
        }
    }

    // Half-opened compound symbols are not allowed
    if compound_start_i.is_some() {
        return Err(VocabularyParseError::new(
            VocabularyParseErrorKind::CompoundSymbolMisMatch,
        ));
    }

    // 2.
    let mut character_counts: Vec<NonZeroUsize> = vec![];

    string.chars().enumerate().try_for_each(|(i, _)| {
        let front_position = surround_positions.front();

        if let Some((pos_start_i, pos_end_i)) = front_position {
            if pos_end_i < pos_start_i || i > *pos_end_i {
                return Err(VocabularyParseError::new(
                    VocabularyParseErrorKind::Internal(
                        "Compound symbol position index is corrupted".to_string(),
                    ),
                ));
            }

            if *pos_start_i <= i && i <= *pos_end_i {
                if i == *pos_end_i {
                    let character_count = NonZeroUsize::new(*pos_end_i - *pos_start_i + 1).ok_or(
                        VocabularyParseError::new(VocabularyParseErrorKind::Internal(
                            "charactor count in square parenthes is not NonZeroUsize".to_string(),
                        )),
                    )?;
                    character_counts.push(character_count);
                    surround_positions.pop_front();
                }
            } else {
                character_counts.push(NonZeroUsize::new(1).unwrap());
            }
        } else {
            character_counts.push(NonZeroUsize::new(1).unwrap());
        }

        Ok(())
    })?;

    Ok((string, character_counts))
}

/// Convert two consecutive backslashes into a single one
fn convert_two_backslash_to_single(s: &str) -> String {
    let mut string = String::new();

    let mut is_prev_escape = false;
    for char in s.chars() {
        if char == '\\' {
            if is_prev_escape {
                string.push(char);
                is_prev_escape = false;
            } else {
                is_prev_escape = true;
            }
        } else {
            if is_prev_escape {
                string.push('\\');
            }

            string.push(char);
            is_prev_escape = false;
        }
    }

    if is_prev_escape {
        string.push('\\');
    }

    string
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{VocabularyEntry, VocabularySpellElement};
    use std::num::NonZeroUsize;

    #[test]
    fn split_by_non_escaped_separator_split_empty_string_correctly() {
        let v = split_by_non_escaped_separator("", ':');
        assert_eq!(v, vec![""]);
    }

    #[test]
    fn split_by_non_escaped_separator_split_non_escaped_correctly() {
        let v = split_by_non_escaped_separator(r"hoge:fuga:jojo", ':');
        assert_eq!(
            v,
            vec![
                String::from("hoge"),
                String::from("fuga"),
                String::from("jojo")
            ]
        );
    }

    #[test]
    fn split_by_non_escaped_separator_escape_backslash_correctly() {
        let v = split_by_non_escaped_separator(r"hoge\::fuga", ':');
        assert_eq!(v, vec![String::from("hoge:"), String::from("fuga")]);
    }

    #[test]
    fn split_by_non_escaped_separator_remain_backslash_correctly() {
        let v = split_by_non_escaped_separator(r"h\o\\ge:fuga", ':');
        assert_eq!(v, vec![String::from(r"h\o\\ge"), String::from("fuga")]);
    }

    #[test]
    fn split_by_non_escaped_separator_split_empty_component_correctly() {
        let v = split_by_non_escaped_separator(r"::", ':');
        assert_eq!(
            v,
            vec![String::from(""), String::from(""), String::from("")]
        );
    }

    #[test]
    fn remove_square_parentheses_recognize_count_correctly() {
        assert_eq!(
            remove_square_parentheses(r"a[123]bc"),
            Ok((
                "a123bc".to_string(),
                vec![
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(3).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap()
                ]
            ))
        );
    }

    #[test]
    fn remove_square_parentheses_recognize_backslashed_backslash_correctly() {
        assert_eq!(
            remove_square_parentheses(r"a\\bc"),
            Ok((
                r"a\bc".to_string(),
                vec![
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap()
                ]
            ))
        );
    }

    #[test]
    fn remove_square_parentheses_recognize_backslashed_square_parentheses_correctly() {
        assert_eq!(
            remove_square_parentheses(r"a[\[123\]]b\[\]"),
            Ok((
                "a[123]b[]".to_string(),
                vec![
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(5).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap()
                ]
            ))
        );
    }

    #[test]
    fn remove_square_parentheses_remain_backslash_not_following_special_charactors_correctly() {
        assert_eq!(
            remove_square_parentheses(r"a\bc"),
            Ok((
                r"a\bc".to_string(),
                vec![
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap(),
                    NonZeroUsize::new(1).unwrap()
                ]
            ))
        );
    }

    #[test]
    fn remove_square_parentheses_returns_err_when_nesting() {
        assert_eq!(
            remove_square_parentheses(r"[[]]"),
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::CompoundSymbolMisMatch,
            ))
        );
    }

    #[test]
    fn remove_square_parentheses_returns_err_when_compound_is_closed_without_opened() {
        assert_eq!(
            remove_square_parentheses(r"a]bdf\["),
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::CompoundSymbolMisMatch,
            ))
        );
    }

    #[test]
    fn remove_square_parentheses_returns_err_when_compound_is_not_closed() {
        assert_eq!(
            remove_square_parentheses(r"a[bdf\["),
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::CompoundSymbolMisMatch,
            ))
        );
    }

    #[test]
    fn remove_square_parentheses_returns_err_when_compound_is_empty() {
        assert_eq!(
            remove_square_parentheses(r"[]"),
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::EmptyCompound,
            ))
        );
    }

    #[test]
    fn convert_two_backslash_to_single_convert_two_backslashes_correctly() {
        assert_eq!(convert_two_backslash_to_single(r"\\"), r"\");
    }

    #[test]
    fn convert_two_backslash_to_single_not_convert_third_backslash_with_following_charactor() {
        assert_eq!(convert_two_backslash_to_single(r"\\\a"), r"\\a");
    }

    #[test]
    fn convert_two_backslash_to_single_not_convert_third_backslash_without_following_charactor() {
        assert_eq!(convert_two_backslash_to_single(r"\\\"), r"\\");
    }

    #[test]
    fn convert_two_backslash_to_single_convert_four_backshasled_totwo_backslashes() {
        assert_eq!(convert_two_backslash_to_single(r"\\\\"), r"\\");
    }

    #[test]
    fn parse_vocabulary_entry_success_normal() {
        let result = parse_vocabulary_entry("頑張る:がん,ば,る");

        assert_eq!(
            result,
            Ok(VocabularyEntry::new(
                "頑張る".to_string(),
                vec![
                    VocabularySpellElement::Normal("がん".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal("ば".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal("る".to_string().try_into().unwrap())
                ]
            )
            .unwrap())
        );
    }

    #[test]
    fn parse_vocabulary_entry_success_with_compound() {
        let result =
            parse_vocabulary_entry("[昨日]の敵は[今日]の友:きのう,の,てき,は,きょう,の,とも");

        assert_eq!(
            result,
            Ok(VocabularyEntry::new(
                "昨日の敵は今日の友".to_string(),
                vec![
                    VocabularySpellElement::Compound((
                        "きのう".to_string().try_into().unwrap(),
                        NonZeroUsize::new(2).unwrap()
                    )),
                    VocabularySpellElement::Normal("の".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal("てき".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal("は".to_string().try_into().unwrap()),
                    VocabularySpellElement::Compound((
                        "きょう".to_string().try_into().unwrap(),
                        NonZeroUsize::new(2).unwrap()
                    )),
                    VocabularySpellElement::Normal("の".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal("とも".to_string().try_into().unwrap()),
                ]
            )
            .unwrap())
        );
    }

    #[test]
    fn parse_vocabulary_entry_success_with_escaped_characters() {
        let result = parse_vocabulary_entry(r"\\\::\\,\:");

        assert_eq!(
            result,
            Ok(VocabularyEntry::new(
                r"\:".to_string(),
                vec![
                    VocabularySpellElement::Normal(r"\".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal(":".to_string().try_into().unwrap()),
                ]
            )
            .unwrap())
        );
    }

    #[test]
    fn parse_vocabulary_entry_success_with_escaped_brackets() {
        let result = parse_vocabulary_entry(r"[\[]12:[,1,2");

        assert_eq!(
            result,
            Ok(VocabularyEntry::new(
                "[12".to_string(),
                vec![
                    VocabularySpellElement::Normal(r"[".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal("1".to_string().try_into().unwrap()),
                    VocabularySpellElement::Normal("2".to_string().try_into().unwrap()),
                ]
            )
            .unwrap())
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_multiple_lines() {
        let result = parse_vocabulary_entry("頑張る:がん,ば,る\n頑張る:がんば,る");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::MultipleLines
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_components_count_mismatch_no_colon() {
        let result = parse_vocabulary_entry("頑張る");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::ComponentsCountMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_components_count_mismatch_too_many_colons() {
        let result = parse_vocabulary_entry("頑:張:る:がん,ば,る");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::ComponentsCountMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_compound_symbol_mismatch_unclosed() {
        let result = parse_vocabulary_entry("[頑張る:がん,ば,る");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::CompoundSymbolMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_compound_symbol_mismatch_unopened() {
        let result = parse_vocabulary_entry("頑張る]:がん,ば,る");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::CompoundSymbolMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_compound_symbol_mismatch_nested() {
        let result = parse_vocabulary_entry("[[頑張る]]:がん,ば,る");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::CompoundSymbolMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_empty_compound() {
        let result = parse_vocabulary_entry("頑張[]る:がん,ば,る");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::EmptyCompound
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_view_and_spells_count_mismatch_too_few_spells() {
        let result = parse_vocabulary_entry("頑張る:がん,ば");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::ViewAndSpellsCountMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_view_and_spells_count_mismatch_too_many_spells() {
        let result = parse_vocabulary_entry("頑張る:がん,ば,る,よ");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::ViewAndSpellsCountMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_view_and_spells_count_mismatch_with_compound() {
        let result = parse_vocabulary_entry("[今日]の:きょ,う,の");

        assert_eq!(
            result,
            Err(VocabularyParseError::new(
                VocabularyParseErrorKind::ViewAndSpellsCountMisMatch
            ))
        );
    }

    #[test]
    fn parse_vocabulary_entry_error_invalid_spell_string() {
        let result = parse_vocabulary_entry("頑張る:がん,ば,る三");

        match result {
            Err(VocabularyParseError {
                kind: VocabularyParseErrorKind::InvalidSpellString(_),
            }) => assert!(true),
            _ => assert!(false, "Expected InvalidSpellString error, got {:?}", result),
        }
    }
}
