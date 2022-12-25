use egg::Language;
use sql_optimizer_labs::{
    agg::{plan_select, Error},
    EGraph, RecExpr,
};

#[test]
fn no_agg() {
    // SELECT a FROM t;
    test(Case {
        select: "(list a)",
        from: "(scan t (list a))",
        where_: "",
        having: "",
        groupby: "",
        orderby: "",
        expected: Ok("
        (proj (list a)
            (order list
                (filter true
                    (filter true
                        (scan t (list a))
        ))))"),
    });
}

#[test]
fn agg() {
    // SELECT sum(a + b) + (a + 1) FROM t
    // WHERE b > 1
    // GROUP BY a + 1
    // HAVING count(a) > 1
    // ORDER BY max(b)
    test(Case {
        select: "(list (+ (sum (+ a b)) (+ a 1)))",
        from: "(scan t (list a b))",
        where_: "(> b 1)",
        having: "(> (count a) 1)",
        groupby: "(list (+ a 1))",
        orderby: "(list (asc (max b)))",
        expected: Ok("
        (proj (list (+ (` (sum (+ a b))) (` (+ a 1))))
            (order (list (asc (` (max b))))
                (filter (> (` (count a)) 1)
                    (agg (list (sum (+ a b)) (count a) (max b)) (list (+ a 1))
                        (filter (> b 1)
                            (scan t (list a b))
        )))))"),
    });
}

#[test]
fn error_agg_in_where() {
    // SELECT a FROM t WHERE sum(a) > 1
    test(Case {
        select: "(list a)",
        from: "(scan t (list a b))",
        where_: "(> (sum a) 1)",
        having: "",
        groupby: "",
        orderby: "",
        expected: Err(Error::AggInWhere),
    });
}

#[test]
fn error_agg_in_groupby() {
    // SELECT a FROM t GROUP BY sum(a)
    test(Case {
        select: "(list a)",
        from: "(scan t (list a b))",
        where_: "",
        having: "",
        groupby: "(list (sum a))",
        orderby: "",
        expected: Err(Error::AggInGroupBy),
    });
}

#[test]
fn error_nested_agg() {
    // SELECT count(sum(a)) FROM t
    test(Case {
        select: "(list (count (sum a)))",
        from: "(scan t (list a b))",
        where_: "",
        having: "",
        groupby: "",
        orderby: "",
        expected: Err(Error::NestedAgg("count".into())),
    });
}

#[test]
fn error_column_not_in_agg() {
    // SELECT b FROM t GROUP BY a
    test(Case {
        select: "(list b)",
        from: "(scan t (list a b))",
        where_: "",
        having: "",
        groupby: "(list a)",
        orderby: "",
        expected: Err(Error::ColumnNotInAgg("b".into())),
    });
}

struct Case {
    select: &'static str,
    from: &'static str,
    where_: &'static str,
    having: &'static str,
    groupby: &'static str,
    orderby: &'static str,
    expected: Result<&'static str, Error>,
}

#[track_caller]
fn test(mut case: Case) {
    if case.where_.is_empty() {
        case.where_ = "true";
    }
    if case.having.is_empty() {
        case.having = "true";
    }
    if case.groupby.is_empty() {
        case.groupby = "list";
    }
    if case.orderby.is_empty() {
        case.orderby = "list";
    }
    let mut egraph = EGraph::default();
    let projection = egraph.add_expr(&case.select.parse().unwrap());
    let from = egraph.add_expr(&case.from.parse().unwrap());
    let where_ = egraph.add_expr(&case.where_.parse().unwrap());
    let having = egraph.add_expr(&case.having.parse().unwrap());
    let groupby = egraph.add_expr(&case.groupby.parse().unwrap());
    let orderby = egraph.add_expr(&case.orderby.parse().unwrap());
    match plan_select(
        &mut egraph,
        from,
        where_,
        having,
        groupby,
        orderby,
        projection,
    ) {
        Err(e) => assert_eq!(case.expected, Err(e)),
        Ok(id) => {
            let get_node = |id| egraph[id].nodes[0].clone();
            let actual = get_node(id).build_recexpr(get_node).to_string();
            let expected = case
                .expected
                .expect(&format!("expect error, but got: {actual:?}"))
                .parse::<RecExpr>()
                .unwrap()
                .to_string();
            assert_eq!(actual, expected);
        }
    }
}
