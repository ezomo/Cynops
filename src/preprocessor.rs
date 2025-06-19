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
