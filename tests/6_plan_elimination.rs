use sql_optimizer_labs::plan::rules;

egg::test_fn! {
    limit_0,
    rules(),
    "(limit 0 0
        (scan t (list a b)))" =>
    "(empty (scan t (list a b)))",
}

egg::test_fn! {
    limit_null,
    rules(),
    "(limit null 0
        (scan t (list a b)))" =>
    "(scan t (list a b))",
}

egg::test_fn! {
    order_null,
    rules(),
    "(order (list)
        (scan t (list a b)))" =>
    "(scan t (list a b))",
}

egg::test_fn! {
    filter_true,
    rules(),
    "(filter true
        (scan t (list a b)))" =>
    "(scan t (list a b))",
}

egg::test_fn! {
    filter_false,
    rules(),
    "(filter false
        (scan t (list a b)))" =>
    "(empty (scan t (list a b)))",
}

egg::test_fn! {
    inner_join_false,
    rules(),
    "(join inner false
        (scan t1 (list a b))
        (scan t2 (list c d)))" => 
    "(empty (join inner false
        (scan t1 (list a b))
        (scan t2 (list c d))
    ))",
}

egg::test_fn! {
    empty_propagation,
    rules(),
    "(proj (list b)
        (limit 1 1
            (order (list (asc (sum a)))
                (filter (= a 1)
                    (join inner false
                        (scan t1 (list a b))
                        (scan t2 (list c d))
    )))))" => 
    "(empty (list b))",
}

egg::test_fn! {
    limit_order_topn,
    rules(),
    "(limit 10 1
        (order (list (asc a)) 
            (scan t (list a b))))" =>
    "(topn 10 1 (list (asc a))
        (scan t (list a b)))",
}

egg::test_fn! {
    filter_merge,
    rules(),
    "(filter (= a 1)
        (filter (= b 2) 
            (scan t (list a b))))" =>
    "(filter (and (= a 1) (= b 2))
        (scan t (list a b)))"
}

egg::test_fn! {
    proj_merge,
    rules(),
    "(proj (list a)
        (proj (list a b) 
            (scan t (list a b))))" =>
    "(proj (list a)
        (scan t (list a b)))"
}
