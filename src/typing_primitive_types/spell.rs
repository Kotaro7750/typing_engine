use crate::utility::{is_displayable_ascii, is_hiragana, is_japanese_symbol};
use std::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// A string only contains characters which can be used as spells.
///
/// Characters can be used as spells are
/// * A displayable ASCII. (`U+20` ~ `U+7E`)
/// * A japanese hiragana. (`U+3041` ~ `U+308F`, `U+3092` ~ `U+3094`)
/// * A japanese symbol.
pub struct SpellString(String);

impl SpellString {
    /// Returns whether SpellString contains displayable ASCII characters.
    pub(crate) fn contains_displayable_ascii(&self) -> bool {
        for c in self.chars() {
            if is_displayable_ascii(c) {
                return true;
            }
        }

        false
    }
}

/// Checks whether a character can be used in a SpellString.
fn usable_in_spell_string(c: char) -> bool {
    is_displayable_ascii(c) || is_hiragana(c) || is_japanese_symbol(c)
}

impl From<SpellString> for String {
    fn from(ss: SpellString) -> Self {
        ss.0
    }
}

impl Deref for SpellString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SpellString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug)]
/// An error type for SpellString related operations.
pub struct SpellStringError {
    char: char,
}

impl SpellStringError {
    fn new(char: char) -> Self {
        Self { char }
    }
}

impl Display for SpellStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` cannot be used as a spell", self.char)
    }
}

impl Error for SpellStringError {}

impl TryFrom<String> for SpellString {
    type Error = SpellStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        for c in value.chars() {
            if !usable_in_spell_string(c) {
                return Err(SpellStringError::new(c));
            }
        }

        Ok(Self(value))
    }
}
