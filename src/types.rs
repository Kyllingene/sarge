use crate::ArgumentValue;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ArgumentValueType {
    Bool,
    String,
    I64,
    U64,
    Float,
}

pub trait ArgumentType: Sized {
    fn to_argtyp() -> ArgumentValueType;
    fn to_argval(self) -> ArgumentValue;
    fn from_argval(val: ArgumentValue) -> Option<Self>;
}

impl ArgumentType for bool {
    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::Bool
    }

    fn to_argval(self) -> ArgumentValue {
        ArgumentValue::Bool(self)
    }

    fn from_argval(val: ArgumentValue) -> Option<Self> {
        if let ArgumentValue::Bool(b) = val {
            Some(b)
        } else {
            None
        }
    }
}

impl ArgumentType for String {
    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::String
    }

    fn to_argval(self) -> ArgumentValue {
        ArgumentValue::String(self)
    }

    fn from_argval(val: ArgumentValue) -> Option<Self> {
        if let ArgumentValue::String(s) = val {
            Some(s)
        } else {
            None
        }
    }
}

impl ArgumentType for i64 {
    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::I64
    }

    fn to_argval(self) -> ArgumentValue {
        ArgumentValue::I64(self)
    }

    fn from_argval(val: ArgumentValue) -> Option<Self> {
        if let ArgumentValue::I64(i) = val {
            Some(i)
        } else {
            None
        }
    }
}

impl ArgumentType for u64 {
    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::U64
    }

    fn to_argval(self) -> ArgumentValue {
        ArgumentValue::U64(self)
    }

    fn from_argval(val: ArgumentValue) -> Option<Self> {
        if let ArgumentValue::U64(u) = val {
            Some(u)
        } else {
            None
        }
    }
}

impl ArgumentType for f64 {
    fn to_argtyp() -> ArgumentValueType {
        ArgumentValueType::Float
    }

    fn to_argval(self) -> ArgumentValue {
        ArgumentValue::Float(self)
    }

    fn from_argval(val: ArgumentValue) -> Option<Self> {
        if let ArgumentValue::Float(f) = val {
            Some(f)
        } else {
            None
        }
    }
}
