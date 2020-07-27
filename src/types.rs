use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PrimitiveType {
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Bool,
    Unknown,
    Void,
}

impl PrimitiveType {
    pub fn get_size(&self) -> i32 {
        match self {
            PrimitiveType::Int8 => 8,
            PrimitiveType::Int16 => 16,
            PrimitiveType::Int32 => 32,
            PrimitiveType::Int64 => 64,
            PrimitiveType::UInt8 => 8,
            PrimitiveType::UInt16 => 16,
            PrimitiveType::UInt32 => 32,
            PrimitiveType::UInt64 => 64,
            PrimitiveType::Bool => 8,
            _ => 0,
        }
    }

    pub fn is_signed(&self) -> bool {
        match self {
            PrimitiveType::Int8
            | PrimitiveType::Int16
            | PrimitiveType::Int32
            | PrimitiveType::Int64 => true,
            _ => false,
        }
    }

    pub fn is_unsigned(&self) -> bool {
        match self {
            PrimitiveType::UInt8
            | PrimitiveType::UInt16
            | PrimitiveType::UInt32
            | PrimitiveType::UInt64 => true,
            _ => false,
        }
    }

    pub fn is_compatible_with(&self, dest_type: &PrimitiveType, one_sided: bool) -> bool {
        if self == dest_type {
            return true;
        }

        if *self == PrimitiveType::Bool && *dest_type != PrimitiveType::Bool {
            return false;
        }

        if self.is_signed() && dest_type.is_unsigned() {
            return false;
        }

        if !one_sided {
            if *self != PrimitiveType::Bool && *dest_type == PrimitiveType::Bool {
                return false;
            }

            if self.is_unsigned() && dest_type.is_signed() {
                return false;
            }

            return true;
        }

        dest_type.get_size() > self.get_size()
    }
}

impl FromStr for PrimitiveType {
    type Err = ();

    fn from_str(s: &str) -> Result<PrimitiveType, ()> {
        match s {
            "i8" => Ok(PrimitiveType::Int8),
            "i16" => Ok(PrimitiveType::Int16),
            "i32" => Ok(PrimitiveType::Int32),
            "i64" => Ok(PrimitiveType::Int64),
            "u8" => Ok(PrimitiveType::UInt8),
            "u16" => Ok(PrimitiveType::UInt16),
            "u32" => Ok(PrimitiveType::UInt32),
            "u64" => Ok(PrimitiveType::UInt64),
            "bool" => Ok(PrimitiveType::Bool),
            _ => Err(()),
        }
    }
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