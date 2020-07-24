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

//#[derive(Debug, Clone, Copy)]
//pub enum UnaryOperationType {
//    Negate,
//}

pub enum AstNode {
    BinaryOperation(BinaryOperationType, Box<AstNode>, Box<AstNode>),
    //  UnaryOperation(UnaryOperationType, Box<AstNode>),
    NumericLiteral(PrimitiveType, PrimitiveValue),
    VariableDeclaration(String, PrimitiveType),
    Assignment(String, Box<AstNode>),
    Widen(PrimitiveType, Box<AstNode>),
    //  Empty(),
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
            AstNode::NumericLiteral(primitive_type, value) => {
                println!(
                    "{}{:?}: {:?}",
                    " ".repeat(indentation),
                    primitive_type,
                    unsafe { value.int32 }
                );
            }
            AstNode::Block(children) => {
                println!("{}Block", " ".repeat(indentation));
                for child in children {
                    child.print(indentation + 1);
                }
            }
            AstNode::VariableDeclaration(name, primitive_type) => {
                println!("{}Var {}: {:?}", " ".repeat(indentation), name, primitive_type);
            }
            AstNode::Assignment(name, node) => {
                println!("{}{} =", " ".repeat(indentation), name);
                node.print(indentation + 1);
            }
            AstNode::Widen(primitive_type, node) => {
                println!("{}Widen {:?}", " ".repeat(indentation), primitive_type);
                node.print(indentation + 1);
            }
        }
    }

    pub fn get_primitive_type(&self) -> PrimitiveType {
        match self {
            AstNode::BinaryOperation(op_type, left, right) => {
                match op_type {
                    BinaryOperationType::Equals | BinaryOperationType::NotEquals | BinaryOperationType::LessThan | BinaryOperationType::LessThanOrEqual | BinaryOperationType::GreaterThan | BinaryOperationType::GreaterThanOrEqual => {
                        PrimitiveType::Bool
                    },
                    _ => {
                        let left_type = left.get_primitive_type();
                        let right_type = right.get_primitive_type();

                        if left_type.get_size() > right_type.get_size() {
                            left_type
                        } else {
                            right_type
                        }
                    }
                }
            },
            AstNode::NumericLiteral(primitive_type, _) => *primitive_type,
            _ => {
                PrimitiveType::Unknown
            }
        }
    }
}
