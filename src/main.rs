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
        AstNode::Block(children) => {
            println!("{}Block", " ".repeat(indentation));
            for child in children {
                print_node(child, indentation + 1);
            }
        }
        AstNode::VariableDeclaration(name, _primitive_type) => {
            println!("{}Var {}", " ".repeat(indentation), name);
        }
        _ => {}
    }
}

fn eval(node: &AstNode) -> i64 {
    match node {
        AstNode::BinaryOperation(op_type, left, right) => match op_type {
            BinaryOperationType::Add => eval(left) + eval(right),
            BinaryOperationType::Subtract => eval(left) - eval(right),
            BinaryOperationType::Multiply => eval(left) * eval(right),
            BinaryOperationType::Divide => eval(left) / eval(right),
        },
        AstNode::NumericLiteral(_primitive_type, value) => unsafe { value.int64 },
        _ => panic!("Trying to eval node which isn't supported!"),
    }
}

fn main() {
    let tokens = Lexer::new("var x;").tokenize();
    println!("===== Tokens =====");
    for token in &tokens {
        println!("{:?}", token);
    }

    println!("\n===== AST =====");
    let result_node = Parser::new(tokens).parse();
    print_node(&result_node, 0);
    //    println!("\n\n= {}", eval(&result_node));
}
