use sql_optimizer_labs::{expr, plan, Rewrite};

fn rules() -> Vec<Rewrite> {
    let mut rules = vec![];
    rules.extend(expr::rules());
    rules.extend(plan::rules());
    rules
}

egg::test_fn! {
    predicate_pushdown,
    rules(),
    // SELECT s.name, e.cid
    // FROM student AS s, enrolled AS e
    // WHERE s.sid = e.sid AND e.grade = 'A' AND s.name <> 'Alice'
    "
    (proj (list s.name e.cid)
    (filter (and (and (= s.sid e.sid) (= e.grade 'A')) (<> s.name 'Alice'))
    (join inner true
        (scan s (list s.sid s.name))
        (scan e (list e.sid e.cid e.grade))
    )))" => "
    (proj (list s.name e.cid)
    (join inner (= s.sid e.sid)
        (filter (<> s.name 'Alice')
            (scan s (list s.sid s.name)))
        (filter (= e.grade 'A')
            (scan e (list e.sid e.cid e.grade)))
    ))"
}

egg::test_fn! {
    join_reorder,
    rules(),
    // SELECT * FROM t1
    // INNER JOIN t2 ON t1.id = t2.id
    // INNER JOIN t3 ON t3.id = t2.id
    "
    (join inner (= t3.id t2.id)
        (join inner (= t1.id t2.id)
            (scan t1 (list t1.id t1.a))
            (scan t2 (list t2.id t2.b))
        )
        (scan t3 (list t3.id t3.c))
    )" => "
    (join inner (= t1.id t2.id)
        (scan t1 (list t1.id t1.a))
        (join inner (= t2.id t3.id)
            (scan t2 (list t2.id t2.b))
            (scan t3 (list t3.id t3.c))
        )
    )"
}

egg::test_fn! {
    hash_join,
    rules(),
    // SELECT * FROM t1, t2
    // WHERE t1.id = t2.id AND t1.age > 2
    "
    (filter (and (= t1.id t2.id) (> t1.age 2))
    (join inner true
        (scan t1 (list t1.id t1.age))
        (scan t2 (list t2.id t2.name))
    ))" => "
    (hashjoin inner (list t1.id) (list t2.id)
        (filter (> t1.age 2)
            (scan t1 (list t1.id t1.age))
        )
        (scan t2 (list t2.id t2.name))
    )"
}
