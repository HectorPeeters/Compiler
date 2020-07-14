use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    IntLiteral,
    Plus,
    Minus,
    Star,
    Slash,
    Identifier,
    EqualSign,
    LeftParen,
    RightParen,
    SemiColon,
    Var,
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
            if !f(c.as_str()) {
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
            "var" => Some(TokenType::Var),
            _ => None,
        }
    }

    fn tokenize_possible_keyword(&mut self) -> Token {
        let value = self.consume_while(is_alphabetic);

        let token_type =
            Self::keyword_to_tokentype(value.as_str()).unwrap_or(TokenType::Identifier);

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
                ';' => Some(self.tokenize_single_char(TokenType::SemiColon)),
                '=' => Some(self.tokenize_single_char(TokenType::EqualSign)),
                _ => None,
            };

            match token {
                Some(x) => result.push(x),
                None => {
                    panic!("Error while tokenizing {}", current_char);
                }
            }
        }
        result
    }
}
