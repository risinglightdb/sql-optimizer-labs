use sql_optimizer_labs::{Expr, RecExpr, Value};

#[test]
fn values() {
    assert_parse_value("null", Value::Null);
    assert_parse_value("true", Value::Bool(true));
    assert_parse_value("1", Value::Int(1));
    assert_parse_value("'string'", Value::String("string".into()));
}

#[test]
fn columns() {
    assert_parse_expr("a");
    assert_parse_expr("t.a");
}

#[test]
fn list() {
    assert_parse_expr("(list null 1 2)");
}

#[test]
fn operations() {
    assert_parse_expr("(isnull null)");
    assert_parse_expr("(- a)");
    assert_parse_expr("(+ a b)");
    assert_parse_expr("(- a b)");
    assert_parse_expr("(* a b)");
    assert_parse_expr("(/ a b)");
    assert_parse_expr("(= a b)");
    assert_parse_expr("(<> a b)");
    assert_parse_expr("(> a b)");
    assert_parse_expr("(< a b)");
    assert_parse_expr("(>= a b)");
    assert_parse_expr("(<= a b)");
    assert_parse_expr("(not a)");
    assert_parse_expr("(and a b)");
    assert_parse_expr("(or a b)");
    assert_parse_expr("(xor a b)");
}

#[track_caller]
fn assert_parse_value(expr: &str, value: Value) {
    assert_eq!(
        expr.parse::<RecExpr>().unwrap()[0.into()],
        Expr::Constant(value)
    );
}

#[track_caller]
fn assert_parse_expr(expr: &str) {
    assert_eq!(expr.parse::<RecExpr>().unwrap().to_string(), expr);
}
