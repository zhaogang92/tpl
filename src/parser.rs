use pest::Parser;
use pest::error::Error;
use pest::iterators::Pair;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "lambda.pest"]
struct LambdaParser;


#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    True,
    False,
    Zero,
    If(Box<Term>, Box<Term>, Box<Term>),
    IsZero(Box<Term>),
    Pred(Box<Term>),
    Succ(Box<Term>),
}

#[derive(Debug)]
pub struct Statement(Box<Term>);

pub fn parse(input: &str) -> Result<Vec<Statement>, Error<Rule>> {
    let mut res = Vec::new();
    let pairs = LambdaParser::parse(Rule::program, input)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::stmt => {
                let mut pair = pair.into_inner();
                let term = pair.next().unwrap();
                res.push(Statement(Box::new(parse_term(term))))
            }
            _ => {}
        }
    }
    Ok(res)
}

fn parse_term(pair: Pair<Rule>) -> Term {
    match pair.as_rule() {
        Rule::true_ty => Term::True,
        Rule::false_ty => Term::False,
        Rule::zero => Term::Zero,
        Rule::if_expr => {
            let mut it = pair.into_inner();
            let cond = parse_term(it.next().unwrap());
            let body = parse_term(it.next().unwrap());
            let alt = parse_term(it.next().unwrap());
            Term::If(Box::new(cond), Box::new(body), Box::new(alt))
        }
        Rule::succ => {
            let mut it = pair.into_inner();
            let val = parse_term(it.next().unwrap());
            Term::Succ(Box::new(val))
        }
        Rule::pred => {
            let mut it = pair.into_inner();
            let val = parse_term(it.next().unwrap());
            Term::Pred(Box::new(val))
        }
        Rule::iszero => {
            let mut it = pair.into_inner();
            let val = parse_term(it.next().unwrap());
            Term::IsZero(Box::new(val))
        }
        _ => panic!("Unexpected term: {}", pair.as_str())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        let input = r"
            /* Examples for testing */

            true;
            if false then true else false; 

            0; 
            succ (pred 0);
            iszero (pred (succ (succ 0))); 


            /* if 0 then 1 else pred 1; */
            if iszero (pred (succ 0)) then succ 0 else pred (succ 0); 
        ";
        let res = parse(input).unwrap();
        println!("{:#?}", res);
    }
}