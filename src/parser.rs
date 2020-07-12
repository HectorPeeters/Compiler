use crate::ast::*;
use crate::lexer::*;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum OperatorPrecedence {
    MulDiv = 100,
    AddSubtract = 50,
    None = 0,
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

fn token_type_to_operator(token_type: TokenType) -> BinaryOperationType {
    match token_type {
        TokenType::Plus => BinaryOperationType::Add,
        TokenType::Minus => BinaryOperationType::Subtract,
        TokenType::Star => BinaryOperationType::Multiply,
        TokenType::Slash => BinaryOperationType::Divide,
        _ => panic!("Trying to convert a non operator token type to a binary operator type"),
    }
}

fn get_operator_precedence(token_type: TokenType) -> OperatorPrecedence {
    match token_type {
        TokenType::Plus => OperatorPrecedence::AddSubtract,
        TokenType::Minus => OperatorPrecedence::AddSubtract,
        TokenType::Star => OperatorPrecedence::MulDiv,
        TokenType::Slash => OperatorPrecedence::MulDiv,
        _ => panic!("Trying to convert a non operator token type to an operator precedence"),
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            index: 0,
        }
    }

    fn peek(&self, index: usize) -> &Token {
        &self.tokens[self.index + index]
    }

    fn consume(&mut self) -> &Token {
        let result = &self.tokens[self.index];
        self.index += 1;

        result
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn parse_unary_expression(&mut self) -> AstNode {
        let current_token = self.peek(0);
        if current_token.token_type != TokenType::IntLiteral {
            panic!("parse_unary_expression expects IntLiteral token type");
        }

        AstNode::NumericLiteral(
            PrimitiveType::Int64,
            PrimitiveValue {
                int64: self.consume().value.parse::<i64>().unwrap(),
            },
        )
    }

    pub fn parse_expression(&mut self, precedence: OperatorPrecedence) -> AstNode {
        let mut left = self.parse_unary_expression();

        if self.eof() {
            return left;
        }

        let mut operator = self.peek(0);
        let mut operator_type = token_type_to_operator(operator.token_type);
        let mut current_precedence = get_operator_precedence(operator.token_type);

        while current_precedence > precedence {
            self.consume();

            let right = self.parse_expression(current_precedence);

            left = AstNode::BinaryOperation(operator_type, Box::new(left), Box::new(right));

            if self.eof() {
                break;
            }

            operator = self.peek(0);
            operator_type = token_type_to_operator(operator.token_type);
            current_precedence = get_operator_precedence(operator.token_type)
        }

        left
    }
}
