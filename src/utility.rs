// 表示可能なASCIIかどうか
// U+0020~U+007E
pub fn is_displayable_ascii(c: char) -> bool {
    c.is_ascii() && !c.is_ascii_control()
}

pub fn is_hiragana(c: char) -> bool {
    matches!(c,'\u{3041}'..='\u{308f}' | '\u{3092}'..='\u{3094}')
}

pub fn is_japanese_symbol(c: char) -> bool {
    matches!(c,
        // 全角ダブルクオーテーション・全角シングルクオーテーション
        '\u{2019}' | '\u{201d}' |
        // 全角スペース・読点・句点
        '\u{3000}'..='\u{3002}' |
        // 鉤括弧
        '\u{300c}'..='\u{300d}' |
        // 全角チルダ
        '\u{301c}' |
        // 中黒・全角バー
        '\u{30fb}'..='\u{30fc}' |
        // 全角エクスクラメーションマーク
        '\u{ff01}' |
        // 全角シャープ・全角ドルマーク・全角パーセント・全角アンパサンド
        '\u{ff03}'..='\u{ff06}' |
        // 全角丸括弧・全角アスタリスク・全角プラス
        '\u{ff08}'..='\u{ff0b}' |
        // 全角スラッシュ
        '\u{ff0f}' |
        // 全角コロン・全角セミコロン・全角山括弧・全角イコール・全角クエスチョンマーク・全角アットマーク
        '\u{ff1a}'..='\u{ff20}' |
        // 全角ハット・全角アンダーバー・全角バッククオート
        '\u{ff3e}'..='\u{ff40}' |
        // 全角波括弧・全角バーティカルバー
        '\u{ff5b}'..='\u{ff5d}' |
        // 全角円マーク
        '\u{ffe5}'
    )
}

/// Return ceil(a/b)
pub(crate) fn calc_ceil_div(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

/// 対象間の数の違いを考慮して位置の変換をする
pub(crate) fn convert_by_weighted_count(
    from_count: usize,
    to_count: usize,
    from_delta: usize,
) -> usize {
    calc_ceil_div(from_delta * to_count, from_count)
}
