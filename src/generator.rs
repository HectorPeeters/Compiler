use crate::ast::*;
use crate::scope::*;
use crate::types::*;

use std::io::Write;

pub struct CodeGenerator<T: Write> {
    output: Box<T>,
    scope: Scope,
    registers: [Option<Register>; 7],
}

const QREGISTERS: &[&str] = &["%r10", "%r11", "%r12", "%r13", "%r14", "%r15"];
const DREGISTERS: &[&str] = &["%r10d", "%r11d", "%r12d", "%r13d", "%r14d", "%r15d"];
const WREGISTERS: &[&str] = &["%r10w", "%r11w", "%r12w", "%r13w", "%r14w", "%r15w"];
const BREGISTERS: &[&str] = &["%r10b", "%r11b", "%r12b", "%r13b", "%r14b", "%r15b"];

#[derive(Debug, Copy, Clone)]
struct Register{
    pub size: i32,
    pub index: usize,
}

impl<T: Write> CodeGenerator<T> {
    pub fn new(output: T) -> Self
    where
        T: Write + 'static,
    {
        CodeGenerator {
            output: Box::new(output),
            scope: Scope::new(),
            registers: [None; 7],
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

    fn get_register(&mut self, size: i32) -> Register {
        for i in 0..self.registers.len() {
            if !self.registers[i].is_some() {
                let register = Register { size, index: i};
                self.registers[i] = Some(register);
                return register;
            }
        }

        panic!("Out of registers!");
    }

    fn free_register(&mut self, reg: Register) {
        if !self.registers[reg.index].is_some() {
            panic!("Trying to free a register which is already free!");
        }
        self.registers[reg.index] = None;
    }

    fn gen_block(&mut self, children: &[AstNode]) {
        for child in children {
            self.gen_node(child);
        }
    }

    fn gen_declaration(&mut self, name: &str, primitive_type: PrimitiveType) {
        if self.scope.get(name).is_some() {
            panic!("Redeclaration of variable {}", name);
        }

        self.scope
            .add(String::from(name), SymbolType::Variable, primitive_type);
        println!("{:?}", self.scope);
    }

    fn gen_assignment(&mut self, name: &str, expression: &AstNode) {
        let reg = self.gen_expression(expression);

        let scope_var: &Symbol = self
            .scope
            .get(name)
            .unwrap_or_else(|| panic!("Unexpected identifier in assignment: {}", name));

        let expression_type = expression.get_primitive_type();

        println!(
            "Checking compatibility of {:?} = {:?}",
            expression_type, scope_var.primitive_type
        );
        if !expression_type.is_compatible_with(&scope_var.primitive_type) {
            panic!(
                "Incompatible types in assignment, {:?} = {:?}",
                expression_type, scope_var.primitive_type
            );
        }

        let offset = scope_var.offset;

        self.write(format!("\tmov\t\t{}, -{}(%rbp)", QREGISTERS[reg.index], offset).as_str());
    }

    fn gen_expression(&mut self, expression: &AstNode) -> Register {
        match expression {
            AstNode::BinaryOperation(operation_type, left, right) => {
                let left_reg = self.gen_expression(left);
                let right_reg = self.gen_expression(right);

                match operation_type {
                    BinaryOperationType::Add => {
                        self.write(
                            format!("\tadd\t\t{}, {}", QREGISTERS[right_reg.index], QREGISTERS[left_reg.index])
                                .as_str(),
                        );
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Subtract => {
                        self.write(
                            format!("\tsub\t\t{}, {}", QREGISTERS[right_reg.index], QREGISTERS[left_reg.index])
                                .as_str(),
                        );
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Multiply => {
                        self.write(
                            format!("\timul\t{}, {}", QREGISTERS[right_reg.index], QREGISTERS[left_reg.index])
                                .as_str(),
                        );
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Divide => {
                        self.write(format!("\tmov\t\t{}, %rax", QREGISTERS[left_reg.index]).as_str());
                        self.write("\tcltd");
                        self.write(format!("\tidiv\t{}", QREGISTERS[right_reg.index]).as_str());
                        self.write(format!("\tmov\t\t%rax, {}", QREGISTERS[left_reg.index]).as_str());
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Equals => {
                        self.write(
                            format!("\tcmpq\t{}, {}", QREGISTERS[right_reg.index], QREGISTERS[left_reg.index])
                                .as_str(),
                        );
                        self.write(format!("\tsete\t{}", BREGISTERS[right_reg.index]).as_str());
                        self.write(format!("\tandq\t$255, {}", QREGISTERS[right_reg.index]).as_str());
                        self.free_register(left_reg);
                        right_reg
                    }
                    _ => panic!("Trying to generate binary operation type which isn't supported!"),
                }
            }
            AstNode::NumericLiteral(primitive_type, value) => {
                // PrimitiveType::Int32 => {
                let register = self.get_register(primitive_type.get_size());

                self.write(
                    format!(
                        "\tmov\t\t${}, {}",
                        unsafe { value.int32 },
                        QREGISTERS[register.index]
                    )
                    .as_str(),
                );

                register
                // }
                //  _ => panic!(
                //   "gen_expression does not support {:?} NumericLiteral",
                //   primitive_type
                //   ),
            }
            _ => panic!("unsupported astnode in gen_expression"),
        }
    }

    fn gen_functioncall(&mut self, name: &String, params: &Vec<String>) {
        const PARAM_REGISTERS: &[&str] = &["%edi", "%esi"];

        let mut index: usize = 0;

        for param in params {
            self.write(&format!(
                "\tmov\t\t-{}(%rbp), {}",
                self.scope
                    .get(param)
                    .expect("Unknown identifier in function call")
                    .offset,
                PARAM_REGISTERS[index]
            ));
            index += 1;
        }

        self.write(&format!("\tcall\t{}", name));
    }

    fn gen_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Block(children) => self.gen_block(children),
            AstNode::VariableDeclaration(name, primitive_type) => {
                self.gen_declaration(name, *primitive_type)
            }
            AstNode::Assignment(name, expression) => self.gen_assignment(name, expression),
            AstNode::FunctionCall(name, params) => self.gen_functioncall(name, params),
            _ => panic!("Trying to generate assembly for unsupported ast node!"),
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
        self.write("\tmov\t\t%rsp, %rbp");

        self.gen_node(node);

        self.write("\tmov\t\t$0, %eax");
        self.write("\tpop\t\t%rbp");
        self.write("\tret");
    }
}
