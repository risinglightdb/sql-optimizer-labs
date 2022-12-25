use sql_optimizer_labs::plan::column_pruning_rules;

egg::test_fn! {
    column_pruning,
    column_pruning_rules(),
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
            (filter (= (sum z) 1)
                (agg (list (sum z)) (list b c)
                    (filter (> y 1)
                        (join inner (= a x)
                            (scan t1 (list a b c))
                            (scan t2 (list x y z))
    ))))))"
}
