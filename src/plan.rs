//! Plan optimization rules.

use std::collections::HashSet;

use super::*;
use egg::rewrite as rw;

/// Returns the rules that always improve the plan.
pub fn rules() -> Vec<Rewrite> {
    let mut rules = vec![];
    rules.extend(projection_pushdown_rules());
    rules.extend(join_rules());
    // TODO: add rules
    rules
}

#[rustfmt::skip]
pub fn join_rules() -> Vec<Rewrite> { vec![
    // TODO: add rules
]}

/// Pushdown projections and prune unused columns.
#[rustfmt::skip]
pub fn projection_pushdown_rules() -> Vec<Rewrite> { vec![
    // TODO: add rules
]}
