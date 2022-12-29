//! Analyze schema and replace all column references with physical indices.
//!
//! This is the final step before executing.

use egg::Subst;

use super::*;

/// The data type of schema analysis.
pub type Schema = Option<Vec<Id>>;

/// Returns the output expressions for plan node.
pub fn analyze_schema(egraph: &EGraph, enode: &Expr) -> Schema {
    use Expr::*;
    let x = |i: &Id| egraph[*i].data.schema.clone();
    let concat = |v1: Vec<Id>, v2: Vec<Id>| v1.into_iter().chain(v2.into_iter()).collect();
    Some(match enode {
        // equal to child
        Filter([_, c]) | Order([_, c]) | Limit([_, _, c]) | TopN([_, _, _, c]) | Empty(c) => x(c)?,

        // concat 2 children
        Join([_, _, l, r]) | HashJoin([_, _, _, l, r]) => concat(x(l)?, x(r)?),

        // list is the source for the following nodes
        List(ids) => ids.to_vec(),

        // plans that change schema
        Scan([_, columns]) => x(columns)?,
        Values(vs) => vs.first().and_then(x)?,
        Proj([exprs, _]) => x(exprs)?,
        Agg([exprs, group_keys, _]) => concat(x(exprs)?, x(group_keys)?),

        // prune node may changes the schema, but we don't know the exact result for now
        // so just return `None` to indicate "unknown"
        Prune(_) => return None,

        // not plan node
        _ => return None,
    })
}

/// Returns true if the schema of two nodes is equal.
pub fn schema_is_eq(v1: &str, v2: &str) -> impl Fn(&mut EGraph, Id, &Subst) -> bool {
    let v1 = var(v1);
    let v2 = var(v2);
    move |egraph, _, subst| {
        let s1 = &egraph[subst[v1]].data.schema;
        let s2 = &egraph[subst[v2]].data.schema;
        s1.is_some() && s1 == s2
    }
}
