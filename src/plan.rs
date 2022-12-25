//! Plan optimization rules.

use super::*;
use egg::rewrite as rw;

/// Returns the rules that always improve the plan.
pub fn rules() -> Vec<Rewrite> {
    let mut rules = vec![];
    rules.extend(cancel_rules());
    rules.extend(merge_rules());
    rules
}

#[rustfmt::skip]
fn cancel_rules() -> Vec<Rewrite> { vec![
    rw!("limit-null";       "(limit null 0 ?child)"     => "?child"),
    rw!("limit-0";          "(limit 0 ?offset ?child)"  => "(empty ?child)"),
    rw!("order-null";       "(order (list) ?child)"     => "?child"),
    rw!("filter-true";      "(filter true ?child)"      => "?child"),
    rw!("filter-false";     "(filter false ?child)"     => "(empty ?child)"),
    rw!("inner-join-false"; "(join inner false ?l ?r)"  => "(empty (join inner false ?l ?r))"),

    rw!("proj-on-empty";    "(proj ?exprs (empty ?c))"                  => "(empty ?exprs)"),
    rw!("filter-on-empty";  "(filter ?cond (empty ?c))"                 => "(empty ?c)"),
    rw!("order-on-empty";   "(order ?keys (empty ?c))"                  => "(empty ?c)"),
    rw!("limit-on-empty";   "(limit ?limit ?offset (empty ?c))"         => "(empty ?c)"),
    rw!("topn-on-empty";    "(topn ?limit ?offset ?keys (empty ?c))"    => "(empty ?c)"),
    rw!("inner-join-on-left-empty";  "(join inner ?on (empty ?l) ?r)"   => "(empty (join inner false ?l ?r))"),
    rw!("inner-join-on-right-empty"; "(join inner ?on ?l (empty ?r))"   => "(empty (join inner false ?l ?r))"),
]}

#[rustfmt::skip]
fn merge_rules() -> Vec<Rewrite> { vec![
    rw!("limit-order-topn";
        "(limit ?limit ?offset (order ?keys ?child))" =>
        "(topn ?limit ?offset ?keys ?child)"
    ),
    rw!("filter-merge";
        "(filter ?cond1 (filter ?cond2 ?child))" =>
        "(filter (and ?cond1 ?cond2) ?child)"
    ),
    rw!("proj-merge";
        "(proj ?exprs1 (proj ?exprs2 ?child))" =>
        "(proj ?exprs1 ?child)"
    ),
]}
