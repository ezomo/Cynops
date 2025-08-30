pub fn unescape_char_literals(input: &mut String) {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\'' {
            // 文字リテラルの開始
            result.push(ch);

            // 文字リテラルの中身を処理
            while let Some(inner_ch) = chars.next() {
                if inner_ch == '\\' {
                    // エスケープシーケンス
                    if let Some(escaped_ch) = chars.next() {
                        match escaped_ch {
                            'n' => result.push('\n'),  // '\n' → 改行文字
                            't' => result.push('\t'),  // '\t' → タブ文字
                            'r' => result.push('\r'),  // '\r' → キャリッジリターン
                            '0' => result.push('\0'),  // '\0' → NULL文字
                            '\\' => result.push('\\'), // '\\' → バックスラッシュ
                            '\'' => result.push('\''), // '\'' → シングルクォート
                            '"' => result.push('"'),   // '\"' → ダブルクォート
                            _ => {
                                // その他のエスケープシーケンスはそのまま
                                result.push('\\');
                                result.push(escaped_ch);
                            }
                        }
                    }
                } else if inner_ch == '\'' {
                    // 文字リテラルの終了
                    result.push(inner_ch);
                    break;
                } else {
                    // 通常の文字
                    result.push(inner_ch);
                }
            }
        } else {
            // 文字リテラル外はそのまま
            result.push(ch);
        }
    }

    *input = result;
}

pub fn remove_comments(src: &mut String) {
    let mut result = String::new();
    let mut chars = src.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // 行コメントをスキップ
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                    continue;
                }
                Some('*') => {
                    // ブロックコメントをスキップ
                    chars.next();
                    while let Some(nc) = chars.next() {
                        if nc == '*' {
                            if let Some('/') = chars.peek() {
                                chars.next();
                                break;
                            }
                        }
                    }
                    continue;
                }
                Some(&next) => {
                    result.push('/');
                    result.push(next);
                    chars.next();
                    continue;
                }
                None => {
                    result.push('/');
                    continue;
                }
            }
        } else {
            result.push(c);
        }
    }
    *src = result;
}
