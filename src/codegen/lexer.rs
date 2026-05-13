use std::{
    error::Error,
    fmt::{Display, write},
};

#[derive(Debug, Clone, Copy, PartialEq)]
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
            "fun" => Some(Self::Function),
            "if" => Some(Self::If),
            "else" => Some(Self::Else),
            "elif" => Some(Self::Elif),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Separator {
    BracketOpen,
    BracketClose,
    BlockOpen,
    BlockClose,
    ArrayOpen,
    ArrayClose,
    Dot,
    Comma,
    End,
    Colon,
}

impl Separator {
    pub fn from_char(ch: char) -> Option<Self> {
        match ch {
            '(' => Some(Separator::BracketOpen),
            ')' => Some(Separator::BracketClose),
            '{' => Some(Separator::BlockOpen),
            '}' => Some(Separator::BlockClose),
            '[' => Some(Separator::ArrayOpen),
            ']' => Some(Separator::ArrayClose),
            '.' => Some(Separator::Dot),
            ',' => Some(Separator::Comma),
            ';' => Some(Separator::End),
            ':' => Some(Separator::Colon),
            _ => None,
        }
    }

    pub fn get_priority(&self) -> i32 {
        match self {
            Separator::BracketOpen => -1,
            Separator::BracketClose => -1,
            Separator::BlockOpen => 1,
            Separator::BlockClose => 2,
            Separator::ArrayOpen => -1,
            Separator::ArrayClose => -1,
            Separator::Dot => 2,
            Separator::Comma => -1,
            Separator::End => -1,
            Separator::Colon => -1,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind<'a> {
    Keyword(Keyword),
    Operator(OperatorTokenData),
    Separator(Separator),
    Identifier(&'a str),
    ConstantString(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
}

impl<'a> Display for TokenKind<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Keyword(keyword) => write!(f, "KEYWORD({:?})", keyword),
            TokenKind::Operator(operator_token_data) => {
                write!(f, "OP({:?})", operator_token_data.operator)
            }
            TokenKind::Separator(separator) => write!(f, "SEP({:?})", separator),
            TokenKind::Identifier(id) => write!(f, "ID({})", id),
            TokenKind::ConstantString(const_str) => write!(f, "STR{})", const_str),
            TokenKind::Integer(i) => write!(f, "{}", i),
            TokenKind::Number(n) => write!(f, "{}", n),
            TokenKind::Boolean(b) => write!(f, "{}", b),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    token_kind: TokenKind<'a>,
    row: usize,
    column: usize,
}

impl<'a> Token<'a> {
    pub fn new(token_kind: TokenKind<'a>, row: usize, column: usize) -> Self {
        Self {
            token_kind,
            row,
            column,
        }
    }

    pub fn get_row(&self) -> usize {
        self.row
    }

    pub fn get_column(&self) -> usize {
        self.column
    }

    pub fn get_priority(&self) -> i32 {
        match &self.token_kind {
            TokenKind::Operator(operator_token_data) => operator_token_data.get_priority(),
            _ => -1,
        }
    }
}

#[derive(Debug)]
pub enum ParsingErrorKind {
    UnknownCharacter,
    InvalidIntegerConstant,
    InvalidNumberConstant,
}
#[derive(Debug)]

pub struct ParsingError {
    error_type: ParsingErrorKind,
    message: Option<&'static str>,
    row: usize,
    column: usize,
}

impl ParsingError {
    pub fn new(error_type: ParsingErrorKind, row: usize, column: usize) -> Self {
        Self {
            error_type,
            message: None,
            row,
            column,
        }
    }
}

impl Error for ParsingError {}

impl Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?}: {} Row: {} Column:{} ",
            self.error_type,
            self.message.unwrap_or(""),
            self.row,
            self.column
        )
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

    /// Advance primary iterator and return the character. Adjusts column and row values.
    /// If string is over returns None
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
        while i < amount && self.chars_indices.next().is_some() {
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
            let keyword_str = &self.code[start..(start + str_len)];
            if let Some(keyword) = Keyword::from_str(keyword_str) {
                let tok = Token::new(TokenKind::Keyword(keyword), self.row, self.column);
                self.advance_by(str_len);
                return Some(tok);
            }
        }

        None
    }

    pub fn tokenize_integer(&mut self) -> Result<Option<Token<'a>>, ParsingError> {
        if !self.peek_char().is_some_and(char::is_numeric) {
            return Ok(None);
        }

        let mut it = self.chars_indices.clone();
        if let Some((start, _)) = it.peek().cloned() {
            let mut len: usize = 0;
            while it.next_if(|(_, c)| c.is_numeric()).is_some() {
                len += 1;
            }
            let tok = Token::new(
                TokenKind::Integer((&self.code[start..(start + len)]).parse::<i64>().map_err(
                    |e| {
                        ParsingError::new(
                            ParsingErrorKind::InvalidIntegerConstant,
                            self.row,
                            self.column,
                        )
                    },
                )?),
                self.row,
                self.column,
            );

            self.advance_by(len);
            Ok(Some(tok))
        } else {
            Ok(None)
        }
    }

    pub fn tokenize_number(&mut self) -> Result<Option<Token<'a>>, ParsingError> {
        if !self.peek_char().is_some_and(char::is_numeric) {
            return Ok(None);
        }

        let mut it = self.chars_indices.clone();
        if let Some((start, _)) = it.peek().cloned() {
            let mut len: usize = 0;
            let mut has_separator: bool = false;
            while let Some((_, c)) = it.next_if(|(_, c)| c.is_numeric() || *c == '.') {
                if c == '.' && has_separator {
                    return Err(ParsingError::new(
                        ParsingErrorKind::InvalidNumberConstant,
                        self.row,
                        self.column,
                    ));
                } else if c == '.' && !has_separator {
                    has_separator = true;
                }
                len += 1;
            }
            let tok = Token::new(
                TokenKind::Number((&self.code[start..(start + len)]).parse::<f64>().map_err(
                    |e| {
                        ParsingError::new(
                            ParsingErrorKind::InvalidNumberConstant,
                            self.row,
                            self.column,
                        )
                    },
                )?),
                self.row,
                self.column,
            );

            self.advance_by(len);
            Ok(Some(tok))
        } else {
            Ok(None)
        }
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
    pub const fn bool_from_str(s: &str) -> Option<bool> {
        match s.as_bytes() {
            b"true" => Some(true),
            b"false" => Some(false),
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

    pub fn tokenize_bool(&mut self) -> Option<Token<'a>> {
        if !self.peek_char().is_some_and(char::is_alphabetic) {
            return None;
        }
        let mut it = self.chars_indices.clone();

        let mut str_len: usize = 0;
        if let Some((start, _)) = it.peek().cloned() {
            while it.next_if(|(_, ch)| ch.is_alphabetic()).is_some() {
                str_len += 1;
            }
            let keyword_str = &self.code[start..(start + str_len)];
            if let Some(b) = Self::bool_from_str(keyword_str) {
                let tok = Token::new(TokenKind::Boolean(b), self.row, self.column);
                self.advance_by(str_len);
                return Some(tok);
            }
        }

        None
    }

    pub fn tokenize_separator(&mut self) -> Option<Token<'a>> {
        if let Some(ch) = self.peek_char().clone()
            && let Some(sep) = Separator::from_char(ch)
        {
            let tok = Token::new(TokenKind::Separator(sep), self.row, self.column);
            self.next_char();
            Some(tok)
        } else {
            None
        }
    }

    pub fn tokenize(&mut self) -> Result<(), ParsingError> {
        while self.peek_char().is_some() {
            self.skip_char_while(char::is_whitespace);

            if let Some(t) = self.tokenize_integer()? {
                self.tokens.push(t);
            }
            if let Some(t2) = self.tokenize_number()? {
                self.tokens.push(t2);
            } else {
                let token = self
                    .tokenize_keyword()
                    .or_else(|| self.tokenize_bool())
                    .or_else(|| self.tokenize_id())
                    .or_else(|| self.tokenize_string())
                    .or_else(|| self.tokenize_separator())
                    .ok_or(ParsingError::new(
                        ParsingErrorKind::UnknownCharacter,
                        self.row,
                        self.column,
                    ))?;

                self.tokens.push(token);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::codegen::lexer::{Keyword, Lexer, Separator, Token, TokenKind};

    #[test]
    fn test_advance_by() {
        let code = "main()";
        let mut lexer = Lexer::new(code);
        lexer.skip_char_while(char::is_whitespace);
        lexer.advance_by("main".len());
        assert_eq!(lexer.chars_indices.peek().unwrap().1, '(');
    }

    #[test]
    fn test_dont_skip_char() {
        let code = "main()";
        let mut lexer = Lexer::new(code);
        assert!(lexer.tokenize_id().is_some());
        lexer.skip_char_while(char::is_whitespace);
        assert_eq!(lexer.chars_indices.peek().unwrap().1, '(');

        let res = lexer.tokenize_separator();
        assert!(matches!(
            res.unwrap().token_kind,
            TokenKind::Separator(Separator::BracketOpen)
        ));
    }

    #[test]
    fn test_id_advance() {
        let code = "main()";
        let mut lexer = Lexer::new(code);
        lexer.skip_char_while(char::is_whitespace);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert_eq!(lexer.chars_indices.peek().unwrap().1, '(');
    }
    #[test]
    fn test_keyword_lexing() {
        let code = "let";
        let mut lexer = Lexer::new(code);

        let res = lexer.tokenize_keyword();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_kind,
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
            res.unwrap().token_kind,
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
            res.unwrap().token_kind,
            TokenKind::Identifier("var1")
        ));
    }

    #[test]
    fn test_id_extra2() {
        let code = " main(";
        let mut lexer = Lexer::new(code);
        lexer.skip_char_while(char::is_whitespace);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_kind,
            TokenKind::Identifier("main")
        ));
    }

    #[test]
    fn test_id_boundary() {
        let code = "va_r1()";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_id();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_kind,
            TokenKind::Identifier("va_r1")
        ));
    }

    #[test]
    fn test_string_full() {
        let code = r#""inner""#;
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_string();
        assert!(res.is_some());
        match res.unwrap().token_kind {
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
        match res.unwrap().token_kind {
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

    #[test]
    fn test_integer() {
        let code = "69420";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_integer();
        assert!(matches!(
            res.unwrap().unwrap().token_kind,
            TokenKind::Integer(69420)
        ));
    }

    #[test]
    fn test_number() {
        let code = "69.420";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_number();
        assert!(matches!(
            res.unwrap().unwrap().token_kind,
            TokenKind::Number(69.420)
        ));
    }

    #[test]
    fn test_integer_incorrect() {
        let code = "v69420";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_integer();
        assert!(res.unwrap().is_none());
    }

    #[test]
    fn test_integer_incorrect_too_large() {
        let code = "69420000000000000000000000000000";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_integer();
        assert!(res.is_err());
    }

    #[test]
    fn test_separator() {
        let code = "(";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_separator();
        assert!(res.is_some());
        assert!(matches!(
            res.unwrap().token_kind,
            TokenKind::Separator(Separator::BracketOpen)
        ));
    }

    #[test]
    fn test_separator_fail() {
        let code = "0";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_separator();
        assert!(res.is_none());
    }

    #[test]
    fn test_bool_true() {
        let code = "true";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_bool();
        assert!(matches!(res.unwrap().token_kind, TokenKind::Boolean(true)));
    }

    #[test]
    fn test_bool_false() {
        let code = "false";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_bool();
        assert!(matches!(res.unwrap().token_kind, TokenKind::Boolean(false)));
    }

    #[test]
    fn test_bool_true_with_extra() {
        let code = "trueer";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_bool();
        assert!(res.is_none());
    }

    #[test]
    fn test_bool_true_with_extra2() {
        let code = "true()";
        let mut lexer = Lexer::new(code);
        let res = lexer.tokenize_bool();
        assert!(res.is_some());
        assert!(matches!(res.unwrap().token_kind, TokenKind::Boolean(true)));
    }

    #[test]
    fn test_tokenize_snippet() -> Result<(), Box<dyn Error>> {
        let code = "fun main() {}";
        let mut lexer = Lexer::new(code);
        lexer.tokenize().unwrap();
        let tokens = lexer.tokens;

        let expected = [
            TokenKind::Keyword(Keyword::Function),
            TokenKind::Identifier("main"),
            TokenKind::Separator(Separator::BracketOpen),
            TokenKind::Separator(Separator::BracketClose),
            TokenKind::Separator(Separator::BlockOpen),
            TokenKind::Separator(Separator::BlockClose),
        ];

        assert_eq!(tokens.len(), expected.len());
        for (t, e) in tokens.iter().zip(expected.iter()) {
            assert_eq!(&t.token_kind, e);
        }

        Ok(())
    }
}
