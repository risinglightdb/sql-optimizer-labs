//! Expression simplification rules and constant folding.

use egg::rewrite as rw;

use super::*;

/// Returns all rules of expression simplification.
#[rustfmt::skip]
pub fn rules() -> Vec<Rewrite> { vec![
    rw!("add-zero";  "(+ ?a 0)" => "?a"),
    rw!("add-comm";  "(+ ?a ?b)" => "(+ ?b ?a)"),
    rw!("add-assoc"; "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
    rw!("add-same";  "(+ ?a ?a)" => "(* ?a 2)"),
    rw!("add-neg";   "(+ ?a (- ?b))" => "(- ?a ?b)"),

    rw!("mul-zero";  "(* ?a 0)" => "0"),
    rw!("mul-one";   "(* ?a 1)" => "?a"),
    rw!("mul-minus"; "(* ?a -1)" => "(- ?a)"),
    rw!("mul-comm";  "(* ?a ?b)"        => "(* ?b ?a)"),
    rw!("mul-assoc"; "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),

    rw!("neg-neg";    "(- (- ?a))" => "?a"),
    rw!("neg-sub";    "(- (- ?a ?b))" => "(- ?b ?a)"),

    rw!("sub-zero";   "(- ?a 0)" => "?a"),
    rw!("zero-sub";   "(- 0 ?a)" => "(- ?a)"),
    rw!("sub-cancel"; "(- ?a ?a)" => "0"),

    rw!("mul-add-distri";   "(* ?a (+ ?b ?c))" => "(+ (* ?a ?b) (* ?a ?c))"),
    rw!("mul-add-factor";   "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),

    rw!("eq-eq";     "(=  ?a ?a)" => "true"),
    rw!("ne-eq";     "(<> ?a ?a)" => "false"),
    rw!("gt-eq";     "(>  ?a ?a)" => "false"),
    rw!("lt-eq";     "(<  ?a ?a)" => "false"),
    rw!("ge-eq";     "(>= ?a ?a)" => "true"),
    rw!("le-eq";     "(<= ?a ?a)" => "true"),
    rw!("eq-comm";   "(=  ?a ?b)" => "(=  ?b ?a)"),
    rw!("ne-comm";   "(<> ?a ?b)" => "(<> ?b ?a)"),
    rw!("gt-comm";   "(>  ?a ?b)" => "(<  ?b ?a)"),
    rw!("lt-comm";   "(<  ?a ?b)" => "(>  ?b ?a)"),
    rw!("ge-comm";   "(>= ?a ?b)" => "(<= ?b ?a)"),
    rw!("le-comm";   "(<= ?a ?b)" => "(>= ?b ?a)"),
    rw!("eq-add";    "(=  (+ ?a ?b) ?c)" => "(=  ?a (- ?c ?b))"),
    rw!("ne-add";    "(<> (+ ?a ?b) ?c)" => "(<> ?a (- ?c ?b))"),
    rw!("gt-add";    "(>  (+ ?a ?b) ?c)" => "(>  ?a (- ?c ?b))"),
    rw!("lt-add";    "(<  (+ ?a ?b) ?c)" => "(<  ?a (- ?c ?b))"),
    rw!("ge-add";    "(>= (+ ?a ?b) ?c)" => "(>= ?a (- ?c ?b))"),
    rw!("le-add";    "(<= (+ ?a ?b) ?c)" => "(<= ?a (- ?c ?b))"),
    rw!("eq-trans";  "(and (= ?a ?b) (= ?b ?c))" => "(and (= ?a ?b) (= ?a ?c))"),

    rw!("not-eq";    "(not (=  ?a ?b))" => "(<> ?a ?b)"),
    rw!("not-ne";    "(not (<> ?a ?b))" => "(=  ?a ?b)"),
    rw!("not-gt";    "(not (>  ?a ?b))" => "(<= ?a ?b)"),
    rw!("not-ge";    "(not (>= ?a ?b))" => "(<  ?a ?b)"),
    rw!("not-lt";    "(not (<  ?a ?b))" => "(>= ?a ?b)"),
    rw!("not-le";    "(not (<= ?a ?b))" => "(>  ?a ?b)"),
    rw!("not-and";   "(not (and ?a ?b))" => "(or  (not ?a) (not ?b))"),
    rw!("not-or";    "(not (or  ?a ?b))" => "(and (not ?a) (not ?b))"),
    rw!("not-not";   "(not (not ?a))"    => "?a"),

    rw!("and-false"; "(and false ?a)"   => "false"),
    rw!("and-true";  "(and true ?a)"    => "?a"),
    rw!("and-null";  "(and null ?a)"    => "null"),
    rw!("and-same";  "(and ?a ?a)"      => "?a"),
    rw!("and-comm";  "(and ?a ?b)"      => "(and ?b ?a)"),
    rw!("and-not";   "(and ?a (not ?a))" => "false"),
    rw!("and-assoc"; "(and ?a (and ?b ?c))" => "(and (and ?a ?b) ?c)"),

    rw!("or-false";  "(or false ?a)" => "?a"),
    rw!("or-true";   "(or true ?a)"  => "true"),
    rw!("or-null";   "(or null ?a)"  => "null"),
    rw!("or-same";   "(or ?a ?a)"    => "?a"),
    rw!("or-comm";   "(or ?a ?b)"    => "(or ?b ?a)"),
    rw!("or-not";    "(or ?a (not ?a))"  => "true"),
    rw!("or-assoc";  "(or ?a (or ?b ?c))" => "(or (or ?a ?b) ?c)"),
    rw!("or-and";    "(or (and ?a ?b) (and ?a ?c))" => "(and ?a (or ?b ?c))"),

    rw!("xor-false"; "(xor false ?a)" => "?a"),
    rw!("xor-true";  "(xor true ?a)"  => "(not ?a)"),
    rw!("xor-null";  "(xor null ?a)"  => "null"),
    rw!("xor-same";  "(xor ?a ?a)"    => "false"),
    rw!("xor-comm";  "(xor ?a ?b)"    => "(xor ?b ?a)"),
    rw!("xor-not";   "(xor ?a (not ?a))"  => "true"),
    rw!("xor-assoc"; "(xor ?a (xor ?b ?c))" => "(xor (xor ?a ?b) ?c)"),
]}
