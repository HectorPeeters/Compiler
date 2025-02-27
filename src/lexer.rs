use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    IntLiteral,

    Plus,
    Minus,
    Star,
    Slash,

    ExclamationMark,

    Identifier,
    EqualSign,

    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,

    SemiColon,
    Colon,
    Comma,
    Var,
    If,
    Else,
    While,
    Function,
    Type,

    DoubleEqualSign,
    NotEqualSign,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
}

#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub col: usize,
    pub line: usize,
}

pub struct Lexer<'a> {
    data: Vec<&'a str>,
    index: usize,
    current_col: usize,
    current_line: usize,
}

fn is_whitespace(string: &str) -> bool {
    string == " " || string == "\t"
}

fn is_newline(string: &str) -> bool {
    string == "\r\n" || string == "\n"
}

fn is_alphabetic(string: &str) -> bool {
    string.chars().all(|x: char| x.is_alphabetic())
}

fn is_numeric(string: &str) -> bool {
    string.chars().all(|x: char| x.is_numeric())
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Lexer {
            data: UnicodeSegmentation::graphemes(input, true).collect::<Vec<&str>>(),
            index: 0,
            current_col: 1,
            current_line: 1,
        }
    }

    fn error(&self, message: &str) {
        eprintln!(
            "Lexer error at line {}:{}\n{}",
            self.current_line, self.current_col, message
        );
        panic!();
    }

    fn eof(&mut self) -> bool {
        self.index >= self.data.len()
    }

    fn peek(&self, index: usize) -> String {
        self.data[self.index + index].to_owned()
    }

    fn consume(&mut self) -> &str {
        let result = self.data[self.index];
        self.index += 1;

        self.current_col += 1;
        if is_newline(result) {
            self.current_col = 1;
            self.current_line += 1;
        }

        result
    }

    fn consume_while(&mut self, f: fn(&str) -> bool) -> String {
        let mut result = String::default();

        loop {
            if self.eof() {
                break;
            }

            let c = self.peek(0);
            if !f(&c) {
                break;
            }

            result.push_str(self.consume());
        }

        result
    }

    fn skip_whitespace(&mut self) {
        self.consume_while(|c| is_whitespace(c) || is_newline(c));
    }

    fn tokenize_single_char(&mut self, token_type: TokenType) -> Token {
        let value = String::from(self.consume());
        Token {
            line: self.current_line,
            col: self.current_col - value.len(),
            token_type,
            value,
        }
    }

    fn tokenize_multichar(&mut self, condition: fn(&str) -> bool, token_type: TokenType) -> Token {
        let value = self.consume_while(condition);
        Token {
            line: self.current_line,
            col: self.current_col - value.len(),
            token_type,
            value,
        }
    }

    fn keyword_to_tokentype(keyword: &str) -> Option<TokenType> {
        match keyword {
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "var" => Some(TokenType::Var),
            "while" => Some(TokenType::While),
            "fn" => Some(TokenType::Function),
            "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "bool" => {
                Some(TokenType::Type)
            }
            _ => None,
        }
    }

    fn tokenize_possible_keyword(&mut self) -> Token {
        let value = self.consume_while(|c| is_alphabetic(c) || is_numeric(c));

        let token_type =
            Self::keyword_to_tokentype(&value).unwrap_or(TokenType::Identifier);

        Token {
            line: self.current_line,
            col: self.current_col - value.len(),
            token_type,
            value,
        }
    }

    fn tokenize_possible_multichar(
        &mut self,
        single_type: TokenType,
        multiple_type: TokenType,
        next_char: &str,
    ) -> Token {
        let mut value = String::from(self.consume());
        let mut token_type = single_type;

        if self.peek(0) == next_char {
            value.push_str(self.consume());
            token_type = multiple_type;
        }

        Token {
            line: self.current_line,
            col: self.current_col - value.len(),
            token_type,
            value,
        }
    }

    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut result: Vec<Token> = Vec::new();

        while !self.eof() {
            self.skip_whitespace();

            if self.eof() {
                break;
            }

            while self.peek(0) == "#" {
                self.consume_while(|c| !is_newline(c));
                self.consume();
                self.skip_whitespace();
            }

            let current_char = self.peek(0);

            let token = match current_char.chars().next().unwrap() {
                '0'..='9' => Some(self.tokenize_multichar(is_numeric, TokenType::IntLiteral)),
                'a'..='z' | 'A'..='Z' => Some(self.tokenize_possible_keyword()),
                '+' => Some(self.tokenize_single_char(TokenType::Plus)),
                '-' => Some(self.tokenize_single_char(TokenType::Minus)),
                '*' => Some(self.tokenize_single_char(TokenType::Star)),
                '/' => Some(self.tokenize_single_char(TokenType::Slash)),
                '(' => Some(self.tokenize_single_char(TokenType::LeftParen)),
                ')' => Some(self.tokenize_single_char(TokenType::RightParen)),
                '{' => Some(self.tokenize_single_char(TokenType::LeftBrace)),
                '}' => Some(self.tokenize_single_char(TokenType::RightBrace)),
                ';' => Some(self.tokenize_single_char(TokenType::SemiColon)),
                ':' => Some(self.tokenize_single_char(TokenType::Colon)),
                ',' => Some(self.tokenize_single_char(TokenType::Comma)),
                '!' => Some(self.tokenize_possible_multichar(
                    TokenType::ExclamationMark,
                    TokenType::NotEqualSign,
                    "=",
                )),
                '=' => Some(self.tokenize_possible_multichar(
                    TokenType::EqualSign,
                    TokenType::DoubleEqualSign,
                    "=",
                )),
                '<' => Some(self.tokenize_possible_multichar(
                    TokenType::LessThan,
                    TokenType::LessThanOrEqual,
                    "=",
                )),
                '>' => Some(self.tokenize_possible_multichar(
                    TokenType::GreaterThan,
                    TokenType::GreaterThanOrEqual,
                    "=",
                )),
                _ => None,
            };

            match token {
                Some(x) => result.push(x),
                None => self.error(&format!("Unexpected character: {}", current_char)),
            }
        }
        result
    }
}
