use egg::Language;

use super::*;

/// The data type of aggragation analysis.
pub type AggSet = Vec<Expr>;

/// Returns all aggragations in the tree.
///
/// Note: if there is an agg over agg, e.g. `sum(count(a))`, only the upper one will be returned.
pub fn analyze_aggs(egraph: &EGraph, enode: &Expr) -> AggSet {
    use Expr::*;
    let x = |i: &Id| egraph[*i].data.aggs.clone();
    match enode {
        Max(_) | Min(_) | Sum(_) | Avg(_) | Count(_) => vec![enode.clone()],
        // merge the set from all children
        Nested(_) | List(_) | Neg(_) | Not(_) | IsNull(_) | Add(_) | Sub(_) | Mul(_) | Div(_)
        | Eq(_) | NotEq(_) | Gt(_) | Lt(_) | GtEq(_) | LtEq(_) | And(_) | Or(_) | Xor(_)
        | Asc(_) | Desc(_) => enode.children().iter().flat_map(x).collect(),
        // ignore plan nodes
        _ => vec![],
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    // #[error("aggregate function calls cannot be nested")]
    NestedAgg(String),
    // #[error("WHERE clause cannot contain aggregates")]
    AggInWhere,
    // #[error("GROUP BY clause cannot contain aggregates")]
    AggInGroupBy,
    // #[error("column {0} must appear in the GROUP BY clause or be used in an aggregate function")]
    ColumnNotInAgg(String),
}

/// Converts the SELECT statement into a plan tree.
///
/// The nodes of all clauses have been added to the `egraph`.
/// `from`, `where_`... are the ids of their root node.
pub fn plan_select(
    egraph: &mut EGraph,
    from: Id,
    where_: Id,
    having: Id,
    groupby: Id,
    orderby: Id,
    projection: Id,
) -> Result<Id, Error> {
    AggExtractor { egraph }.plan_select(from, where_, having, groupby, orderby, projection)
}

struct AggExtractor<'a> {
    egraph: &'a mut EGraph,
}

impl AggExtractor<'_> {
    fn aggs(&self, id: Id) -> &[Expr] {
        &self.egraph[id].data.aggs
    }

    fn node(&self, id: Id) -> &Expr {
        &self.egraph[id].nodes[0]
    }

    fn plan_select(
        &mut self,
        from: Id,
        where_: Id,
        having: Id,
        groupby: Id,
        orderby: Id,
        projection: Id,
    ) -> Result<Id, Error> {
        if !self.aggs(where_).is_empty() {
            return Err(Error::AggInWhere);
        }
        if !self.aggs(groupby).is_empty() {
            return Err(Error::AggInGroupBy);
        }
        let mut plan = self.egraph.add(Expr::Filter([where_, from]));
        let mut to_rewrite = [projection, having, orderby];
        plan = self.plan_agg(&mut to_rewrite, groupby, plan)?;
        let [projection, having, orderby] = to_rewrite;
        plan = self.egraph.add(Expr::Filter([having, plan]));
        plan = self.egraph.add(Expr::Order([orderby, plan]));
        plan = self.egraph.add(Expr::Proj([projection, plan]));
        Ok(plan)
    }

    /// Extracts all aggregations from `exprs` and generates an [`Agg`](Expr::Agg) plan.
    /// If no aggregation is found and no `groupby` keys, returns the original `plan`.
    fn plan_agg(&mut self, exprs: &mut [Id], groupby: Id, plan: Id) -> Result<Id, Error> {
        let expr_list = self.egraph.add(Expr::List(exprs.to_vec().into()));
        let aggs = self.aggs(expr_list).to_vec();
        if aggs.is_empty() && self.node(groupby).as_list().is_empty() {
            return Ok(plan);
        }
        // check nested agg
        for agg in aggs.iter() {
            if agg
                .children()
                .iter()
                .any(|child| !self.aggs(*child).is_empty())
            {
                return Err(Error::NestedAgg(agg.to_string()));
            }
        }
        let mut list: Vec<_> = aggs.into_iter().map(|agg| self.egraph.add(agg)).collect();
        // make sure the order of the aggs is deterministic
        list.sort();
        list.dedup();
        let mut schema = list.clone();
        schema.extend_from_slice(self.node(groupby).as_list());
        let aggs = self.egraph.add(Expr::List(list.into()));
        let plan = self.egraph.add(Expr::Agg([aggs, groupby, plan]));
        // check for not aggregated columns
        // rewrite the expressions with a wrapper over agg or group keys
        for id in exprs {
            *id = self.rewrite_agg_in_expr(*id, &schema)?;
        }
        Ok(plan)
    }

    /// Rewrites the expression `id` with aggs wrapped in a [`Nested`](Expr::Nested) node.
    /// Returns the new expression.
    ///
    /// # Example
    /// ```text
    /// id:         (+ (sum a) (+ b 1))
    /// schema:     (sum a), (+ b 1)
    /// output:     (+ (`(sum a)) (`(+ b 1)))
    ///
    /// so that `id` won't be optimized to:
    ///             (+ b (+ (sum a) 1))
    /// which can not be composed by `schema`
    /// ```
    fn rewrite_agg_in_expr(&mut self, id: Id, schema: &[Id]) -> Result<Id, Error> {
        let mut expr = self.node(id).clone();
        if schema.contains(&id) {
            // found agg, wrap it with Nested
            return Ok(self.egraph.add(Expr::Nested(id)));
        }
        if let Expr::Column(cid) = &expr {
            return Err(Error::ColumnNotInAgg(cid.to_string()));
        }
        for child in expr.children_mut() {
            *child = self.rewrite_agg_in_expr(*child, schema)?;
        }
        Ok(self.egraph.add(expr))
    }
}
