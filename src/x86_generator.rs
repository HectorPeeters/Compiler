use crate::ast::*;
use crate::generator::*;
use crate::scope::*;
use crate::types::*;

use std::fs::File;
use std::io::Write;

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

pub struct X86CodeGenerator {
    output: Box<File>,
    registers: [Option<Register>; 4],
    label_index: i32,
}

impl CodeGenerator for X86CodeGenerator {
    fn new(output_path: &str) -> Self {
        X86CodeGenerator {
            output: Box::new(File::create(output_path).expect("Failed to create output file")),
            registers: [None; 4],
            label_index: 0,
        }
    }

    fn write(&mut self, data: &str) {
        self.output
            .write_all(data.as_bytes())
            .expect("Failed to write to output file");
        self.output
            .write_all(b"\n")
            .expect("Failed to write newline to output file");
        println!("{}", data);
    }

    fn get_label(&mut self) -> i32 {
        let result = self.label_index;
        self.label_index += 1;
        result
    }

    fn get_register(&mut self, size: i32) -> Register {
        for i in 0..self.registers.len() {
            if self.registers[i].is_none() {
                let register = Register { size, index: i };
                self.registers[i] = Some(register);
                return register;
            }
        }

        self.error("Out of registers!");
        unreachable!();
    }

    fn free_register(&mut self, reg: Register) {
        if self.registers[reg.index].is_none() {
            self.error("Trying to free a register which is already freed!");
        }
        self.registers[reg.index] = None;
    }

    fn gen_assignment_instr(&mut self, symbol: &Symbol, register: Register, size_index: usize) {
        self.write(&format!("\tsubq\t${}, %rsp", symbol.offset));
        self.write(&format!(
            "\t{}\t{}, -{}(%rbp)",
            MOV_INSTR[size_index], REGISTERS[size_index][register.index], symbol.offset
        ));
    }

    fn gen_comparison_instr(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        size_index: usize,
        comparison_type: &str,
    ) -> Register {
        self.write(&format!(
            "\t{}\t{}, {}",
            CMP_INSTR[size_index],
            REGISTERS[size_index][right_reg.index],
            REGISTERS[size_index][left_reg.index]
        ));
        self.write(&format!(
            "\t{}\t{}",
            comparison_type, REGISTERS[0][right_reg.index]
        ));
        self.write(&format!(
            "\t{}\t$255, {}",
            AND_INSTR[size_index], REGISTERS[size_index][right_reg.index]
        ));

        self.free_register(left_reg);
        right_reg
    }

    fn gen_add_instr(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        size_index: usize,
    ) -> Register {
        self.write(&format!(
            "\t{}\t{}, {}",
            ADD_INSTR[size_index],
            REGISTERS[size_index][right_reg.index],
            REGISTERS[size_index][left_reg.index]
        ));

        self.free_register(right_reg);
        left_reg
    }

    fn gen_subtract_instr(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        size_index: usize,
    ) -> Register {
        self.write(&format!(
            "\t{}\t{}, {}",
            SUB_INSTR[size_index],
            REGISTERS[size_index][right_reg.index],
            REGISTERS[size_index][left_reg.index]
        ));

        self.free_register(right_reg);
        left_reg
    }

    fn gen_multiply_instr(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        size_index: usize,
    ) -> Register {
        self.write(&format!(
            "\t{}\t{}, {}\n\t{}\t{}\n\t{}\t{}, {}",
            MOV_INSTR[size_index],
            REGISTERS[size_index][right_reg.index],
            EAX[size_index],
            MUL_INSTR[size_index],
            REGISTERS[size_index][left_reg.index],
            MOV_INSTR[size_index],
            EAX[size_index],
            REGISTERS[size_index][left_reg.index]
        ));

        self.free_register(right_reg);
        left_reg
    }

    fn gen_divide_instr(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        size_index: usize,
    ) -> Register {
        self.write(&format!(
            "\t{}\t{}, {}",
            MOV_INSTR[size_index], REGISTERS[size_index][left_reg.index], EAX[size_index]
        ));
        self.write("\tcltd");
        self.write(&format!(
            "\t{}\t{}",
            DIV_INSTR[size_index], REGISTERS[size_index][right_reg.index]
        ));
        self.write(&format!(
            "\t{}\t{}, {}",
            MOV_INSTR[size_index], EAX[size_index], REGISTERS[size_index][left_reg.index]
        ));

        self.free_register(right_reg);
        left_reg
    }

    fn gen_numeric_literal_instr(
        &mut self,
        primitive_type: &PrimitiveType,
        primitive_value: &PrimitiveValue,
    ) -> Register {
        let register = self.get_register(primitive_type.get_size());

        //TODO: fix hardcoded union access
        //TODO: fix hardcoded mov to 64bit reg
        self.write(&format!(
            "\t{}\t${}, {}",
            MOV_INSTR[3],
            unsafe { primitive_value.int64 },
            REGISTERS[3][register.index]
        ));

        register
    }

    fn gen_widen_instr(
        &mut self,
        register: Register,
        primitive_type: &PrimitiveType,
        src_index: usize,
        dest_index: usize,
    ) -> Register {
        let result_reg = self.get_register(primitive_type.get_size());

        self.write(&format!(
            "\tmovzx\t{}, {}",
            REGISTERS[src_index][register.index], REGISTERS[dest_index][result_reg.index]
        ));

        self.free_register(register);

        result_reg
    }

    fn gen_identifier_instr(&mut self, symbol: &Symbol) -> Register {
        let size = symbol.primitive_type.get_size();
        let register = self.get_register(size);
        let index = Self::size_to_instruction_index(size);

        match symbol.symbol_type {
            SymbolType::Variable => {
                self.write(&format!(
                    "\t{}\t-{}(%rbp), {}",
                    MOV_INSTR[index], symbol.offset, REGISTERS[index][register.index],
                ));
            }
            SymbolType::FunctionParameter => {
                self.write(&format!(
                    "\t{}\t{}, {}",
                    MOV_INSTR[index],
                    PARAM_REGISTERS[index][symbol.offset as usize],
                    REGISTERS[index][register.index],
                ));
            }
            _ => {
                self.error("Trying to generate from function symbol ast node");
            }
        }

        register
    }

    fn gen_functioncall_instr(&mut self, name: &str, params: &[AstNode]) {
        assert!(params.len() <= PARAM_REGISTERS.len());

        let mut allocated_regs: Vec<Register> = Vec::new();

        for (index, param) in params.iter().enumerate() {
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
        }

        for reg in allocated_regs {
            self.free_register(reg);
        }

        self.write(&format!("\tcall\t{}", name));
    }

    fn gen_if_instr(
        &mut self,
        condition: &AstNode,
        code: &AstNode,
        else_code: &Option<Box<AstNode>>,
    ) {
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

    fn gen_while_instr(&mut self, condition: &AstNode, code: &AstNode) {
        let start_label = self.get_label();
        let end_label = self.get_label();

        self.write(&format!("L{}:", start_label));

        let condition_reg = self.gen_expression(condition);

        let instr_index = Self::size_to_instruction_index(condition_reg.size);

        self.write(&format!(
            "\t{}\t$0, {}",
            CMP_INSTR[instr_index], REGISTERS[instr_index][condition_reg.index]
        ));
        self.write(&format!("\tjz\t\tL{}", end_label));
        self.gen_node(code);

        self.write(&format!("\tjmp\t\tL{}", start_label));
        self.write(&format!("L{}:", end_label));

        self.free_register(condition_reg);
    }

    fn gen_function_instr(&mut self, symbol: &Symbol, code: &AstNode) {
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

    fn do_post_check(&self) -> bool {
        for i in 0..self.registers.len() {
            if self.registers[i].is_some() {
                self.error("Not all registers were freed!");
                return false;
            }
        }
        true
    }
}
