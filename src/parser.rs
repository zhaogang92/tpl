use pest::{Parser, Span};
use pest::error::Error;
use pest::iterators::Pair;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "lambda.pest"]
struct LambdaParser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    index: usize,   // byte pos
    line_col: (usize, usize)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenInfo(Position, Position);


#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    True(TokenInfo),
    False(TokenInfo),
    Zero(TokenInfo),
    If(TokenInfo, Box<Term>, Box<Term>, Box<Term>),
    IsZero(TokenInfo, Box<Term>),
    Pred(TokenInfo, Box<Term>),
    Succ(TokenInfo, Box<Term>),
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
    let token_info = get_token_info(pair.as_span());
    match pair.as_rule() {
        Rule::true_ty => Term::True(token_info),
        Rule::false_ty => Term::False(token_info),
        Rule::zero => Term::Zero(token_info),
        Rule::if_expr => {
            let mut it = pair.into_inner();
            let cond = parse_term(it.next().unwrap());
            let body = parse_term(it.next().unwrap());
            let alt = parse_term(it.next().unwrap());
            Term::If(token_info, Box::new(cond), Box::new(body), Box::new(alt))
        }
        Rule::succ => {
            let mut it = pair.into_inner();
            let val = parse_term(it.next().unwrap());
            Term::Succ(token_info, Box::new(val))
        }
        Rule::pred => {
            let mut it = pair.into_inner();
            let val = parse_term(it.next().unwrap());
            Term::Pred(token_info, Box::new(val))
        }
        Rule::iszero => {
            let mut it = pair.into_inner();
            let val = parse_term(it.next().unwrap());
            Term::IsZero(token_info, Box::new(val))
        }
        _ => panic!("Unexpected term: {}", pair.as_str())
    }
}

fn get_token_info(span: Span) -> TokenInfo {
    TokenInfo(
        Position {
            index: span.start(),
            line_col: span.start_pos().line_col()
        },
        Position {
            index: span.end(),
            line_col: span.end_pos().line_col()
        }
    )
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