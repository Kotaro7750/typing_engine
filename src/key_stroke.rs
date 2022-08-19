use std::error::Error;
use std::fmt::Display;
use std::ops::Deref;
use std::time::Duration;

use crate::utility::is_displayable_ascii;

// キーストロークとして使用可能な文字
// スペースを含む表示可能なASCII
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
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
pub struct KeyStrokeString(String);

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
pub struct KeyStrokeStringError {
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
}
