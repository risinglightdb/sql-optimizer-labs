use sql_optimizer_labs::expr::rules;

egg::test_fn! {
    arithmetic,
    rules(),
    "(- (+ 1 (- 2 (* 3 (/ 4 5)))))" => "-3",
}

egg::test_fn! {
    cmp,
    rules(),
    "(> 1 2)" => "false",
}

egg::test_fn! {
    null,
    rules(),
    "(isnull (- (+ 1 (- 2 (* 3 (/ 4 null))))))" => "true",
}

egg::test_fn! {
    boolean,
    rules(),
    "(not (and (or null true) (xor (and false null) true)))" => "false",
}
