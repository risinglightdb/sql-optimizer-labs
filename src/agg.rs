use egg::Language;

use super::*;

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
    todo!()
}
