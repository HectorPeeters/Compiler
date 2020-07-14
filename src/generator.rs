use crate::ast::*;
use crate::scope::*;

use std::io::Write;

pub struct CodeGenerator<T: Write> {
    output: Box<T>,
    scope: Scope,
    registers: [bool; 7],
}

const REGISTERS: &[&str] = &["%r10d", "%r11d", "%r12d", "%r13d", "%r14d", "%r15d"];

type Register = usize;

impl<T: Write> CodeGenerator<T> {
    pub fn new(output: T) -> Self
    where
        T: Write + 'static,
    {
        CodeGenerator {
            output: Box::new(output),
            scope: Scope::new(),
            registers: [false; 7],
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

    fn get_register(&mut self) -> Register {
        for i in 0..self.registers.len() {
            if !self.registers[i] {
                self.registers[i] = true;
                return i;
            }
        }

        panic!("Out of registers!");
    }

    fn free_register(&mut self, reg: Register) {
        if !self.registers[reg] {
            panic!("Trying to free a register which is already free!");
        }
        self.registers[reg] = false;
    }

    fn gen_block(&mut self, children: &[AstNode]) {
        for child in children {
            self.gen_node(child);
        }
    }

    fn gen_declaration(&mut self, name: &str, _primitive_type: &PrimitiveType) {
        if self.scope.get(name).is_some() {
            panic!("Redeclaration of variable {}", name);
        }

        self.scope.add(
            String::from(name),
            SymbolType::Variable,
            self.scope.last_offset + 4,
        );
        println!("{:?}", self.scope);
    }

    fn gen_assignment(&mut self, name: &str, expression: &AstNode) {
        let reg = self.gen_expression(expression);

        let offset = self
            .scope
            .get(name)
            .unwrap_or_else(|| panic!("Unexpected identifier in assignment: {}", name))
            .offset;

        self.write(format!("\tmov\t{}, -{}(%rbp)", REGISTERS[reg], offset).as_str());
    }

    fn gen_expression(&mut self, expression: &AstNode) -> Register {
        match expression {
            AstNode::BinaryOperation(operation_type, left, right) => {
                let left_reg = self.gen_expression(left);
                let right_reg = self.gen_expression(right);

                match operation_type {
                    BinaryOperationType::Add => {
                        self.write(
                            format!("\tadd\t{}, {}", REGISTERS[right_reg], REGISTERS[left_reg])
                                .as_str(),
                        );
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Subtract => {
                        self.write(
                            format!("\tsub\t{}, {}", REGISTERS[right_reg], REGISTERS[left_reg])
                                .as_str(),
                        );
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Multiply => {
                        self.write(
                            format!("\timul\t{}, {}", REGISTERS[right_reg], REGISTERS[left_reg])
                                .as_str(),
                        );
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Divide => {
                        self.write(format!("\tmov\t{}, %eax", REGISTERS[left_reg]).as_str());
                        self.write("\tcltd");
                        self.write(format!("\tidiv\t{}", REGISTERS[right_reg]).as_str());
                        self.write(format!("\tmov\t%eax, {}", REGISTERS[left_reg]).as_str());
                        self.free_register(right_reg);

                        left_reg
                    }
                }
            }
            AstNode::NumericLiteral(primitive_type, value) => match primitive_type {
                PrimitiveType::Int32 => {
                    let register = self.get_register();

                    self.write(
                        format!(
                            "\tmov\t${}, {}",
                            unsafe { value.int32 },
                            REGISTERS[register]
                        )
                        .as_str(),
                    );

                    register
                }
            //  _ => panic!(
            //      "gen_expression does not support {:?} NumericLiteral",
            //      primitive_type
            //  ),
            },
            _ => panic!("unsupported astnode in gen_expression"),
        }
    }

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
        self.write(".LC0:");
        self.write("\t.string \"%d\\n\"");
        self.write("\t.text");
        self.write("\t.globl\tmain");
        self.write("\t.type\tmain, @function");
        self.write("main:");
        self.write("\tpush\t%rbp");
        self.write("\tmov\t%rsp, %rbp");

        self.gen_node(node);

        let offset = self.scope.get("x").unwrap().offset;
        self.write(format!("\tmov\t-{}(%rbp), %eax", offset).as_str());
        self.write("\tmov\t%eax, %esi");
        self.write("\tleaq\t.LC0(%rip), %rdi");
        self.write("\tmov\t$0, %eax");
        self.write("\tcall\tprintf@PLT");

        self.write("\tmov\t$0, %eax");
        self.write("\tpop\t%rbp");
        self.write("\tret");
    }
}
