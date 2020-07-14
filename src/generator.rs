use crate::ast::*;
use crate::scope::*;

use std::io::Write;

pub struct CodeGenerator<T: Write> {
    output: Box<T>,
    scope: Scope,
}

enum Register {
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R15,
}

impl<T: Write> CodeGenerator<T> {
    pub fn new(output: T) -> Self
    where
        T: Write + 'static,
    {
        CodeGenerator {
            output: Box::new(output),
            scope: Scope::new(),
        }
    }

    fn write(&mut self, data: &str) {
        (*self.output)
            .write_all(data.as_bytes())
            .expect("Failed to write to output file");
        (*self.output)
            .write_all(b"\n")
            .expect("Failed to write newline to output file");
    }

    fn gen_block(&mut self, children: &[AstNode]) {
        for child in children {
            self.gen_node(child);
        }
    }

    fn gen_declaration(&mut self, name: &str, primitive_type: &PrimitiveType) {
        if self.scope.get(name).is_some() {
            panic!("Redeclaration of variable {}", name);
        }

        self.scope.add(String::from(name), SymbolType::Variable);
        println!("{:?}", self.scope);
    }

    fn gen_assignment(&mut self, name: &str, expression: &AstNode) {}

//    fn gen_expression(&mut self) -> Register {}

    fn gen_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Block(children) => self.gen_block(children),
            AstNode::VariableDeclaration(name, primitive_type) => {
                self.gen_declaration(name, primitive_type)
            }
            AstNode::Assignment(name, expression) => self.gen_assignment(name, expression),
            _ => {}
        }
    }

    pub fn gen(&mut self, node: &AstNode) {
        self.write("\t.text");
        self.write("\t.globl\tmain");
        self.write("\t.type\tmain, @function");
        self.write("main:");
        self.write("\tpushq\t%rbp");
        self.write("\tmovq\t%rsp, %rbp");

//        self.gen_node(node);

        self.write("\tmovl\t$0, %eax");
        self.write("\tpopq\t%rbp");
        self.write("\tret");
    }
}
