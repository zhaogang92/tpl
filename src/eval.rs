use super::parser::Term;


pub fn is_val(t: &Term) -> bool {
    match t {
        Term::True(_) | Term::False(_) => true,
        n if is_numeric(t) => true,
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

pub fn evaluate(t: &Term) {

}