mod ast;
mod lexer;
use lexer::*;
mod parser;
use parser::*;
mod generator;
use generator::*;
mod scope;
mod types;
mod x86_generator;
use x86_generator::*;

use clap::{App, Arg};

fn main() {
    let matches = App::new("Compiler")
        .version("0.0.1")
        .author("Hector Peeters")
        .arg(
            Arg::with_name("INPUT")
                .help("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input_file = matches.value_of("INPUT").unwrap();
    let input = std::fs::read_to_string(input_file).expect("Failed to read input file!");

    let tokens = Lexer::new(&input).tokenize();

    println!("===== Tokens =====");
    for token in &tokens {
        println!("{:?}", token);
    }

    println!("\n===== AST =====");
    let result_node = Parser::new(tokens).parse();
    result_node.print(0);

    println!("\n===== Code Generation =====");
    let mut generator = X86CodeGenerator::new("output.s");
    generator.gen(&result_node);
}
