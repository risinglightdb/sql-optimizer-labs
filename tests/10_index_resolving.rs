use sql_optimizer_labs::plan::rules;

egg::test_fn! {
    identical_projection,
    rules(),
    "(proj (list a b)
        (scan t (list a b)))" =>
    "(scan t (list a b))",
}
