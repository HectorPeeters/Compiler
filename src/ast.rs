#[derive(Debug, Clone, Copy)]
pub enum BinaryOperationType {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOperationType {
    Negate,
}

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveType {
    Int8,
    Int16,
    Int32,
    Int64,
    F32,
    F64,
}

pub union PrimitiveValue {
    pub uint8: u8,
    pub uint16: u16,
    pub uint32: u32,
    pub uint64: u64,

    pub int8: i8,
    pub int16: i16,
    pub int32: i32,
    pub int64: i64,

    pub float32: f32,
    pub float64: f64,
}

pub enum AstNode {
    BinaryOperation(BinaryOperationType, Box<AstNode>, Box<AstNode>),
    UnaryOperation(UnaryOperationType, Box<AstNode>),
    NumericLiteral(PrimitiveType, PrimitiveValue),
    Empty(),
}