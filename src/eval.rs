use crate::parser::{Context, print_term};
use crate::parser::Term;


pub fn is_val(ctx: &Context, t: &Term) -> bool {
    match t {
        Term::Abst(_, _, _, _) => true,
        Term::True(_) | Term::False(_) => true,
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
        Term::App(info, t1, t2) => {
            match t1.as_ref() {
                Term::Abst(_, x,  _, t12) if is_val(ctx, t2) => {
                    let res = subst_top(t2, t12);
                    return eval(ctx, &res);
                }
                _ if is_val(ctx, t1) => {
                    let t22 = eval(ctx, t2);
                    return eval(ctx, &Term::App(*info, t1.clone(), Box::new(t22)));
                }
                _ => {
                    let t12 = eval(ctx, t1);
                    return eval(ctx, &Term::App(*info, Box::new(t12), t2.clone()));
                }
            }
        }
        Term::If(_, cond, body, alt) => {
            let cond_val = eval(ctx, cond);
            match cond_val {
                Term::True(_) => eval(ctx, body),
                Term::False(_) => eval(ctx, alt),
                _ => panic!("If condition must be evaluted to boolean: {:?}", cond_val)
            }
        }
        _ => t.clone()
    }
}