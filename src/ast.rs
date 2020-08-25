use crate::scope::*;
use crate::types::*;

#[derive(Debug, Clone, Copy)]
pub enum BinaryOperationType {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperationType {
    Negate,
}

pub enum AstNode {
    BinaryOperation(BinaryOperationType, Box<AstNode>, Box<AstNode>),
    UnaryOperation(UnaryOperationType, Box<AstNode>),
    NumericLiteral(PrimitiveType, PrimitiveValue),
    VariableDeclaration(Symbol),
    Assignment(Symbol, Box<AstNode>),
    FunctionCall(String, Vec<AstNode>),
    Widen(PrimitiveType, Box<AstNode>),
    Identifier(Symbol),
    Function(Symbol, Box<AstNode>),
    If(Box<AstNode>, Box<AstNode>, Option<Box<AstNode>>),
    While(Box<AstNode>, Box<AstNode>),
    Block(Vec<AstNode>),
}

impl AstNode {
    pub fn print(&self, indentation: usize) {
        match self {
            AstNode::BinaryOperation(op_type, left, right) => {
                println!("{}{:?}", " ".repeat(indentation), op_type);
                left.print(indentation + 2);
                right.print(indentation + 2);
            }
            AstNode::UnaryOperation(op_type, node) => {
                println!("{}{:?}", " ".repeat(indentation), op_type);
                node.print(indentation + 2);
            }
            AstNode::NumericLiteral(primitive_type, value) => {
                println!(
                    "{}{:?}: {:?}",
                    " ".repeat(indentation),
                    primitive_type,
                    unsafe { value.uint32 }
                );
            }
            AstNode::Block(children) => {
                println!("{}Block", " ".repeat(indentation));
                for child in children {
                    child.print(indentation + 2);
                }
            }
            AstNode::VariableDeclaration(var) => {
                println!(
                    "{}Var {}: {:?}",
                    " ".repeat(indentation),
                    var.name,
                    var.primitive_type
                );
            }
            AstNode::Assignment(var, node) => {
                println!("{}{} =", " ".repeat(indentation), var.name);
                node.print(indentation + 2);
            }
            AstNode::FunctionCall(name, params) => {
                println!("{}{}(", " ".repeat(indentation), name);
                for param in params {
                    param.print(indentation + 2);
                }
                println!("{})", " ".repeat(indentation));
            }
            AstNode::Widen(primitive_type, node) => {
                println!("{}Widen {:?}", " ".repeat(indentation), primitive_type);
                node.print(indentation + 2);
            }
            AstNode::Identifier(var) => {
                println!("{}{}", " ".repeat(indentation), var.name);
            }
            AstNode::If(condition, code, else_code) => {
                println!("{}If (", " ".repeat(indentation));
                condition.print(indentation + 2);
                println!("{}){{", " ".repeat(indentation));
                code.print(indentation + 2);
                if let Some(else_code) = else_code {
                    println!("{}}} else {{", " ".repeat(indentation));
                    else_code.print(indentation + 2);
                }
                println!("{}}}", " ".repeat(indentation));
            }
            AstNode::While(condition, code) => {
                println!("{}While (", " ".repeat(indentation));
                condition.print(indentation + 2);
                println!("{}){{", " ".repeat(indentation));
                code.print(indentation + 2);
                println!("{}}}", " ".repeat(indentation));
            }
            AstNode::Function(symbol, code) => {
                println!("{}Fn {}", " ".repeat(indentation), symbol.name);
                code.print(indentation + 2);
            }
        }
    }

    pub fn get_primitive_type(&self) -> PrimitiveType {
        match self {
            AstNode::BinaryOperation(op_type, left, right) => match op_type {
                BinaryOperationType::Equals
                | BinaryOperationType::NotEquals
                | BinaryOperationType::LessThan
                | BinaryOperationType::LessThanOrEqual
                | BinaryOperationType::GreaterThan
                | BinaryOperationType::GreaterThanOrEqual => PrimitiveType::Bool,
                _ => {
                    let left_type = left.get_primitive_type();
                    let right_type = right.get_primitive_type();

                    if left_type.get_size() > right_type.get_size() {
                        left_type
                    } else {
                        right_type
                    }
                }
            },
            AstNode::NumericLiteral(primitive_type, _) => *primitive_type,
            AstNode::Widen(primitive_type, _) => *primitive_type,
            AstNode::Identifier(symbol) => symbol.primitive_type,
            AstNode::UnaryOperation(op_type, node) => {
                match op_type {
                    UnaryOperationType::Negate => 
                    node.get_primitive_type().switch_sign()
                }
            }
            _ => {
                println!("WARNING: get_primitive_type called for unknown AstNode type!");
                PrimitiveType::Unknown
            }
        }
    }
}
