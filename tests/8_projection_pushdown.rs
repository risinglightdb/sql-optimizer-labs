use sql_optimizer_labs::plan::projection_pushdown_rules;

egg::test_fn! {
    scan,
    projection_pushdown_rules(),
    // SELECT a FROM t(a, b, c, d)
    "
    (proj (list a)
        (scan t (list a b c d))
    )" => "
    (scan t (list a))"
}

egg::test_fn! {
    filter,
    projection_pushdown_rules(),
    // SELECT a FROM t(a, b, c, d) WHERE b > 1
    "
    (proj (list a)
        (filter (> b 1)
            (scan t (list a b c d))
    ))" => "
    (proj (list a)
        (filter (> b 1)
            (scan t (list a b))
    ))"
}

egg::test_fn! {
    join,
    projection_pushdown_rules(),
    // SELECT b FROM t1(a, b, c, d) JOIN t2(x, y, z, w) ON a = x
    "
    (proj (list b)
        (join inner (= a x)
            (scan t1 (list a b c d))
            (scan t2 (list x y z w))
    ))" => "
    (proj (list b)
        (join inner (= a x)
            (scan t1 (list a b))
            (scan t2 (list x))
    ))"
}

egg::test_fn! {
    agg,
    projection_pushdown_rules(),
    // SELECT sum(a) FROM t(a, b, c, d) GROUP BY b
    "
    (proj (list (sum a))
        (agg (list (sum a)) (list b)
            (scan t (list a b c d))
    ))" => "
    (proj (list (sum a))
        (agg (list (sum a)) (list b)
            (scan t (list a b))
    ))"
}

egg::test_fn! {
    having,
    projection_pushdown_rules(),
    // SELECT b FROM t(a, b, c, d) GROUP BY b HAVING sum(a) > 1
    "
    (proj (list b)
        (filter (> (sum a) 1)
            (agg (list (sum a)) (list b)
                (scan t (list a b c d))
    )))" => "
    (proj (list b)
        (filter (> (sum a) 1)
        (proj (list a b)
            (agg (list (sum a)) (list b)
                (scan t (list a b))
    ))))"
}

egg::test_fn! {
    projection_pushdown_2,
    projection_pushdown_rules(),
    // SELECT b
    // FROM t1(a, b, c, d)
    //      JOIN t2(x, y, z, w) ON a = x
    // WHERE y > 1
    // GROUP BY b, c
    // HAVING sum(z) = 1
    // ORDER BY c;
    "
    (proj (list b)
        (order (list (asc c))
            (filter (= (sum z) 1)
                (agg (list (sum z)) (list b c)
                    (filter (> y 1)
                        (join inner (= a x)
                            (scan t1 (list a b c d))
                            (scan t2 (list x y z w))
    ))))))" => "
    (proj (list b)
        (order (list (asc c))
        (proj (list b c)
            (filter (= (sum z) 1)
            (proj (list b c z)
                (agg (list (sum z)) (list b c)
                (proj (list b c z)
                    (filter (> y 1)
                    (proj (list b c y z)
                        (join inner (= a x)
                            (scan t1 (list a b c))
                            (scan t2 (list x y z))
    ))))))))))"
}
