# SQL Optimizer Labs

Build a SQL optimizer in 1000 lines of Rust using [egg](https://egraphs-good.github.io).

🚧 Under construction 🚧 Stay tuned 👀

## Tasks

Fill the code in `src` and pass the tests in `tests`!

```sh
cargo test --test 1_language
cargo test --test 2_rewrite
cargo test --test 3_conditional_rewrite
cargo test --test 4_constant_folding
cargo test --test 5_sql_plan
cargo test --test 6_plan_elimination
cargo test --test 7_predicate_pushdown
cargo test --test 8_projection_pushdown
cargo test --test 9_agg_extraction
cargo test --test 10_index_resolving
```

## What's Next

These labs are taken from the [RisingLight] project.
[Check out] how it works in a real database system!

[RisingLight]: https://github.com/risinglightdb/risinglight
[Check out]: https://github.com/risinglightdb/risinglight/blob/main/src/planner/mod.rs
