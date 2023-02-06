//! Expression simplification rules and constant folding.

use egg::rewrite as rw;

use super::*;

/// Returns all rules of expression simplification.
#[rustfmt::skip]
pub fn rules() -> Vec<Rewrite> { vec![
    rw!("add-zero";  "(+ ?a 0)" => "?a"),

    // TODO: add more rules
]}
