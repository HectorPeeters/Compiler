use crate::ast::*;
use crate::scope::*;

use std::io::Write;

pub struct CodeGenerator<T: Write> {
    output: Box<T>,
    scope: Scope,
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

    fn gen_block(&mut self, children: &Vec<AstNode>) {
        for child in children {
            self.gen_node(child);
        }
    }

    fn gen_declaration(&mut self, name: &String, primitive_type: &PrimitiveType) {
        if let Some(var) = self.scope.get(name) {
            panic!("Redeclaration of variable {}", name);
        }

        self.scope.add(name, SymbolType::Variable);
        println!("{:?}", self.scope);
    }

    fn gen_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Block(children) => self.gen_block(children),
            AstNode::VariableDeclaration(name, primitive_type) => {
                self.gen_declaration(name, primitive_type)
            }
            _ => {}
        }
    }

    pub fn gen(&mut self, node: &AstNode) {
        self.write("\t.text");
        self.write("\t.globl\tmain");
        self.write("\t.type\tmain, @function");
        self.write("main:");
        self.write("\tpush\t%rbp");
        self.write("\tmov\t%rbp, %rsp");

        self.gen_node(node);

        self.write("\tmov\t%eax, 0");
        self.write("\tpop\t%rbp");
        self.write("\tret");
    }
}
