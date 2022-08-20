use crate::key_stroke::KeyStrokeString;

// 綴り文字列を表示するための情報
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SpellDisplayInfo {
    spell: String,
    // 現在のカーソル位置
    // 複数文字をまとめて入力する場合もあるため複数持てるようにしている
    current_cursor_positions: Vec<usize>,
    missed_positions: Vec<usize>,
    // タイプすべき最後のチャンクの綴りの末尾の位置
    // クエリをタイプ数で指定する場合には語彙の途中のチャンクで切れている可能性がある
    last_position: usize,
}

impl SpellDisplayInfo {
    pub(crate) fn new(
        spell: String,
        current_cursor_positions: Vec<usize>,
        missed_positions: Vec<usize>,
        last_position: usize,
    ) -> Self {
        Self {
            spell,
            current_cursor_positions,
            missed_positions,
            last_position,
        }
    }
}

// キーストローク文字列を表示するための情報
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct KeyStrokeDisplayInfo {
    key_stroke: String,
    current_cursor_position: usize,
    missed_positions: Vec<usize>,
}

impl KeyStrokeDisplayInfo {
    pub(crate) fn new(
        key_stroke: String,
        current_cursor_position: usize,
        missed_positions: Vec<usize>,
    ) -> Self {
        Self {
            key_stroke,
            current_cursor_position,
            missed_positions,
        }
    }
}
