use crate::ast::*;
use crate::lexer::*;
use crate::scope::*;
use crate::types::*;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum OperatorPrecedence {
    //GreaterLessThan = 200,
    EqualsNotEquals = 150,
    MulDiv = 100,
    AddSubtract = 50,
    Zero = 0,
}

pub struct Parser {
    tokens: Vec<Token>,
    index: usize,
    scope: Vec<Scope>,
}

fn token_type_to_operator(token_type: TokenType) -> BinaryOperationType {
    match token_type {
        TokenType::Plus => BinaryOperationType::Add,
        TokenType::Minus => BinaryOperationType::Subtract,
        TokenType::Star => BinaryOperationType::Multiply,
        TokenType::Slash => BinaryOperationType::Divide,
        TokenType::DoubleEqualSign => BinaryOperationType::Equals,
        TokenType::NotEqualSign => BinaryOperationType::NotEquals,
        _ => panic!(
            "Trying to convert a non operator token type to a binary operator type, {:?}",
            token_type
        ),
    }
}

fn get_operator_precedence(operation_type: BinaryOperationType) -> OperatorPrecedence {
    match operation_type {
        BinaryOperationType::Add | BinaryOperationType::Subtract => OperatorPrecedence::AddSubtract,
        BinaryOperationType::Multiply | BinaryOperationType::Divide => OperatorPrecedence::MulDiv,
        BinaryOperationType::Equals | BinaryOperationType::NotEquals => {
            OperatorPrecedence::EqualsNotEquals
        }
        _ => panic!(
            "Trying to convert a non operator token type to an operator precedence, {:?}",
            operation_type
        ),
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            index: 0,
            scope: vec![Scope::new()],
        }
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
        assert!(
            token.token_type == token_type,
            "{:?} == {:?}",
            token.token_type,
            token_type
        );
        token
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn find_scope_var(&self, name: &String) -> Option<&Symbol> {
        for scope in self.scope.iter().rev() {
            if let Some(var) = scope.get(name) {
                return Some(&var);
            }
        }

        None
    }

    fn parse_unary_expression(&mut self) -> AstNode {
        let current_token = self.peek(0);
        if current_token.token_type != TokenType::IntLiteral
            && current_token.token_type != TokenType::LeftParen
            && current_token.token_type != TokenType::Identifier
        {
            panic!("parse_unary_expression expects IntLiteral, LeftParen or Identifier token type");
        }

        match current_token.token_type {
            TokenType::LeftParen => {
                self.assert_consume(TokenType::LeftParen);
                let expression = self.parse_expression(OperatorPrecedence::Zero);
                self.assert_consume(TokenType::RightParen);
                expression
            }
            TokenType::IntLiteral => {
                let value = self
                    .assert_consume(TokenType::IntLiteral)
                    .value
                    .parse::<u64>()
                    .unwrap();
                let mut primitive_type = PrimitiveType::UInt8;

                if value > 2u64.pow(32) - 1 {
                    primitive_type = PrimitiveType::UInt64;
                } else if value > 2u64.pow(16) - 1 {
                    primitive_type = PrimitiveType::UInt32;
                } else if value > 2u64.pow(8) - 1 {
                    primitive_type = PrimitiveType::UInt16;
                }

                AstNode::NumericLiteral(primitive_type, PrimitiveValue { uint64: value })
            }
            TokenType::Identifier => {
                let identifier = self.assert_consume(TokenType::Identifier).value.clone();
                let scope_var = self
                    .find_scope_var(&identifier)
                    .expect("Undefined identifier!");
                AstNode::Identifier(scope_var.clone())
            }
            _ => unreachable!(),
        }
    }

    /// Converts an expression of binary operators into an AST
    ///
    /// It uses the pratt parsing algorithm to recursively construct the
    /// AST with the correct precedence rules.
    fn parse_expression(&mut self, precedence: OperatorPrecedence) -> AstNode {
        let break_condition = |token: &Token| {
            token.token_type == TokenType::SemiColon
                || token.token_type == TokenType::RightParen
                || token.token_type == TokenType::Comma
                || token.token_type == TokenType::LeftBrace
        };

        let mut left = self.parse_unary_expression();

        let mut operator = self.peek(0);

        if break_condition(operator) {
            return left;
        }

        let mut operator_type = token_type_to_operator(operator.token_type);
        let mut current_precedence = get_operator_precedence(operator_type);

        while current_precedence > precedence {
            self.consume();

            let mut right = self.parse_expression(current_precedence);

            if !left
                .get_primitive_type()
                .is_compatible_with(&right.get_primitive_type(), false)
            {
                panic!("Incompatible types in expression");
            }

            if left.get_primitive_type().get_size() > right.get_primitive_type().get_size() {
                right = AstNode::Widen(left.get_primitive_type(), Box::new(right));
            } else if left.get_primitive_type().get_size() < right.get_primitive_type().get_size() {
                left = AstNode::Widen(right.get_primitive_type(), Box::new(left));
            }

            left = AstNode::BinaryOperation(operator_type, Box::new(left), Box::new(right));

            operator = self.peek(0);

            if break_condition(operator) {
                return left;
            }

            operator_type = token_type_to_operator(operator.token_type);
            current_precedence = get_operator_precedence(operator_type)
        }

        left
    }

    fn parse_variable_type(&mut self) -> PrimitiveType {
        let type_token = self.assert_consume(TokenType::Type);
        type_token
            .value
            .parse::<PrimitiveType>()
            .unwrap_or_else(|_| panic!("Unknown primitive type: {}", type_token.value))
    }

    fn parse_variable_declaration(&mut self) -> AstNode {
        self.assert_consume(TokenType::Var);
        let name = self.assert_consume(TokenType::Identifier).value.clone();
        self.assert_consume(TokenType::Colon);
        let primitive_type = self.parse_variable_type();
        self.assert_consume(TokenType::SemiColon);

        let scope: &mut Scope = self.scope.last_mut().expect("No scope in scope stack");
        let symbol = scope.add(&name, SymbolType::Variable, primitive_type);

        AstNode::VariableDeclaration(symbol.clone())
    }

    fn parse_assignment(&mut self) -> AstNode {
        let identifier_name = self.consume().value.clone();
        self.assert_consume(TokenType::EqualSign);

        let mut expression = self.parse_expression(OperatorPrecedence::Zero);
        self.consume();

        let scope_var = self
            .find_scope_var(&identifier_name)
            .expect("Unknown identifier");

        if scope_var.primitive_type.get_size() > expression.get_primitive_type().get_size() {
            expression = AstNode::Widen(scope_var.primitive_type, Box::new(expression));
        }

        AstNode::Assignment(scope_var.clone(), Box::new(expression))
    }

    fn parse_functioncall(&mut self) -> AstNode {
        let name = self.assert_consume(TokenType::Identifier).value.clone();

        self.assert_consume(TokenType::LeftParen);

        let mut params: Vec<AstNode> = Vec::new();

        //TODO: check parameter types
        loop {
            let expression = self.parse_expression(OperatorPrecedence::Zero);
            params.push(expression);

            if self.peek(0).token_type == TokenType::RightParen {
                break;
            } else {
                self.assert_consume(TokenType::Comma);
            }
        }

        self.assert_consume(TokenType::RightParen);
        self.assert_consume(TokenType::SemiColon);

        AstNode::FunctionCall(name, params)
    }

    fn parse_block(&mut self) -> AstNode {
        self.scope.push(Scope::new());

        let mut children: Vec<AstNode> = vec![];

        self.consume();

        while self.peek(0).token_type != TokenType::RightBrace {
            let node = self.parse();
            children.push(node);
        }

        self.consume();

        self.scope.pop();

        AstNode::Block(children)
    }

    fn parse_if(&mut self) -> AstNode {
        self.assert_consume(TokenType::If);

        let expression = self.parse_expression(OperatorPrecedence::Zero);
        if expression.get_primitive_type() != PrimitiveType::Bool {
            panic!("If statement should contain a boolean expression");
        }

        let code = self.parse_block();

        let mut else_statement: Option<Box<AstNode>> = None;

        if self.peek(0).token_type == TokenType::Else {
            self.assert_consume(TokenType::Else);
            else_statement = Some(Box::new(self.parse_block()));
        }

        AstNode::If(Box::new(expression), Box::new(code), else_statement)
    }

    fn parse_while(&mut self) -> AstNode {
        self.assert_consume(TokenType::While);

        let expression = self.parse_expression(OperatorPrecedence::Zero);
        if expression.get_primitive_type() != PrimitiveType::Bool {
            panic!("While statement condition should be a boolean expression");
        }

        let code = self.parse_block();

        AstNode::While(Box::new(expression), Box::new(code))
    }

    pub fn parse(&mut self) -> AstNode {
        let next_token: &Token = self.peek(0);
        match next_token.token_type {
            TokenType::LeftBrace => self.parse_block(),
            TokenType::If => self.parse_if(),
            TokenType::While => self.parse_while(),
            TokenType::Var => self.parse_variable_declaration(),
            TokenType::Identifier => {
                let next_token_type = self.peek(1).token_type;
                match next_token_type {
                    TokenType::LeftParen => self.parse_functioncall(),
                    TokenType::EqualSign => self.parse_assignment(),
                    _ => panic!("Unexpected token {:?} after identifier", next_token_type),
                }
            }
            _ => panic!("Unexpected token: {:?}", next_token),
        }
    }
}
