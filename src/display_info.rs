use serde::{Deserialize, Serialize};

use crate::statistics::lap_statistics::PrimitiveLapStatisticsBuilder;
use crate::statistics::statistics_counter::PrimitiveStatisticsCounter;
use crate::statistics::{
    construct_on_typing_statistics_target, KeyStrokeDisplayStringBuilder, OnTypingStatisticsTarget,
    SpellDisplayStringBuilder,
};
use crate::typing_primitive_types::vocabulary::{
    corresponding_view_positions_for_spell, ViewPosition,
};

/// A type for composing typing game UI.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DisplayInfo {
    view: ViewDisplayInfo,
    spell: SpellDisplayInfo,
    key_stroke: KeyStrokeDisplayInfo,
}

impl DisplayInfo {
    pub(crate) fn new(
        view: ViewDisplayInfo,
        spell: SpellDisplayInfo,
        key_stroke: KeyStrokeDisplayInfo,
    ) -> Self {
        Self {
            view,
            spell,
            key_stroke,
        }
    }
    /// Get an information about query string itself.
    pub fn view_info(&self) -> &ViewDisplayInfo {
        &self.view
    }

    /// Get an information about spell of query string.
    pub fn spell_info(&self) -> &SpellDisplayInfo {
        &self.spell
    }

    /// Get an information about key strokes of query string.
    pub fn key_stroke_info(&self) -> &KeyStrokeDisplayInfo {
        &self.key_stroke
    }
}

/// Information about query string itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewDisplayInfo {
    view: String,
    current_cursor_positions: Vec<usize>,
    missed_positions: Vec<usize>,
    last_position: usize,
}

impl ViewDisplayInfo {
    pub(crate) fn new(
        spell_display_info: &SpellDisplayInfo,
        view: String,
        view_position_of_spell: Vec<ViewPosition>,
    ) -> Self {
        Self {
            view,
            current_cursor_positions: corresponding_view_positions_for_spell(
                &spell_display_info.current_cursor_positions,
                &view_position_of_spell,
            ),
            missed_positions: corresponding_view_positions_for_spell(
                &spell_display_info.missed_positions,
                &view_position_of_spell,
            ),
            last_position: view_position_of_spell[spell_display_info.last_position].last_position(),
        }
    }

    /// Query string itself.
    pub fn view(&self) -> &str {
        &self.view
    }

    /// Index of character currently typed.
    ///
    /// Index may be multiple because some key strokes can type multiple characters.
    ///
    /// ex. When start typing `シュート`, key strokes `syu` can type `シュ`, so this function returns
    /// `[0,1]`.
    pub fn current_cursor_positions(&self) -> &Vec<usize> {
        &self.current_cursor_positions
    }

    /// Index of character which is not correctly typed.
    ///
    /// ex. When query string is `シュート`, and given key stroke was `s` -> `a(miss type)` -> `y` -> `u` ->
    /// `-`, this function returns `[0,1]`.
    pub fn missed_positions(&self) -> &Vec<usize> {
        &self.missed_positions
    }

    /// Index of last view string to be typed.
    ///
    /// This function is useful to distinct view string character to be typed and not to be typed.
    pub fn last_position(&self) -> usize {
        self.last_position
    }
}

/// Information about spell of query string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SpellDisplayInfo {
    spell: String,
    // 現在のカーソル位置
    // 複数文字をまとめて入力する場合もあるため複数持てるようにしている
    current_cursor_positions: Vec<usize>,
    missed_positions: Vec<usize>,
    // タイプすべき最後のチャンクの綴りの末尾の位置
    // クエリをタイプ数で指定する場合には語彙の途中のチャンクで切れている可能性がある
    last_position: usize,
    on_typing_statistics: OnTypingStatisticsTarget,
}

impl SpellDisplayInfo {
    pub(crate) fn new(
        spell: String,
        current_cursor_positions: Vec<usize>,
        missed_positions: Vec<usize>,
        last_position: usize,
        on_typing_statistics: OnTypingStatisticsTarget,
    ) -> Self {
        Self {
            spell,
            current_cursor_positions,
            missed_positions,
            last_position,
            on_typing_statistics,
        }
    }

    pub(crate) fn new_with(
        spell_display_string_builder: &SpellDisplayStringBuilder,
        spell_statistics_counter: &PrimitiveStatisticsCounter,
        spell_lap_statistics_builder: &PrimitiveLapStatisticsBuilder,
    ) -> Self {
        Self::new(
            spell_display_string_builder.spell().to_string(),
            spell_display_string_builder
                .cursor_position()
                .construct_vec(),
            spell_display_string_builder.wrong_positions().to_vec(),
            spell_display_string_builder.last_position(),
            construct_on_typing_statistics_target(
                spell_statistics_counter,
                spell_lap_statistics_builder,
            ),
        )
    }

    /// Spell of query string.
    ///
    /// ex. When query string is `巨大`, this function returns `きょだい`.
    pub fn spell(&self) -> &str {
        &self.spell
    }

    /// Index of spell currently typed.
    ///
    /// Index may be multiple because some key strokes can type multiple spell.
    ///
    /// ex. When start typing `巨大` ( spell is `きょだい` ), key strokes `kyo` can type `巨` ( spell is `きょ` ), so this function returns
    /// `[0,1]`.
    pub fn current_cursor_positions(&self) -> &Vec<usize> {
        &self.current_cursor_positions
    }

    /// Index of spell which is not correctly typed.
    ///
    /// ex. When query string is `巨大` ( spell is `きょだい` ), and given key stroke was `k` -> `a(miss type)` -> `y` -> `o` ->
    /// `d`, this function returns `[0,1]`.
    pub fn missed_positions(&self) -> &Vec<usize> {
        &self.missed_positions
    }

    /// Index of last spell to be typed.
    ///
    /// This function is useful to distinct spell to be typed and not to be typed.
    pub fn last_position(&self) -> usize {
        self.last_position
    }
}

/// Information about key stroke of query string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyStrokeDisplayInfo {
    key_stroke: String,
    current_cursor_position: usize,
    missed_positions: Vec<usize>,
    on_typing_statistics: OnTypingStatisticsTarget,
    on_typing_statistics_ideal: OnTypingStatisticsTarget,
}

impl KeyStrokeDisplayInfo {
    pub(crate) fn new(
        key_stroke: String,
        current_cursor_position: usize,
        missed_positions: Vec<usize>,
        on_typing_statistics: OnTypingStatisticsTarget,
        on_typing_statistics_ideal: OnTypingStatisticsTarget,
    ) -> Self {
        Self {
            key_stroke,
            current_cursor_position,
            missed_positions,
            on_typing_statistics,
            on_typing_statistics_ideal,
        }
    }

    pub(crate) fn new_with(
        key_stroke_display_string_builder: &KeyStrokeDisplayStringBuilder,
        key_stroke_statistics_counter: &PrimitiveStatisticsCounter,
        key_stroke_lap_statistics_builder: &PrimitiveLapStatisticsBuilder,
        ideal_key_stroke_statistics_counter: &PrimitiveStatisticsCounter,
        ideal_key_stroke_lap_statistics_builder: &PrimitiveLapStatisticsBuilder,
    ) -> Self {
        Self::new(
            key_stroke_display_string_builder.key_stroke().to_string(),
            key_stroke_display_string_builder.cursor_position(),
            key_stroke_display_string_builder.wrong_positions().to_vec(),
            construct_on_typing_statistics_target(
                key_stroke_statistics_counter,
                key_stroke_lap_statistics_builder,
            ),
            construct_on_typing_statistics_target(
                ideal_key_stroke_statistics_counter,
                ideal_key_stroke_lap_statistics_builder,
            ),
        )
    }

    /// Information about key strokes of query string.
    pub fn key_stroke(&self) -> &str {
        &self.key_stroke
    }

    /// Index of key stroke currently typed.
    pub fn current_cursor_position(&self) -> usize {
        self.current_cursor_position
    }

    /// Index of key stroke which is not correctly typed.
    pub fn missed_positions(&self) -> &Vec<usize> {
        &self.missed_positions
    }

    pub fn on_typing_statistics(&self) -> &OnTypingStatisticsTarget {
        &self.on_typing_statistics
    }

    pub fn on_typing_statistics_ideal(&self) -> &OnTypingStatisticsTarget {
        &self.on_typing_statistics_ideal
    }
}
