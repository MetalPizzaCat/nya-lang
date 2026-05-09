#[derive(Debug, Clone, Copy)]
pub enum Keyword {
    Function,
    Let,
    If,
    Else,
    Elif,
}

impl Keyword {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "let" => Some(Self::Let),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub enum Operator {
    Plus,
    Minus,
    Assign,
}

impl Operator {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "=" => Some(Self::Assign),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OperatorTokenData {
    operator: Operator,
    unary: bool,
}

impl OperatorTokenData {
    pub fn get_priority(&self) -> i32 {
        match &self.operator {
            Operator::Plus => 6,
            Operator::Minus => 6,
            Operator::Assign => 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TokenKind<'a> {
    Keyword(Keyword),
    Operator(OperatorTokenData),
    Identifier(&'a str),
    ConstantString(String),
    Integer(i64),
    Boolean(bool),
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    token_type: TokenKind<'a>,
    row: usize,
    column: usize,
}

impl<'a> Token<'a> {
    pub fn new(token_type: TokenKind<'a>, row: usize, column: usize) -> Self {
        Self {
            token_type,
            row,
            column,
        }
    }
    pub fn get_priority(&self) -> i32 {
        match &self.token_type {
            TokenKind::Operator(operator_token_data) => operator_token_data.get_priority(),
            _ => -1,
        }
    }
}

pub enum ParsingErrorKind {
    UnknownCharacter,
}
pub struct ParsingError {
    error_type: ParsingErrorKind,
    message: Option<&'static str>,
}

impl ParsingError {
    pub fn new(error_type: ParsingErrorKind) -> Self {
        Self {
            error_type,
            message: None,
        }
    }
}

pub struct Lexer<'a> {
    code: &'a str,
    tokens: Vec<Token<'a>>,
    chars_indices: core::iter::Peekable<core::str::CharIndices<'a>>,
    column: usize,
    row: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(code: &'a str) -> Self {
        Self {
            code,
            tokens: Vec::new(),
            chars_indices: code.char_indices().peekable(),
            row: 0,
            column: 0,
        }
    }

    pub fn peek_char(&mut self) -> Option<char> {
        self.chars_indices.peek().map(|(_, ch)| ch).copied()
    }

    /// Check if next sequence of characters is the same as given string without advancing primary iterator
    pub fn is_string(&self, value: &'static str) -> bool {
        let mut tmp = self.chars_indices.clone();

        for i in value.chars() {
            let curr = tmp.next();
            if curr.is_none() || curr.unwrap().1 != i {
                return false;
            }
        }

        // check if we reached end of the line or the next character is not suitable
        return !tmp.next().is_some_and(|ch| ch.1.is_alphanumeric());
    }
    fn next_char(&mut self) -> Option<char> {
        self.chars_indices
            .next()
            .inspect(|(idx, ch)| {
                if *ch == '\n' {
                    self.row += 1;
                    self.column = 0;
                } else {
                    self.column += 1;
                }
            })
            .map(|(_, ch)| ch)
    }

    /// Advance code iterator by given amount or until the end of string
    fn advance_by(&mut self, amount: usize) {
        let mut i: usize = 0;
        while i < amount && self.next_char().is_some() {
            i += 1
        }
    }

    fn next_char_if<F>(&mut self, func: F) -> Option<char>
    where
        F: Fn(char) -> bool,
    {
        if func(self.chars_indices.peek().map(|(_, ch)| *ch)?) {
            self.next_char()
        } else {
            None
        }
    }

    pub fn skip_char_while<F>(&mut self, f: F)
    where
        F: Fn(char) -> bool,
    {
        while self.next_char_if(&f).is_some() {}
    }

    /// Collect characters as long as predicate is true
    pub fn collect_char_while<F>(&mut self, f: F) -> Option<String>
    where
        F: Fn(char) -> bool,
    {
        let mut s = String::new();
        while let Some(ch) = self.next_char_if(&f) {
            s.push(ch);
        }
        if s.len() == 0 { None } else { Some(s) }
    }

    /// Take a long string which can contain identifiers or quoted string
    pub fn take_string(&mut self) -> Option<String> {
        if self.peek_char() == Some('"') {
        } else {
            // handle keywords
            //
            return self.collect_char_while(|c| c.is_alphabetic());
        }
        None
    }

    pub fn tokenize_keyword(&mut self) -> Option<Token<'a>> {
        if !self.peek_char().is_some_and(char::is_alphabetic) {
            return None;
        }
        let mut it = self.chars_indices.clone();

        let mut str_len: usize = 0;
        if let Some((start, _)) = it.peek().cloned() {
            while it.next_if(|(_, ch)| ch.is_alphabetic()).is_some() {
                str_len += 1;
            }
            let keyword_str = &self.code[start..str_len];
            if let Some(keyword) = Keyword::from_str(keyword_str) {
                let tok = Token::new(TokenKind::Keyword(keyword), self.row, self.column);
                self.advance_by(str_len);
                return Some(tok);
            }
        }

        None
    }

    pub const fn convert_special(ch: &str) -> Option<char> {
        match ch.as_bytes() {
            b"\\n" => Some('\n'),
            b"\\\"" => Some('"'),
            b"\\'" => Some('\''),
            b"\\t" => Some('\t'),
            b"\\\\" => Some('\\'),
            _ => None,
        }
    }

    pub fn tokenize_string(&mut self) -> Option<Token<'a>> {
        if !self.peek_char().is_some_and(|c| c == '"') {
            return None;
        }
        let mut it = self.chars_indices.clone();
        it.next();
        let mut const_str = String::new();
        let mut offset: usize = 1;
        while let Some((i, ch)) = it.next_if(|(_, ch)| *ch != '"') {
            if it.peek().is_some()
                && let Some(spec) = Self::convert_special(&self.code[i..(i + 2)])
            {
                it.next();
                const_str.push(spec);
                offset += 1;
            } else {
                const_str.push(ch);
            }
        }
        if it.peek().is_none_or(|(_, c)| *c != '"') {
            return None;
        }
        let tok = Token::new(TokenKind::ConstantString(const_str), self.row, self.column);
        self.advance_by(offset + 1);

        Some(tok)
    }

    pub fn tokenize_id(&mut self) -> Option<Token<'a>> {
        if !self
            .peek_char()
            .is_some_and(|ch| ch.is_alphabetic() || ch == '_')
        {
            return None;
        }

        let mut it = self.chars_indices.clone();

        let mut str_len: usize = 0;
        if let Some((start, _)) = it.peek().cloned() {
            while it
                .next_if(|(_, c)| c.is_alphanumeric() || *c == '_')
                .is_some()
            {
                str_len += 1;
            }
            let tok = Token::new(
                TokenKind::Identifier(&self.code[start..(start + str_len)]),
                self.row,
                self.column,
            );
            self.advance_by(str_len);
            return Some(tok);
        }

        None
    }

    pub fn tokenize(&mut self) -> Result<(), ParsingError> {
        while self.peek_char().is_some() {
            self.skip_char_while(char::is_whitespace);

            let token = self
                .tokenize_keyword()
                .or(self.tokenize_id())
                .or(self.tokenize_string())
                .ok_or(ParsingError::new(ParsingErrorKind::UnknownCharacter))?;

            self.tokens.push(token);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::codegen::lexer::{Keyword, Lexer, Token, TokenKind};

    #[test]
    fn test_keyword_lexing() {
        let code = "let";
        let mut lexer = Lexer::new(code);

        let res = lexer.tokenize_keyword();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_type,
            TokenKind::Keyword(Keyword::Let)
        ));
    }

    #[test]
    fn test_keyword_lexing_boundry() {
        let code = "letting";
        let mut lexer = Lexer::new(code);

        let res = lexer.tokenize_keyword();
        assert!(res.is_none());
    }

    #[test]
    fn test_id() {
        let code = "var1";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_type,
            TokenKind::Identifier("var1")
        ));
    }

    #[test]
    fn test_id_extra() {
        let code = "var1=";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_type,
            TokenKind::Identifier("var1")
        ));
    }

    #[test]
    fn test_id_boundary() {
        let code = "va_r1()";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_type,
            TokenKind::Identifier("va_r1")
        ));
    }

    #[test]
    fn test_string_full() {
        let code = r#""inner""#;
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_string();
        assert!(res.is_some());
        match res.unwrap().token_type {
            TokenKind::ConstantString(s) => assert_eq!(s, "inner".to_owned()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_string_with_special() {
        let code = r#""in\"n\"er""#;
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_string();
        assert!(res.is_some());
        match res.unwrap().token_type {
            TokenKind::ConstantString(s) => assert_eq!(s, "in\"n\"er".to_owned()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_string_incomplete() {
        let code = r#""inner"#;
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_string();
        assert!(res.is_none());
    }
}
