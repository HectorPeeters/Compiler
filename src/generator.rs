use crate::ast::*;
use crate::scope::*;
use crate::types::*;

use std::io::Write;

pub struct CodeGenerator<T: Write> {
    output: Box<T>,
    scope: Scope,
    registers: [Option<Register>; 4],
}

const REGISTERS: &[&[&str]] = &[
    &["%r8b", "%r9b", "%r10b", "%r11b"],
    &["%r8w", "%r9w", "%r10w", "%r11w"],
    &["%r8d", "%r9d", "%r10d", "%r11d"],
    &["%r8", "%r9", "%r10", "%r11"],
];

const PARAM_REGISTERS: &[&[&str]] = &[
    &["%dil", "%sil"],
    &["%di", "%si"],
    &["%edi", "%esi"],
    &["%rdi", "%rsi"],
];

const EAX: &[&str] = &["%al", "%ax", "%eax", "%rax"];

const MOV_INSTR: &[&str] = &["movb", "movw", "movl", "movq"];
const ADD_INSTR: &[&str] = &["addb", "addw", "addl", "addq"];
const SUB_INSTR: &[&str] = &["subb", "subw", "subl", "subq"];
const MUL_INSTR: &[&str] = &["mulb", "mulw", "mull", "mulq"];
const DIV_INSTR: &[&str] = &["divb", "divw", "divl", "divq"];
const CMP_INSTR: &[&str] = &["cmpb", "cmpw", "cmpl", "cmpq"];
const AND_INSTR: &[&str] = &["andb", "andw", "andl", "andq"];

#[derive(Debug, Copy, Clone)]
struct Register {
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
            registers: [None; 4],
        }
    }

    fn size_to_instruction_index(size: i32) -> usize {
        match size {
            8 => 0,
            16 => 1,
            32 => 2,
            64 => 3,
            _ => panic!("Trying to get instruction index for unknown primitive size!"),
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
                let register = Register { size, index: i };
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

        if !expression_type.is_compatible_with(&scope_var.primitive_type, true) {
            panic!(
                "Incompatible types in assignment, {:?} = {:?}",
                expression_type, scope_var.primitive_type
            );
        }

        let offset = scope_var.offset;

        let index = Self::size_to_instruction_index(scope_var.primitive_type.get_size());

        self.write(&format!("\tsubq\t${}, %rsp", offset));
        self.write(&format!(
            "\t{}\t{}, -{}(%rbp)",
            MOV_INSTR[index], REGISTERS[index][reg.index], offset
        ));

        self.free_register(reg);
    }

    fn gen_comparison(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        index: usize,
        comparison_type: &str,
    ) -> Register {
        self.write(&format!(
            "\t{}\t{}, {}",
            CMP_INSTR[index], REGISTERS[index][right_reg.index], REGISTERS[index][left_reg.index]
        ));
        self.write(&format!("\t{}\t{}", comparison_type, REGISTERS[0][right_reg.index]));
        self.write(&format!(
            "\t{}\t$255, {}",
            AND_INSTR[index], REGISTERS[index][right_reg.index]
        ));
        self.free_register(left_reg);
        right_reg
    }

    fn gen_expression(&mut self, expression: &AstNode) -> Register {
        match expression {
            AstNode::BinaryOperation(operation_type, left, right) => {
                assert!(
                    left.get_primitive_type().get_size() == right.get_primitive_type().get_size()
                );

                assert!(!left.get_primitive_type().is_signed());
                assert!(!right.get_primitive_type().is_signed());

                let left_reg = self.gen_expression(left);
                let right_reg = self.gen_expression(right);
                let index = Self::size_to_instruction_index(left.get_primitive_type().get_size());

                match operation_type {
                    BinaryOperationType::Add => {
                        self.write(&format!(
                            "\t{}\t{}, {}",
                            ADD_INSTR[index],
                            REGISTERS[index][right_reg.index],
                            REGISTERS[index][left_reg.index]
                        ));
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Subtract => {
                        self.write(&format!(
                            "\t{}\t{}, {}",
                            SUB_INSTR[index],
                            REGISTERS[index][right_reg.index],
                            REGISTERS[index][left_reg.index]
                        ));
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Multiply => {
                        self.write(&format!(
                            "\t{}\t{}, {}\n\t{}\t{}\n\t{}\t{}, {}",
                            MOV_INSTR[index],
                            REGISTERS[index][right_reg.index],
                            EAX[index],
                            MUL_INSTR[index],
                            REGISTERS[index][left_reg.index],
                            MOV_INSTR[index],
                            EAX[index],
                            REGISTERS[index][left_reg.index]
                        ));
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Divide => {
                        self.write(&format!(
                            "\t{}\t{}, {}",
                            MOV_INSTR[index], REGISTERS[index][left_reg.index], EAX[index]
                        ));
                        self.write("\tcltd");
                        self.write(&format!(
                            "\t{}\t{}",
                            DIV_INSTR[index], REGISTERS[index][right_reg.index]
                        ));
                        self.write(&format!(
                            "\t{}\t{}, {}",
                            MOV_INSTR[index], EAX[index], REGISTERS[index][left_reg.index]
                        ));
                        self.free_register(right_reg);

                        left_reg
                    }
                    BinaryOperationType::Equals => {
                        self.gen_comparison(left_reg, right_reg, index, "sete")
                    }
                    BinaryOperationType::NotEquals => {
                        self.gen_comparison(left_reg, right_reg, index, "setne")
                    }
                    _ => panic!("Trying to generate binary operation type which isn't supported!"),
                }
            }
            AstNode::NumericLiteral(primitive_type, value) => {
                let register = self.get_register(primitive_type.get_size());

                //TODO: fix hardcoded union access
                //TODO: fix hardcoded mov to 64bit reg
                self.write(&format!(
                    "\t{}\t${}, {}",
                    MOV_INSTR[3],
                    unsafe { value.int64 },
                    REGISTERS[3][register.index]
                ));

                register
            }
            AstNode::Widen(primitive_type, node) => {
                let register = self.gen_expression(node);

                assert!(primitive_type.is_unsigned());
                let result_reg = self.get_register(primitive_type.get_size());

                let src_index =
                    Self::size_to_instruction_index(node.get_primitive_type().get_size());
                let dst_index = Self::size_to_instruction_index(primitive_type.get_size());

                self.write(&format!(
                    "\tmovzx\t{}, {}",
                    REGISTERS[src_index][register.index], REGISTERS[dst_index][result_reg.index]
                ));

                self.free_register(register);

                result_reg
            }
            _ => panic!("unsupported astnode in gen_expression"),
        }
    }

    fn gen_functioncall(&mut self, name: &String, params: &Vec<String>) {
        let mut index: usize = 0;

        assert!(params.len() <= PARAM_REGISTERS.len());

        for param in params {
            let scope_var = self
                .scope
                .get(param)
                .expect("Unknown identifier in function call");
            let instr_index = Self::size_to_instruction_index(scope_var.primitive_type.get_size());

            //TODO: maybe make this movzx?
            let var_offset = scope_var.offset;
            self.write(&format!(
                "\t{}\t-{}(%rbp), {}",
                MOV_INSTR[instr_index], var_offset, PARAM_REGISTERS[instr_index][index]
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

        self.write("\tnop");
        self.write("\tleave");
        self.write("\tret");
    }
}
