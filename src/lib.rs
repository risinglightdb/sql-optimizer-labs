use egg::{define_language, Analysis, DidMerge, Id, Symbol};

pub mod expr;
mod value;

pub use value::*;

pub type RecExpr = egg::RecExpr<Expr>;
type EGraph = egg::EGraph<Expr, ExprAnalysis>;
type Rewrite = egg::Rewrite<Expr, ExprAnalysis>;

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

/// The unified analysis for all rules.
#[derive(Default)]
pub struct ExprAnalysis;

/// The analysis data associated with each eclass.
///
/// See [`egg::Analysis`] for how data is being processed.
#[derive(Debug)]
pub struct Data {
    /// Some if the expression is a constant.
    pub constant: expr::ConstValue,
}

impl Analysis<Expr> for ExprAnalysis {
    type Data = Data;

    /// Analyze a node and give the result.
    fn make(egraph: &EGraph, enode: &Expr) -> Self::Data {
        Data {
            constant: expr::eval_constant(egraph, enode),
        }
    }

    /// Merge the analysis data with previous one.
    ///
    /// This process makes the analysis data more accurate.
    ///
    /// For example, if we have an expr `a + 1 - a`, the constant analysis will give a result `None`
    /// since we are not sure if it is a constant or not. But after we applied a rule and turned
    /// it to `a - a + 1` -> `0 + 1`, we know it is a constant. Then in this function, we merge the
    /// new result `Some(1)` with the previous `None` and keep `Some(1)` as the final result.
    fn merge(&mut self, to: &mut Self::Data, from: Self::Data) -> DidMerge {
        let merge_const = egg::merge_max(&mut to.constant, from.constant);
        merge_const
    }

    /// Modify the graph after analyzing a node.
    fn modify(egraph: &mut EGraph, id: Id) {
        expr::union_constant(egraph, id);
    }
}
