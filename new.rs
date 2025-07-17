use crate::ast::*;
use crate::token::{Keyword, Token};
use std::collections::HashMap;

// 括弧系の種類を定義
#[derive(Debug, Clone, PartialEq)]
enum BracketType {
    Paren,   // ()
    Square,  // []
}

// 括弧系の情報を保持
#[derive(Debug, Clone)]
struct BracketGroup {
    bracket_type: BracketType,
    content: Vec<Token>,
    position: usize, // 元のトークン列での位置
}

// 改良された宣言子パーサ
impl ParseSession {
    // メイン宣言子パーサ
    pub fn parse_declarator(&mut self) -> Declarator {
        let mut pointer_level = 0;
        
        // ポインタのレベルを数える
        while consume(Token::Asterisk, &mut self.tokens) {
            pointer_level += 1;
        }
        
        // 括弧系を抽出
        let bracket_groups = self.extract_bracket_groups();
        
        // 識別子を見つける
        let ident = self.find_identifier(&bracket_groups);
        
        // 宣言子を構築
        let direct_declarator = self.build_declarator_from_brackets(ident, bracket_groups);
        
        if pointer_level == 0 {
            Declarator::Direct(direct_declarator)
        } else {
            Declarator::Pointer(Pointer {
                level: pointer_level,
                inner: Box::new(Some(direct_declarator)),
            })
        }
    }
    
    // 括弧系を抽出する
    fn extract_bracket_groups(&mut self) -> Vec<BracketGroup> {
        let mut groups = Vec::new();
        let mut position = 0;
        
        while position < self.tokens.len() {
            match &self.tokens[position] {
                Token::LParen => {
                    let content = self.extract_paren_content(position);
                    groups.push(BracketGroup {
                        bracket_type: BracketType::Paren,
                        content,
                        position,
                    });
                    // 対応する右括弧まで進む
                    position = self.find_matching_paren(position) + 1;
                }
                Token::LBracket => {
                    let content = self.extract_bracket_content(position);
                    groups.push(BracketGroup {
                        bracket_type: BracketType::Square,
                        content,
                        position,
                    });
                    // 対応する右括弧まで進む
                    position = self.find_matching_bracket(position) + 1;
                }
                _ => position += 1,
            }
        }
        
        groups
    }
    
    // 括弧の内容を抽出
    fn extract_paren_content(&self, start: usize) -> Vec<Token> {
        let mut content = Vec::new();
        let mut depth = 0;
        
        for i in (start + 1)..self.tokens.len() {
            match &self.tokens[i] {
                Token::LParen => depth += 1,
                Token::RParen => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
            content.push(self.tokens[i].clone());
        }
        
        content
    }
    
    // 角括弧の内容を抽出
    fn extract_bracket_content(&self, start: usize) -> Vec<Token> {
        let mut content = Vec::new();
        let mut depth = 0;
        
        for i in (start + 1)..self.tokens.len() {
            match &self.tokens[i] {
                Token::LBracket => depth += 1,
                Token::RBracket => {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                }
                _ => {}
            }
            content.push(self.tokens[i].clone());
        }
        
        content
    }
    
    // 対応する右括弧を見つける
    fn find_matching_paren(&self, start: usize) -> usize {
        let mut depth = 0;
        
        for i in start..self.tokens.len() {
            match &self.tokens[i] {
                Token::LParen => depth += 1,
                Token::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        
        panic!("Unmatched parenthesis");
    }
    
    // 対応する右角括弧を見つける
    fn find_matching_bracket(&self, start: usize) -> usize {
        let mut depth = 0;
        
        for i in start..self.tokens.len() {
            match &self.tokens[i] {
                Token::LBracket => depth += 1,
                Token::RBracket => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        
        panic!("Unmatched bracket");
    }
    
    // 識別子を見つける
    fn find_identifier(&self, groups: &[BracketGroup]) -> Option<Ident> {
        // 括弧系の外にある識別子を探す
        let mut covered_positions = Vec::new();
        
        for group in groups {
            let end_pos = match group.bracket_type {
                BracketType::Paren => self.find_matching_paren(group.position),
                BracketType::Square => self.find_matching_bracket(group.position),
            };
            
            for pos in group.position..=end_pos {
                covered_positions.push(pos);
            }
        }
        
        // 括弧系に覆われていない識別子を探す
        for (i, token) in self.tokens.iter().enumerate() {
            if let Token::Ident(name) = token {
                if !covered_positions.contains(&i) {
                    return Some(Ident::new(name.clone()));
                }
            }
        }
        
        None
    }
    
    // 括弧系から宣言子を構築
    fn build_declarator_from_brackets(&self, ident: Option<Ident>, groups: Vec<BracketGroup>) -> DirectDeclarator {
        // 識別子がある場合はそれをベースにする
        let mut base = if let Some(ident) = ident {
            DirectDeclarator::Ident(ident)
        } else {
            // 識別子がない場合は最初の括弧系を調べる
            if groups.is_empty() {
                panic!("No identifier found in declarator");
            }
            
            // 最初の括弧系が () で中身が宣言子なら、それを処理
            if groups[0].bracket_type == BracketType::Paren {
                return self.parse_paren_declarator(&groups[0]);
            }
            
            panic!("Invalid declarator structure");
        };
        
        // 括弧系を位置順にソートして処理
        let mut sorted_groups = groups;
        sorted_groups.sort_by_key(|g| g.position);
        
        // 各括弧系を処理
        for group in sorted_groups {
            match group.bracket_type {
                BracketType::Paren => {
                    // 関数として処理
                    let params = self.parse_param_list_from_tokens(&group.content);
                    base = DirectDeclarator::Func(Func {
                        base: Box::new(base),
                        params,
                    });
                }
                BracketType::Square => {
                    // 配列として処理
                    let size = self.parse_array_size_from_tokens(&group.content);
                    base = DirectDeclarator::Array(Array {
                        base: Box::new(base),
                        size,
                    });
                }
            }
        }
        
        base
    }
    
    // 括弧内の宣言子を処理
    fn parse_paren_declarator(&self, group: &BracketGroup) -> DirectDeclarator {
        // 括弧内が別の宣言子の場合
        let mut temp_session = self.clone();
        temp_session.tokens = group.content.clone();
        
        let inner_declarator = temp_session.parse_declarator();
        DirectDeclarator::Paren(Box::new(inner_declarator))
    }
    
    // パラメータリストを解析
    fn parse_param_list_from_tokens(&self, tokens: &[Token]) -> Option<ParamList> {
        if tokens.is_empty() {
            return None;
        }
        
        // void の場合
        if tokens.len() == 1 && tokens[0] == Token::void() {
            return Some(ParamList::Void);
        }
        
        // 実際のパラメータリスト解析
        let mut temp_session = self.clone();
        temp_session.tokens = tokens.to_vec();
        
        if temp_session.tokens.is_empty() {
            return None;
        }
        
        Some(temp_session.parse_param_list())
    }
    
    // 配列サイズを解析
    fn parse_array_size_from_tokens(&self, tokens: &[Token]) -> Option<Expr> {
        if tokens.is_empty() {
            return None;
        }
        
        let mut temp_session = self.clone();
        temp_session.tokens = tokens.to_vec();
        
        Some(temp_session.parse_expr())
    }
    
    // パラメータリストを解析（実装）
    fn parse_param_list(&mut self) -> ParamList {
        if consume(Token::void(), &mut self.tokens) {
            ParamList::Void
        } else {
            let mut params = vec![];
            
            if !self.tokens.is_empty() {
                params.push(self.parse_param());
                
                while consume(Token::Comma, &mut self.tokens) {
                    params.push(self.parse_param());
                }
            }
            
            ParamList::Params(params)
        }
    }
    
    // 単一パラメータを解析
    fn parse_param(&mut self) -> Param {
        let ty = consume_type(&mut self.tokens);
        
        let declarator = if self.has_declarator() {
            Some(self.parse_declarator())
        } else {
            None
        };
        
        Param::new(ty, declarator)
    }
    
    // 宣言子があるかチェック
    fn has_declarator(&self) -> bool {
        !self.tokens.is_empty() && 
        (self.tokens[0] == Token::Asterisk || 
         matches!(self.tokens[0], Token::Ident(_)) ||
         self.tokens[0] == Token::LParen ||
         self.tokens[0] == Token::LBracket)
    }
    
    // 式を解析（簡単な実装）
    fn parse_expr(&mut self) -> Expr {
        // 実際の式解析は既存のexpr()関数を使用
        expr(self)
    }
}

// テスト用のヘルパー関数
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_array_of_pointers() {
        // int *x[3]; のテスト
        let tokens = vec![
            Token::int(),
            Token::Asterisk,
            Token::Ident("x".to_string()),
            Token::LBracket,
            Token::NumInt(3),
            Token::RBracket,
        ];
        
        let mut session = ParseSession::new(tokens);
        let result = session.parse_declarator();
        
        // 期待される構造: Array(Pointer(Ident("x")), Some(3))
        println!("Result: {:?}", result);
    }
    
    #[test]
    fn test_pointer_to_array() {
        // int (*x)[3]; のテスト
        let tokens = vec![
            Token::int(),
            Token::LParen,
            Token::Asterisk,
            Token::Ident("x".to_string()),
            Token::RParen,
            Token::LBracket,
            Token::NumInt(3),
            Token::RBracket,
        ];
        
        let mut session = ParseSession::new(tokens);
        let result = session.parse_declarator();
        
        // 期待される構造: Pointer(Array(Ident("x"), Some(3)))
        println!("Result: {:?}", result);
    }
    
    #[test]
    fn test_function_pointer() {
        // void (*signal(int, void (*)(int)))(int); のテスト
        let tokens = vec![
            Token::void(),
            Token::LParen,
            Token::Asterisk,
            Token::Ident("signal".to_string()),
            Token::LParen,
            Token::int(),
            Token::Comma,
            Token::void(),
            Token::LParen,
            Token::Asterisk,
            Token::RParen,
            Token::LParen,
            Token::int(),
            Token::RParen,
            Token::RParen,
            Token::RParen,
            Token::LParen,
            Token::int(),
            Token::RParen,
        ];
        
        let mut session = ParseSession::new(tokens);
        let result = session.parse_declarator();
        
        println!("Result: {:?}", result);
    }
}

// 既存の関数を維持しつつ、新しいパーサを統合
fn declarator(parse_session: &mut ParseSession) -> Declarator {
    parse_session.parse_declarator()
}

// consume関数と他のヘルパー関数は既存のものを使用
fn consume(op: Token, tokens: &mut Vec<Token>) -> bool {
    if tokens.is_empty() {
        return false;
    }

    if *tokens.first().unwrap() != op {
        return false;
    }

    tokens.remove(0);
    return true;
}

fn consume_type(tokens: &mut Vec<Token>) -> Type {
    if tokens.is_empty() {
        panic!("Expected type, but no tokens available");
    }

    if consume(Token::int(), tokens) {
        return Type::Int;
    } else if consume(Token::double(), tokens) {
        return Type::Double;
    } else if consume(Token::char(), tokens) {
        return Type::Char;
    } else if consume(Token::void(), tokens) {
        return Type::Void;
    } else if consume(Token::r#struct(), tokens) {
        return Type::Struct(consume_ident(tokens));
    } else if consume(Token::r#union(), tokens) {
        return Type::Union(consume_ident(tokens));
    } else if consume(Token::r#enum(), tokens) {
        return Type::Enum(consume_ident(tokens));
    } else if is_next_ident(tokens) {
        return Type::Typedef(consume_ident(tokens));
    } else {
        panic!("Expected type, found {:?}", tokens.first());
    }
}

fn consume_ident(tokens: &mut Vec<Token>) -> Ident {
    let ident = get_ident(tokens);
    tokens.remove(0);
    ident
}

fn get_ident(tokens: &[Token]) -> Ident {
    if let Some(Token::Ident(name)) = tokens.first() {
        let name = name.clone();
        Ident::new(name)
    } else {
        panic!("Expected identifier, found {:?}", tokens);
    }
}

fn is_next_ident(tokens: &[Token]) -> bool {
    if tokens.is_empty() {
        return false;
    }
    let next = tokens.first().unwrap();
    return matches!(next, Token::Ident(_));
}

fn expr(parse_session: &mut ParseSession) -> Expr {
    // 既存のexpr実装を使用
    // この部分は既存のコードから持ってくる
    unimplemented!("Use existing expr implementation")
}