use sql_optimizer_labs::expr::rules;

egg::test_fn! {
    add_sub,
    rules(),
    "(+ (- (- a 0)) (+ a b))" => "b",
}

egg::test_fn! {
    mul,
    rules(),
    "(+ (* (- b) a) (* b a))" => "0",
}

egg::test_fn! {
    cmp,
    rules(),
    "(> (+ a b) a)" => "(< 0 b)",
}

egg::test_fn! {
    boolean,
    rules(),
    "(and (xor a true) (or (and a b) (and (not b) a)))" => "false",
}
