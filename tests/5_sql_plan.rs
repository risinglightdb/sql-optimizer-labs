use sql_optimizer_labs::RecExpr;

#[test]
fn aggregations() {
    assert_parse_expr("(max a)");
    assert_parse_expr("(min a)");
    assert_parse_expr("(sum a)");
    assert_parse_expr("(avg a)");
    assert_parse_expr("(count a)");
}

#[test]
fn plans() {
    // SELECT a, b FROM t;
    assert_parse_expr("(scan t (list a b))");

    // VALUES (false, 1), (true, 2);
    assert_parse_expr(
        "(values (list 
            (list false 1) 
            (list true 2)
        ))",
    );

    let child = "(scan t (list a b))";
    // SELECT a FROM t;
    assert_parse_expr(&format!("(proj (list a) {child})"));

    // SELECT max(a) FROM t GROUP BY b;
    assert_parse_expr(&format!("(agg (list (max a)) (list b) {child})"));

    // SELECT a, b FROM t WHERE a = 1;
    assert_parse_expr(&format!("(filter (= a 1) {child})"));

    // SELECT a, b FROM t ORDER BY a ASC, b DESC;
    assert_parse_expr(&format!("(order (list (asc a) (desc b)) {child})"));

    // SELECT a, b FROM t LIMIT 10 OFFSET 1;
    assert_parse_expr(&format!("(limit 10 1 {child})"));

    // SELECT a, b FROM t ORDER BY a ASC, b DESC LIMIT 10 OFFSET 1;
    assert_parse_expr(&format!("(topn 10 1 (list (asc a) (desc b)) {child})"));

    // SELECT a, b, c, d FROM t1, t2 WHERE a = c;
    for join_type in &["inner", "left_outer", "right_outer", "full_outer"] {
        assert_parse_expr(&format!(
            "(join {join_type} (list (= a c))
                (scan t1 (list a b))
                (scan t2 (list c d))
            )",
        ));
    }

    // SELECT a, b, c, d FROM t1, t2 WHERE a = c;
    assert_parse_expr(
        "(hashjoin inner (list a) (list c)
            (scan t1 (list a b))
            (scan t2 (list c d))
        )",
    );
}

#[track_caller]
fn assert_parse_expr(expr: &str) {
    expr.parse::<RecExpr>().expect("failed to parse expression");
}
