use crate::parser::TokenInfo;

use super::parser::Term;


pub fn is_val(t: &Term) -> bool {
    match t {
        Term::True(_) | Term::False(_) => true,
        _ if is_numeric(t) => true,
        _ => false,
    }
}

fn is_numeric(t: &Term) -> bool {
    match t  {
        Term::Zero(_) => true,
        Term::Succ(_, inner) => is_numeric(inner.as_ref()),
        _ => false,
    }
}

pub fn evaluate(t: &Term) -> Term {
    match t {
        Term::Zero(_) | Term::True(_) | Term::False(_)
            => t.clone(),
        Term::If(_, cond, body, alt)
            => {
            match evaluate(cond) {
                Term::True(_) => evaluate(body),
                Term::False(_) => evaluate(alt),
                _ => panic!("If condition can only be true or false")
            }
        }
        Term::IsZero(info, val) => {
            match val.as_ref() {
                Term::Succ(_, inner) if is_numeric(inner) => Term::False(*info),
                Term::Zero(_) => Term::True(TokenInfo::default()),
                _ => {
                    if let Term::Zero(_) = evaluate(val) {
                        Term::True(TokenInfo::default())
                    } else {
                        Term::False(TokenInfo::default())
                    }
                }
            }
        }
        Term::Pred(info, val) => {
            match val.as_ref() {
                Term::Zero(_) => Term::Zero(*info),
                Term::Succ(_, inner) if is_numeric(inner) => {
                    evaluate(inner)
                }
                _ => evaluate(val)
            }
        }
        Term::Succ(info, val) => Term::Succ(*info, Box::new(evaluate(val.as_ref())))
    }
}