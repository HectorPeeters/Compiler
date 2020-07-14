mod ast;
use ast::*;
mod lexer;
use lexer::*;
mod parser;
use parser::*;
mod generator;
use generator::*;
mod scope;

use std::fs::File;

#[allow(dead_code)]
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
    let tokens = Lexer::new("var x;\nx = 12 - 2;\nx = 12 + 2;").tokenize();

    println!("===== Tokens =====");
    for token in &tokens {
        println!("{:?}", token);
    }

    println!("\n===== AST =====");
    let result_node = Parser::new(tokens).parse();
    result_node.print(0);

    println!("\n===== Code Generation =====");
    let mut generator =
        CodeGenerator::new(File::create("output.s").expect("Failed to open output file"));
    generator.gen(&result_node);
}
