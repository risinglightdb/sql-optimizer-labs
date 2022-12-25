//! Plan optimization rules.

use std::collections::HashSet;

use super::*;
use egg::{rewrite as rw, Language, Pattern, Subst, Var};

/// Returns the rules that always improve the plan.
pub fn rules() -> Vec<Rewrite> {
    let mut rules = vec![];
    rules.extend(cancel_rules());
    rules.extend(merge_rules());
    rules.extend(pushdown_rules());
    rules.extend(join_rules());
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

#[rustfmt::skip]
fn pushdown_rules() -> Vec<Rewrite> { vec![
    pushdown("proj", "?exprs", "limit", "?limit ?offset"),
    pushdown("limit", "?limit ?offset", "proj", "?exprs"),
    pushdown("filter", "?cond", "order", "?keys"),
    pushdown("filter", "?cond", "limit", "?limit ?offset"),
    pushdown("filter", "?cond", "topn", "?limit ?offset ?keys"),
    rw!("pushdown-filter-join";
        "(filter ?cond (join inner ?on ?left ?right))" =>
        "(join inner (and ?on ?cond) ?left ?right)"
    ),
    rw!("pushdown-join-left";
        "(join inner (and ?cond1 ?cond2) ?left ?right)" =>
        "(join inner ?cond2 (filter ?cond1 ?left) ?right)"
        if columns_is_subset("?cond1", "?left")
    ),
    rw!("pushdown-join-left-1";
        "(join inner ?cond1 ?left ?right)" =>
        "(join inner true (filter ?cond1 ?left) ?right)"
        if columns_is_subset("?cond1", "?left")
    ),
    rw!("pushdown-join-right";
        "(join inner (and ?cond1 ?cond2) ?left ?right)" =>
        "(join inner ?cond2 ?left (filter ?cond1 ?right))"
        if columns_is_subset("?cond1", "?right")
    ),
    rw!("pushdown-join-right-1";
        "(join inner ?cond1 ?left ?right)" =>
        "(join inner true ?left (filter ?cond1 ?right))"
        if columns_is_subset("?cond1", "?right")
    ),
]}

/// Returns a rule to pushdown plan `a` through `b`.
fn pushdown(a: &str, a_args: &str, b: &str, b_args: &str) -> Rewrite {
    let name = format!("pushdown-{a}-{b}");
    let searcher = format!("({a} {a_args} ({b} {b_args} ?child))")
        .parse::<Pattern<_>>()
        .unwrap();
    let applier = format!("({b} {b_args} ({a} {a_args} ?child))")
        .parse::<Pattern<_>>()
        .unwrap();
    Rewrite::new(name, searcher, applier).unwrap()
}

#[rustfmt::skip]
pub fn join_rules() -> Vec<Rewrite> { vec![
    // we only have right rotation rule,
    // because the initial state is always a left-deep tree
    // thus left rotation is not needed.
    rw!("join-reorder";
        "(join ?type ?cond2 (join ?type ?cond1 ?left ?mid) ?right)" =>
        "(join ?type ?cond1 ?left (join ?type ?cond2 ?mid ?right))"
        if columns_is_disjoint("?cond2", "?left")
    ),
    rw!("hash-join-on-one-eq";
        "(join ?type (= ?el ?er) ?left ?right)" =>
        "(hashjoin ?type (list ?el) (list ?er) ?left ?right)"
        if columns_is_subset("?el", "?left")
        if columns_is_subset("?er", "?right")
    ),
    rw!("hash-join-on-two-eq";
        "(join ?type (and (= ?l1 ?r1) (= ?l2 ?r2)) ?left ?right)" =>
        "(hashjoin ?type (list ?l1 ?l2) (list ?r1 ?r2) ?left ?right)"
        if columns_is_subset("?l1", "?left")
        if columns_is_subset("?l2", "?left")
        if columns_is_subset("?r1", "?right")
        if columns_is_subset("?r2", "?right")
    ),
    // TODO: support more than two equals
]}

/// Returns true if the columns in `var1` are a subset of the columns in `var2`.
fn columns_is_subset(var1: &str, var2: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    columns_is(var1, var2, ColumnSet::is_subset)
}

/// Returns true if the columns in `var1` has no elements in common with the columns in `var2`.
fn columns_is_disjoint(var1: &str, var2: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    columns_is(var1, var2, ColumnSet::is_disjoint)
}

fn columns_is(
    var1: &str,
    var2: &str,
    f: impl Fn(&ColumnSet, &ColumnSet) -> bool,
) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let var1 = var1.parse::<Var>().unwrap();
    let var2 = var2.parse::<Var>().unwrap();
    move |egraph, _, subst| {
        let var1_set = &egraph[subst[var1]].data.columns;
        let var2_set = &egraph[subst[var2]].data.columns;
        f(var1_set, var2_set)
    }
}

/// The data type of column analysis.
pub type ColumnSet = HashSet<Column>;

/// Returns all columns involved in the node.
pub fn analyze_columns(egraph: &EGraph, enode: &Expr) -> ColumnSet {
    use Expr::*;
    let x = |i: &Id| &egraph[*i].data.columns;
    match enode {
        Column(col) => [*col].into_iter().collect(),
        Proj([exprs, _]) => x(exprs).clone(),
        Agg([exprs, group_keys, _]) => x(exprs).union(x(group_keys)).cloned().collect(),
        _ => {
            // merge the columns from all children
            (enode.children().iter())
                .flat_map(|id| x(id).iter().cloned())
                .collect()
        }
    }
}

/// Merge two result set and keep the smaller one.
pub fn merge(to: &mut ColumnSet, from: ColumnSet) -> DidMerge {
    if from.len() < to.len() {
        *to = from;
        DidMerge(true, false)
    } else {
        DidMerge(false, true)
    }
}
