use std::{
    fmt::Display,
    ops::{Neg, Not},
    str::FromStr,
};

/// SQL value.
///
/// # Display and Parse Format
///
/// - Null: `null`
/// - Bool: `false`
/// - Integer: `1`
/// - String: `'string'`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i32),
    String(String),
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(i) => write!(f, "{i}"),
            Value::String(s) => write!(f, "'{s}'"),
        }
    }
}

impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "null" {
            return Ok(Value::Null);
        } else if let Ok(i) = s.parse() {
            return Ok(Value::Bool(i));
        } else if let Ok(i) = s.parse() {
            return Ok(Value::Int(i));
        } else if s.starts_with('\'') && s.ends_with('\'') {
            return Ok(Value::String(s[1..s.len() - 1].to_string()));
        }
        Err(s.to_string())
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn is_zero(&self) -> bool {
        matches!(self, Value::Int(0))
    }
}

macro_rules! impl_arith_for_value {
    ($Trait:ident, $name:ident) => {
        impl std::ops::$Trait for &Value {
            type Output = Value;

            fn $name(self, rhs: Self) -> Self::Output {
                use Value::*;
                match (self, rhs) {
                    (Null, _) | (_, Null) => Null,
                    (&Int(x), &Int(y)) => Int(x.$name(y)),
                    _ => panic!(
                        "invalid operation: {:?} {} {:?}",
                        self,
                        stringify!($name),
                        rhs
                    ),
                }
            }
        }

        impl std::ops::$Trait for Value {
            type Output = Value;
            fn $name(self, rhs: Self) -> Self::Output {
                (&self).$name(&rhs)
            }
        }
    };
}
impl_arith_for_value!(Add, add);
impl_arith_for_value!(Sub, sub);
impl_arith_for_value!(Mul, mul);
impl_arith_for_value!(Div, div);
impl_arith_for_value!(Rem, rem);

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Self::Output {
        use Value::*;
        match self {
            Null => Null,
            Int(i) => Int(-i),
            _ => panic!("invalid operation: -{:?}", self),
        }
    }
}

impl Value {
    pub fn and(&self, rhs: &Value) -> Value {
        use Value::*;
        match (self, rhs) {
            (Null, _) | (_, Null) => Null,
            (Bool(false), _) | (_, Bool(false)) => Bool(false),
            (&Bool(x), &Bool(y)) => Bool(x && y),
            _ => panic!("invalid operation: {:?} and {:?}", self, rhs),
        }
    }

    pub fn or(&self, rhs: &Value) -> Value {
        use Value::*;
        match (self, rhs) {
            (Null, _) | (_, Null) => Null,
            (Bool(true), _) | (_, Bool(true)) => Bool(true),
            (&Bool(x), &Bool(y)) => Bool(x || y),
            _ => panic!("invalid operation: {:?} or {:?}", self, rhs),
        }
    }

    pub fn xor(&self, rhs: &Value) -> Value {
        use Value::*;
        match (self, rhs) {
            (Null, _) | (_, Null) => Null,
            (&Bool(x), &Bool(y)) => Bool(x ^ y),
            _ => panic!("invalid operation: {:?} xor {:?}", self, rhs),
        }
    }
}

impl Not for Value {
    type Output = Value;

    fn not(self) -> Self::Output {
        use Value::*;
        match self {
            Null => Null,
            Bool(b) => Bool(!b),
            _ => panic!("invalid operation: not {:?}", self),
        }
    }
}

pub type Column = egg::Symbol;

/// The physical index to the column of the child plan.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Clone)]
pub struct ColumnIndex(pub u32);

impl Display for ColumnIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

impl FromStr for ColumnIndex {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let body = s
            .strip_prefix('#')
            .ok_or_else(|| "no leading #".to_string())?;
        let num = body.parse().map_err(|e| format!("invalid number: {e}"))?;
        Ok(Self(num))
    }
}
