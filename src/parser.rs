use crate::ast::*;
use crate::lexer::*;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum OperatorPrecedence {
    MulDiv = 100,
    AddSubtract = 50,
    Zero = 0,
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
        _ => panic!(
            "Trying to convert a non operator token type to a binary operator type, {:?}",
            token_type
        ),
    }
}

fn get_operator_precedence(token_type: TokenType) -> OperatorPrecedence {
    match token_type {
        TokenType::Plus => OperatorPrecedence::AddSubtract,
        TokenType::Minus => OperatorPrecedence::AddSubtract,
        TokenType::Star => OperatorPrecedence::MulDiv,
        TokenType::Slash => OperatorPrecedence::MulDiv,
        _ => panic!(
            "Trying to convert a non operator token type to an operator precedence, {:?}",
            token_type
        ),
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, index: 0 }
    }

    fn peek(&self, index: usize) -> &Token {
        if self.index + index >= self.tokens.len() {
            panic!("Reached end of tokenstream while peeking!");
        }
        &self.tokens[self.index + index]
    }

    fn consume(&mut self) -> &Token {
        if self.eof() {
            panic!("Reached end of tokenstream while consuming!");
        }
        let result = &self.tokens[self.index];
        self.index += 1;

        result
    }

    fn assert_consume(&mut self, token_type: TokenType) -> &Token {
        let token = self.consume();
        assert!(token.token_type == token_type);
        token
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
            PrimitiveType::Int32,
            PrimitiveValue {
                int32: self.consume().value.parse::<i32>().unwrap(),
            },
        )
    }

    /// Converts an expression of binary operators into an AST
    ///
    /// It uses the pratt parsing algorithm to recursively construct the
    /// AST with the correct precedence rules.
    fn parse_expression(&mut self, precedence: OperatorPrecedence) -> AstNode {
        let break_condition = |token: &Token| token.token_type == TokenType::SemiColon;

        let mut left = self.parse_unary_expression();

        let mut operator = self.peek(0);

        if break_condition(operator) {
            return left;
        }

        let mut operator_type = token_type_to_operator(operator.token_type);
        let mut current_precedence = get_operator_precedence(operator.token_type);

        while current_precedence > precedence {
            self.consume();

            let right = self.parse_expression(current_precedence);

            left = AstNode::BinaryOperation(operator_type, Box::new(left), Box::new(right));

            operator = self.peek(0);

            if break_condition(operator) {
                return left;
            }

            operator_type = token_type_to_operator(operator.token_type);
            current_precedence = get_operator_precedence(operator.token_type)
        }

        left
    }

    fn parse_variable_declaration(&mut self) -> AstNode {
        self.assert_consume(TokenType::Var);
        let name = self.assert_consume(TokenType::Identifier).value.clone();
        self.assert_consume(TokenType::SemiColon);

        AstNode::VariableDeclaration(name, PrimitiveType::Int32)
    }

    fn parse_assignment(&mut self) -> AstNode {
        let identifier_name = self.consume().value.clone();
        self.assert_consume(TokenType::EqualSign);

        let expression = self.parse_expression(OperatorPrecedence::Zero);
        self.consume();
        AstNode::Assignment(identifier_name, Box::new(expression))
    }

    fn parse_block(&mut self) -> AstNode {
        let mut children: Vec<AstNode> = vec![];

        self.consume();

        while self.peek(0).token_type != TokenType::RightBrace {
            let node = self.parse();
            children.push(node);
        }

        self.consume();

        AstNode::Block(children)
    }

    pub fn parse(&mut self) -> AstNode {
        let next_token: &Token = self.peek(0);
        let node = match next_token.token_type {
            TokenType::LeftBrace => self.parse_block(),
            TokenType::Var => self.parse_variable_declaration(),
            TokenType::Identifier => self.parse_assignment(),
            _ => panic!("Unexpected token: {:?}", next_token),
        };
        node
    }
}
