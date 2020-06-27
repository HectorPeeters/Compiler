mod ast;
use ast::*;
mod lexer;
use lexer::*;
mod parser;
use parser::*;

fn print_node(node: &AstNode, indentation: usize) {
    match node {
        AstNode::BinaryOperation(op_type, left, right) => {
            println!("{}{:?}", " ".repeat(indentation), op_type);
            print_node(left, indentation + 2);
            print_node(right, indentation + 2);
        }
        AstNode::NumericLiteral(primitive_type, value) => {
            println!(
                "{}{:?}: {:?}",
                " ".repeat(indentation),
                primitive_type,
                unsafe { value.int64 }
            );
        }
        _ => {}
    }
}

fn eval(node: &AstNode) -> i64 {
    match node {
        AstNode::BinaryOperation(op_type, left, right) => match op_type {
            BinaryOperationType::Add => return eval(left) + eval(right),
            BinaryOperationType::Subtract => return eval(left) - eval(right),
            BinaryOperationType::Multiply => return eval(left) * eval(right),
            BinaryOperationType::Divide => return eval(left) / eval(right),
        },
        AstNode::NumericLiteral(primitive_type, value) => {
            return unsafe { value.int64 };
        }
        _ => panic!("Trying to eval node which isn't supported!"),
    }
}

fn main() {
    let tokens = Lexer::new("6 * 5 + 4 * 3 + 2 * 1 + 1 * 2 + 3 * 4 + 5 * 6 + 7 * 8").tokenize();
    println!("===== Tokens =====");
    for token in &tokens {
        println!("{:?}", token);
    }

    println!("\n===== AST =====");
    let result_node = Parser::new(tokens).parse_expression(OperatorPrecedence::None);
    print_node(&result_node, 0);
    println!("\n\n= {}", eval(&result_node));
}
