use std::{fmt::Display, str::FromStr};

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
