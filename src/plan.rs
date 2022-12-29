//! Plan optimization rules.

use std::collections::HashSet;

use crate::schema::schema_is_eq;

use super::*;
use egg::{rewrite as rw, Applier, Language, Pattern, PatternAst, Subst, Symbol, Var};

/// Returns the rules that always improve the plan.
pub fn rules() -> Vec<Rewrite> {
    let mut rules = vec![];
    rules.extend(cancel_rules());
    rules.extend(merge_rules());
    rules.extend(predicate_pushdown_rules());
    rules.extend(projection_pushdown_rules());
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
fn predicate_pushdown_rules() -> Vec<Rewrite> { vec![
    pushdown("filter", "?cond", "order", "?keys"),
    pushdown("filter", "?cond", "limit", "?limit ?offset"),
    pushdown("filter", "?cond", "topn", "?limit ?offset ?keys"),
    rw!("pushdown-filter-join";
        "(filter ?cond (join inner ?on ?left ?right))" =>
        "(join inner (and ?on ?cond) ?left ?right)"
    ),
    rw!("pushdown-filter-join-left";
        "(join inner (and ?cond1 ?cond2) ?left ?right)" =>
        "(join inner ?cond2 (filter ?cond1 ?left) ?right)"
        if columns_is_subset("?cond1", "?left")
    ),
    rw!("pushdown-filter-join-left-1";
        "(join inner ?cond1 ?left ?right)" =>
        "(join inner true (filter ?cond1 ?left) ?right)"
        if columns_is_subset("?cond1", "?left")
    ),
    rw!("pushdown-filter-join-right";
        "(join inner (and ?cond1 ?cond2) ?left ?right)" =>
        "(join inner ?cond2 ?left (filter ?cond1 ?right))"
        if columns_is_subset("?cond1", "?right")
    ),
    rw!("pushdown-filter-join-right-1";
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
    let var1 = var(var1);
    let var2 = var(var2);
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
        ColumnPrune([filter, _]) => x(filter).clone(), // inaccurate
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

/// Pushdown projections and prune unused columns.
#[rustfmt::skip]
pub fn projection_pushdown_rules() -> Vec<Rewrite> { vec![
    rw!("identical-proj";
        "(proj ?exprs ?child)" => "?child"
        if schema_is_eq("?exprs", "?child")
    ),
    pushdown("proj", "?exprs", "limit", "?limit ?offset"),
    pushdown("limit", "?limit ?offset", "proj", "?exprs"),
    rw!("pushdown-proj-order";
        "(proj ?exprs (order ?keys ?child))" =>
        "(proj ?exprs (order ?keys (proj (column-merge ?exprs ?keys) ?child)))"
    ),
    rw!("pushdown-proj-topn";
        "(proj ?exprs (topn ?limit ?offset ?keys ?child))" =>
        "(proj ?exprs (topn ?limit ?offset ?keys (proj (column-merge ?exprs ?keys) ?child)))"
    ),
    rw!("pushdown-proj-filter";
        "(proj ?exprs (filter ?cond ?child))" =>
        "(proj ?exprs (filter ?cond (proj (column-merge ?exprs ?cond) ?child)))"
    ),
    rw!("pushdown-proj-agg";
        "(agg ?aggs ?groupby ?child)" =>
        "(agg ?aggs ?groupby (proj (column-merge ?aggs ?groupby) ?child))"
    ),
    rw!("pushdown-proj-join";
        "(proj ?exprs (join ?type ?on ?left ?right))" =>
        "(proj ?exprs (join ?type ?on
            (proj (column-prune ?left (column-merge ?exprs ?on)) ?left)
            (proj (column-prune ?right (column-merge ?exprs ?on)) ?right)
        ))"
    ),
    // column pruning
    rw!("pushdown-proj-scan";
        "(proj ?exprs (scan ?table ?columns))" =>
        "(proj ?exprs (scan ?table (column-prune ?exprs ?columns)))"
    ),
    // evaluate 'column-merge' and 'column-prune'
    rw!("column-merge";
        "(column-merge ?list1 ?list2)" =>
        { ColumnMerge {
            lists: [var("?list1"), var("?list2")],
        }}
    ),
    rw!("column-prune";
        "(column-prune ?filter ?list)" =>
        { ColumnPrune {
            filter: var("?filter"),
            list: var("?list"),
        }}
        if is_list("?list")
    ),
]}

/// Return a list of columns from `lists`.
struct ColumnMerge {
    lists: [Var; 2],
}

impl Applier<Expr, ExprAnalysis> for ColumnMerge {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        subst: &Subst,
        _searcher_ast: Option<&PatternAst<Expr>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let list1 = &egraph[subst[self.lists[0]]].data.columns;
        let list2 = &egraph[subst[self.lists[1]]].data.columns;
        let mut list: Vec<&Column> = list1.union(list2).collect();
        list.sort_unstable_by_key(|c| c.as_str());
        let list = list
            .into_iter()
            .map(|col| egraph.lookup(Expr::Column(col.clone())).unwrap())
            .collect();
        let id = egraph.add(Expr::List(list));

        // copied from `Pattern::apply_one`
        if egraph.union(eclass, id) {
            vec![eclass]
        } else {
            vec![]
        }
    }
}

/// Remove element from `list` whose column set is not a subset of `filter`
struct ColumnPrune {
    filter: Var,
    list: Var,
}

impl Applier<Expr, ExprAnalysis> for ColumnPrune {
    fn apply_one(
        &self,
        egraph: &mut EGraph,
        eclass: Id,
        subst: &Subst,
        _searcher_ast: Option<&PatternAst<Expr>>,
        _rule_name: Symbol,
    ) -> Vec<Id> {
        let columns = &egraph[subst[self.filter]].data.columns;
        let list = egraph[subst[self.list]].as_list();
        let pruned = (list.iter().cloned())
            .filter(|id| egraph[*id].data.columns.is_subset(columns))
            .collect();
        let id = egraph.add(Expr::List(pruned));

        // copied from `Pattern::apply_one`
        if egraph.union(eclass, id) {
            vec![eclass]
        } else {
            vec![]
        }
    }
}

/// Returns true if the variable is a list.
fn is_list(v: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let v = var(v);
    move |egraph, _, subst| {
        egraph[subst[v]]
            .iter()
            .any(|node| matches!(node, Expr::List(_)))
    }
}
