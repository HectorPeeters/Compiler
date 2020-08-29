use crate::ast::*;
use crate::scope::*;
use crate::types::*;

#[derive(Debug, Copy, Clone)]
pub struct Register {
    pub size: i32,
    pub index: usize,
}

pub trait CodeGenerator {
    fn new(output_path: &str) -> Self;
    fn write(&mut self, data: &str);

    fn get_label(&mut self) -> i32;

    fn get_register(&mut self, size: i32) -> Register;
    fn free_register(&mut self, reg: Register);

    fn gen_assignment_instr(&mut self, variable: &Symbol, register: Register, size_index: usize);
    fn gen_comparison_instr(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        size_index: usize,
        comparison_type: &str,
    ) -> Register;
    fn gen_add_instr(&mut self, left_reg: Register, right_reg: Register, size_index: usize) -> Register;
    fn gen_subtract_instr(&mut self, left_reg: Register, right_reg: Register, size_index: usize) -> Register;
    fn gen_multiply_instr(&mut self, left_reg: Register, right_reg: Register, size_index: usize) -> Register;
    fn gen_divide_instr(&mut self, left_reg: Register, right_reg: Register, size_index: usize) -> Register;
    fn gen_numeric_literal_instr(&mut self, primitive_type: &PrimitiveType, primitive_value: &PrimitiveValue) -> Register;
    fn gen_widen_instr(&mut self, register: Register, primitive_type: &PrimitiveType, src_index: usize, dest_index: usize) -> Register;
    fn gen_identifier_instr(&mut self, symbol: &Symbol) -> Register;
    fn gen_functioncall_instr(&mut self, name: &String, params: &Vec<AstNode>);
    fn gen_if_instr(&mut self, condition: &AstNode, code: &AstNode, else_code: &Option<Box<AstNode>>);
    fn gen_while_instr(&mut self, condition: &AstNode, code: &AstNode);
    fn gen_function_instr(&mut self, symbol: &Symbol, code: &AstNode);
    
    fn do_post_check(&self) -> bool;
    
    fn error(&self, message: &str) {
        eprintln!("Generator error: {}", message);
        panic!();
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

    fn gen_block(&mut self, children: &[AstNode]) {
        for child in children {
            self.gen_node(child);
        }
    }

    fn gen_assignment(&mut self, variable: &Symbol, expression: &AstNode) {
        let reg = self.gen_expression(expression);

        let expression_type = expression.get_primitive_type();

        if !expression_type.is_compatible_with(&variable.primitive_type, true) {
            self.error(&format!(
                "Incompatible types in assignment, {:?} = {:?}",
                variable.primitive_type, expression_type
            ));
        }

        let index = Self::size_to_instruction_index(variable.primitive_type.get_size());
 
        self.gen_assignment_instr(&variable, reg, index);

        self.free_register(reg);
    }

    fn gen_comparison(
        &mut self,
        left_reg: Register,
        right_reg: Register,
        index: usize,
        comparison_type: &str,
    ) -> Register {
        self.gen_comparison_instr(left_reg, right_reg, index, comparison_type)
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
                        self.gen_add_instr(left_reg, right_reg, index)
                    }
                    BinaryOperationType::Subtract => {
                        self.gen_subtract_instr(left_reg, right_reg, index)
                    }
                    BinaryOperationType::Multiply => {
                        self.gen_multiply_instr(left_reg, right_reg, index)
                    }
                    BinaryOperationType::Divide => {
                        self.gen_divide_instr(left_reg, right_reg, index)
                    }
                    BinaryOperationType::Equals => {
                        self.gen_comparison_instr(left_reg, right_reg, index, "sete")
                    }
                    BinaryOperationType::NotEquals => {
                        self.gen_comparison_instr(left_reg, right_reg, index, "setne")
                    }
                    BinaryOperationType::LessThan => {
                        self.gen_comparison_instr(left_reg, right_reg, index, "setl")
                    }
                    BinaryOperationType::LessThanOrEqual => {
                        self.gen_comparison_instr(left_reg, right_reg, index, "setle")
                    }
                    BinaryOperationType::GreaterThan => {
                        self.gen_comparison_instr(left_reg, right_reg, index, "setg")
                    }
                    BinaryOperationType::GreaterThanOrEqual => {
                        self.gen_comparison_instr(left_reg, right_reg, index, "setge")
                    }
                }
            }
            AstNode::NumericLiteral(primitive_type, value) => {
                self.gen_numeric_literal_instr(primitive_type, value)
            }
            AstNode::Widen(primitive_type, node) => {
                let register = self.gen_expression(node);

                assert!(primitive_type.is_unsigned());

                let src_index =
                    Self::size_to_instruction_index(node.get_primitive_type().get_size());
                let dst_index = Self::size_to_instruction_index(primitive_type.get_size());

                self.gen_widen_instr(register, &primitive_type, src_index, dst_index)
            }
            AstNode::Identifier(symbol) => {
                self.gen_identifier_instr(symbol)
            }
            _ => {
                self.error(&format!("unsupported astnode in gen_expression"));
                unreachable!();
            }
        }
    }

    fn gen_node(&mut self, node: &AstNode) {
        match node {
            AstNode::Block(children) => self.gen_block(children),
            AstNode::VariableDeclaration(_) => {},
            AstNode::Assignment(var, expression) => self.gen_assignment(var, expression),
            AstNode::FunctionCall(name, params) => self.gen_functioncall_instr(name, params),
            AstNode::If(condition, code, else_code) => self.gen_if_instr(condition, code, else_code),
            AstNode::While(condition, code) => self.gen_while_instr(condition, code),
            AstNode::Function(symbol, code) => self.gen_function_instr(symbol, code),
            _ => {
                self.error("Trying to generate assembly for unsupported ast node!");
                unreachable!();
            }
        }
    }

    fn gen(&mut self, node: &AstNode) {
        self.write("\t.globl\tmain");
        self.write("\t.type\tmain, @function");

        self.gen_node(node);

        self.do_post_check();
    }
}
