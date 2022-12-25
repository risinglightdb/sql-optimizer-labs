use egg::{define_language, Id, Symbol};

pub mod expr;
mod value;

pub use value::*;

pub type RecExpr = egg::RecExpr<Expr>;
pub type Rewrite = egg::Rewrite<Expr, ()>;

define_language! {
    pub enum Expr {
        // values
        Constant(Value),            // null, true, 1, 'hello'
        Column(Symbol),             // t.a, b, c

        // utilities
        "list" = List(Box<[Id]>),   // (list ...)

        // unary operations
        "-" = Neg(Id),
        "not" = Not(Id),
        "isnull" = IsNull(Id),

        // binary operations
        "+" = Add([Id; 2]),
        "-" = Sub([Id; 2]),
        "*" = Mul([Id; 2]),
        "/" = Div([Id; 2]),
        "=" = Eq([Id; 2]),
        "<>" = NotEq([Id; 2]),
        ">" = Gt([Id; 2]),
        "<" = Lt([Id; 2]),
        ">=" = GtEq([Id; 2]),
        "<=" = LtEq([Id; 2]),
        "and" = And([Id; 2]),
        "or" = Or([Id; 2]),
        "xor" = Xor([Id; 2]),
    }
}
