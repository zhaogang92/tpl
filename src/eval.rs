use std::collections::HashMap;

use crate::parser::{Context};
use crate::parser::Term;


pub fn is_val(ctx: &Context, t: &Term) -> bool {
    match t {
        Term::Abst(_, _, _, _) => true,
        Term::True(_) | Term::False(_) => true,
        Term::Record(_, records) => {
            for (_, box item) in records {
                if !is_val(ctx, item) {
                    return false;
                }
            }
            true
        },
        _ => false,
    }
}

fn shift(d: i32, t: &Term) -> Term {
    fn walk(c: i32, d: i32, t: &Term) -> Term {
        match t {
            &Term::Var(info, k, l) => {
                if k >= c {
                    Term::Var(info, k+d, l+d)
                } else {
                    Term::Var(info, k, l+d)
                }
            }
            Term::Abst(info, x, ty_t1, body) => {
                Term::Abst(*info, x.to_string(), ty_t1.clone(), Box::new(walk(c+1, d, body)))
            }
            Term::App(info, t1, t2) => {
                Term::App(*info, Box::new(walk(c, d, t1)), Box::new(walk(c, d, t2)))
            },
            Term::If(info, t1, t2, t3) => {
                Term::If(*info, 
                    Box::new(walk(c, d, t1)),
                    Box::new(walk(c, d, t2)),
                    Box::new(walk(c, d, t3)),
                )
            }
            Term::True(_) | Term::False(_) => t.clone(),
            Term::Proj(info, box record, l) => {
                Term::Proj(*info, Box::new(walk(c, d, record)), l.to_string())
            }
            Term::Record(info, records) => {
                Term::Record(*info, records.iter().map(|(name, item)| {
                    (name.to_string(), Box::new(walk(c, d, item)))
                }).collect())
            }
            _ => unimplemented!("Not handled: {:?}", t)
        }
    }
    return walk(0, d, t);
}

fn subst(j: i32, s: &Term, t: &Term) -> Term {
    fn walk(c: i32, j: i32, s: &Term, t: &Term) -> Term {
        match t {
            &Term::Var(info, k, l) => {
                if k == j + c {
                    shift(c, s)
                } else {
                    Term::Var(info, k, l)
                }
            }
            Term::Abst(info, x, ty_t1, body) => {
                Term::Abst(*info, x.to_string(), ty_t1.clone(), Box::new(walk(c+1, j, s, body)))
            }
            Term::App(info, t1, t2) => {
                Term::App(*info, Box::new(walk(c, j, s, t1)), Box::new(walk(c, j, s, t2)))
            }
            Term::If(info, t1, t2, t3) => {
                Term::If(*info, 
                    Box::new(walk(c, j, s, t1)),
                    Box::new(walk(c, j, s, t2)),
                    Box::new(walk(c, j, s, t3)),
                )
            }
            Term::True(_) | Term::False(_) => t.clone(),
            Term::Proj(info, record, l) => {
                Term::Proj(*info, Box::new(walk(c, j, s, record)), l.to_string())
            }
            Term::Record(info, record) => {
                Term::Record(*info, record.iter().map(|(name, item)| {
                    (name.to_string(), Box::new(walk(c, j, s, item)))
                }).collect())
            }
            _ => unimplemented!("Not handled: {:?}", t)
        }
    }
    walk(0, j, s, t)
}

fn subst_top(s: &Term, t: &Term) -> Term {
    shift(-1, &subst(0, &shift(1, s), t))
}

pub fn eval(ctx: &Context, t: &Term) -> Term {
    match t {
        Term::App(_, box Term::Abst(_,_,_, box t12), box t2) if is_val(ctx, t2) => {
            let res = subst_top(t2, t12);
            eval(ctx, &res)
        }
        Term::App(info, box t1, box t2) if is_val(ctx, t1) => {
            let t22 = eval(ctx, t2);
            eval(ctx, &Term::App(*info, Box::new(t1.clone()), Box::new(t22)))
        }
        Term::App(info, box t1, box t2) => {
            let t12 = eval(ctx, t1);
            eval(ctx, &Term::App(*info, Box::new(t12), Box::new(t2.clone())))
        }
        Term::If(_, box cond, box body, box alt) => {
            let cond_val = eval(ctx, cond);
            match cond_val {
                Term::True(_) => eval(ctx, body),
                Term::False(_) => eval(ctx, alt),
                _ => panic!("If condition must be evaluted to boolean: {:?}", cond_val)
            }
        }
        Term::Record(info, records) => {
            if records.is_empty() {
                return t.clone()
            }
            let mut map = HashMap::new();
            for (name, box item) in records {
                if !is_val(ctx, item) {
                    let result = eval(ctx, item);
                    map.insert(name.to_string(), Box::new(result));
                } else {
                    map.insert(name.to_string(), Box::new(item.clone()));
                }
            }
            Term::Record(*info, map)
        }
        Term::Proj(_, box Term::Record(_, map), index) => {
            if let Some(box val) = map.get(index) {
                eval(ctx, val)
            } else {
                panic!("Cannot find key {} in the record", index)
            }
        }
        _ => t.clone()
    }
}