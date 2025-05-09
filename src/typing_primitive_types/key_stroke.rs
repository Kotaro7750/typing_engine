use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::Display;
use std::ops::Deref;
use std::time::Duration;

use crate::utility::is_displayable_ascii;

use super::chunk::inflight::KeyStrokeResult;

/// A type representing a character can be used as a key stroke.
#[derive(Debug, PartialEq, Eq, Clone, Hash, Ord, PartialOrd, Serialize, Deserialize)]
pub struct KeyStrokeChar(char);

impl From<KeyStrokeChar> for char {
    fn from(c: KeyStrokeChar) -> Self {
        c.0
    }
}

impl PartialEq<char> for KeyStrokeChar {
    fn eq(&self, other: &char) -> bool {
        self.0 == *other
    }
}

#[derive(Debug)]
pub struct KeyStrokeCharError;

impl Display for KeyStrokeCharError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cannot use as a key stroke")
    }
}

impl Error for KeyStrokeCharError {}

impl TryFrom<char> for KeyStrokeChar {
    type Error = KeyStrokeCharError;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        if is_displayable_ascii(value) {
            Ok(Self(value))
        } else {
            Err(KeyStrokeCharError)
        }
    }
}

// KeyStrokeCharで構成された文字列
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct KeyStrokeString(String);

impl KeyStrokeString {
    /// Returns key stroke chars of this key stroke string.
    pub(crate) fn key_stroke_chars(&self) -> Vec<KeyStrokeChar> {
        self.chars().map(|c| c.try_into().unwrap()).collect()
    }
}

impl From<KeyStrokeString> for String {
    fn from(s: KeyStrokeString) -> Self {
        s.0
    }
}

impl Deref for KeyStrokeString {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub(crate) struct KeyStrokeStringError {
    char: char,
}

impl KeyStrokeStringError {
    fn new(char: char) -> Self {
        Self { char }
    }
}

impl Display for KeyStrokeStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` cannot be used as a key stroke", self.char)
    }
}

impl Error for KeyStrokeStringError {}

impl TryFrom<String> for KeyStrokeString {
    type Error = KeyStrokeStringError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        for c in value.chars() {
            if !is_displayable_ascii(c) {
                return Err(KeyStrokeStringError::new(c));
            }
        }

        Ok(Self(value))
    }
}

// タイピング中のそれぞれのキーストローク
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub(crate) struct ActualKeyStroke {
    // タイピングを開始した時点からこのキーストロークが起こった時点までにかかった時間
    elapsed_time: Duration,
    key_stroke: KeyStrokeChar,
    is_correct: bool,
}

impl ActualKeyStroke {
    pub(crate) fn new(elapsed_time: Duration, key_stroke: KeyStrokeChar, is_correct: bool) -> Self {
        Self {
            elapsed_time,
            key_stroke,
            is_correct,
        }
    }

    pub(crate) fn elapsed_time(&self) -> &Duration {
        &self.elapsed_time
    }

    pub(crate) fn key_stroke(&self) -> &KeyStrokeChar {
        &self.key_stroke
    }

    pub(crate) fn is_correct(&self) -> bool {
        self.is_correct
    }
}

/// An enum representing the result of a key stroke.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum KeyStrokeHitMiss {
    // キーストロークが正しかったケース
    Correct,
    // キーストロークが間違っていたケース
    Wrong,
}

impl From<KeyStrokeResult> for KeyStrokeHitMiss {
    fn from(result: KeyStrokeResult) -> Self {
        if result.is_correct() {
            Self::Correct
        } else {
            Self::Wrong
        }
    }
}
