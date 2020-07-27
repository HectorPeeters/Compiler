use crate::ast::*;
use crate::scope::*;
use crate::types::*;

use std::io::Write;

pub struct CodeGenerator<T: Write> {
    output: Box<T>,
    registers: [Option<Register>; 4],
    label_index: i32,
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
            registers: [None; 4],
            label_index: 0,
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

    fn get_label(&mut self) -> i32 {
        let result = self.label_index;
        self.label_index += 1;
        result
    }

    fn gen_assignment(&mut self, variable: &Symbol, expression: &AstNode) {
        let reg = self.gen_expression(expression);

        let expression_type = expression.get_primitive_type();

        if !expression_type.is_compatible_with(&variable.primitive_type, true) {
            panic!(
                "Incompatible types in assignment, {:?} = {:?}",
                variable.primitive_type, expression_type
            );
        }

        let index = Self::size_to_instruction_index(variable.primitive_type.get_size());

        //TODO: Move all subq calls to one big subq at the start of the scope
        self.write(&format!("\tsubq\t${}, %rsp", variable.offset));
        self.write(&format!(
            "\t{}\t{}, -{}(%rbp)",
            MOV_INSTR[index], REGISTERS[index][reg.index], variable.offset
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
        self.write(&format!(
            "\t{}\t{}",
            comparison_type, REGISTERS[0][right_reg.index]
        ));
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
                    BinaryOperationType::LessThan => {
                        self.gen_comparison(left_reg, right_reg, index, "setl")
                    }
                    BinaryOperationType::LessThanOrEqual => {
                        self.gen_comparison(left_reg, right_reg, index, "setle")
                    }
                    BinaryOperationType::GreaterThan => {
                        self.gen_comparison(left_reg, right_reg, index, "setg")
                    }
                    BinaryOperationType::GreaterThanOrEqual => {
                        self.gen_comparison(left_reg, right_reg, index, "setge")
                    }
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
            AstNode::Identifier(symbol) => {
                let size = symbol.primitive_type.get_size();
                let register = self.get_register(size);

                let index = Self::size_to_instruction_index(size);

                self.write(&format!(
                    "\t{}\t-{}(%rbp), {}",
                    MOV_INSTR[index], symbol.offset, REGISTERS[index][register.index],
                ));

                register
            }
            _ => panic!("unsupported astnode in gen_expression"),
        }
    }

    fn gen_functioncall(&mut self, name: &String, params: &Vec<AstNode>) {
        let mut index: usize = 0;

        assert!(params.len() <= PARAM_REGISTERS.len());

        let mut allocated_regs: Vec<Register> = Vec::new();

        for param in params {
            let instr_index =
                Self::size_to_instruction_index(param.get_primitive_type().get_size());
            let expression_reg = self.gen_expression(param);

            //TODO: fix this
            self.write(&format!(
                "\txor\t\t{},{}",
                PARAM_REGISTERS[3][index], PARAM_REGISTERS[3][index]
            ));
            self.write(&format!(
                "\t{}\t{}, {}",
                MOV_INSTR[instr_index],
                REGISTERS[instr_index][expression_reg.index],
                PARAM_REGISTERS[instr_index][index]
            ));

            allocated_regs.push(expression_reg);

            index += 1;
        }

        for reg in allocated_regs {
            self.free_register(reg);
        }

        self.write(&format!("\tcall\t{}", name));
    }

    fn gen_if(&mut self, condition: &AstNode, code: &AstNode, else_code: &Option<Box<AstNode>>) {
        let has_else = else_code.is_some();

        let condition_reg = self.gen_expression(&condition);

        let else_label = self.get_label();
        let end_label = self.get_label();

        let instr_index = Self::size_to_instruction_index(condition_reg.size);

        self.write(&format!(
            "\t{}\t$0, {}",
            CMP_INSTR[instr_index], REGISTERS[instr_index][condition_reg.index]
        ));
        self.write(&format!(
            "\tjz\t\tL{}",
            if has_else { else_label } else { end_label }
        ));
        self.gen_node(code);
        self.write(&format!("\tjmp L{}", end_label));
        if has_else {
            self.write(&format!("L{}:", else_label));
            if let Some(else_code) = else_code {
                self.gen_node(else_code);
            }
        }
        self.write(&format!("L{}:", end_label));

        self.free_register(condition_reg);
    }

    fn gen_while(&mut self, condition: &AstNode, code: &AstNode) {
        let start_label = self.get_label();
        let end_label = self.get_label();

        self.write(&format!("L{}:", start_label));

        let condition_reg = self.gen_expression(condition);

        let instr_index = Self::size_to_instruction_index(condition_reg.size);

        self.write(&format!(
            "\t{}\t$0, {}",
            CMP_INSTR[instr_index], REGISTERS[instr_index][condition_reg.index]
        ));
        self.write(&format!(
            "\tjz\t\tL{}", end_label
        ));
        self.gen_node(code);

        self.write(&format!("\tjmp\t\tL{}", start_label));
        self.write(&format!("L{}:", end_label));

        self.free_register(condition_reg);
    }

    fn gen_function(&mut self, symbol: &Symbol, code: &AstNode) {
        assert!(symbol.symbol_type == SymbolType::Function);

        self.write(&format!("{}:", symbol.name));
        self.write("\tpush\t%rbp");
        self.write("\tmov\t\t%rsp, %rbp");
        self.gen_node(code);
        self.write("\tmov\t\t%rbp, %rsp");
        self.write("\tpop\t\t%rbp");

        assert!(symbol.primitive_type == PrimitiveType::Void);
        self.write("\tret");
    }

    fn gen_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Block(children) => self.gen_block(children),
            AstNode::VariableDeclaration(_) => {},
            AstNode::Assignment(var, expression) => self.gen_assignment(var, expression),
            AstNode::FunctionCall(name, params) => self.gen_functioncall(name, params),
            AstNode::If(condition, code, else_code) => self.gen_if(condition, code, else_code),
            AstNode::While(condition, code) => self.gen_while(condition, code),
            AstNode::Function(symbol, code) => self.gen_function(symbol, code),
            _ => panic!("Trying to generate assembly for unsupported ast node!"),
        }
    }

    pub fn gen(&mut self, node: &AstNode) {
        self.write("\t.globl\tmain");
        self.write("\t.type\tmain, @function");

        self.gen_node(node);

        for i in 0..self.registers.len() {
            if self.registers[i].is_some() {
                panic!("Not all registers were freed!");
            }
        }
    }
}
