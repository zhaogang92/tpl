use std::{collections::VecDeque, fmt::Display};
use pest::{Parser, Span, Token};
use pest::error::Error;
use pest::iterators::Pair;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "lambda.pest"]
struct LambdaParser;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    index: usize,   // byte pos
    line_col: (usize, usize)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TokenInfo(Position, Position);


#[derive(Debug, PartialEq, Clone)]
pub enum Term {
    Var(TokenInfo, usize, usize),
    App(TokenInfo, Box<Term>, Box<Term>),
    Abst(TokenInfo, String, Box<Term>)
}

#[derive(Debug)]
pub struct Statement(pub Box<Term>);

#[derive(Debug, Clone)]
pub struct Bind();

#[derive(Debug, Clone)]
pub struct VarContext(String, Bind);

pub type Context = VecDeque<VarContext>;

pub fn init_context() -> Context {
    VecDeque::default()
}

pub fn parse(input: &str, ctx: &mut Context) -> Result<Vec<Statement>, Error<Rule>> {
    let mut res = Vec::new();
    let pairs = LambdaParser::parse(Rule::program, input)?;
    for pair in pairs {
        match pair.as_rule() {
            Rule::stmt => {
                let mut pair = pair.into_inner();
                let term = pair.next().unwrap();
                res.push(Statement(Box::new(parse_term(term, ctx))))
            }
            _ => {}
        }
    }
    Ok(res)
}

fn parse_term(pair: Pair<Rule>, ctx: &mut Context) -> Term {
    let token_info = get_token_info(pair.as_span());
    match pair.as_rule() {
        Rule::var =>  {
            let var = pair.as_str();
            let var_index = name2index(ctx, var);
            Term::Var(token_info, var_index, ctx.len())
        }
        Rule::app => {
            let mut pair = pair.into_inner();
            let t1 = parse_term(pair.next().unwrap(), ctx);
            let t2 = parse_term(pair.next().unwrap(), ctx);
            Term::App(
                token_info,
                Box::new(t1),
                Box::new(t2)
            ) 
        }
        Rule::abst => {
            let mut pair = pair.into_inner();
            let var = pair.next().unwrap();
            add_bind(ctx, var.as_str());
            let body = parse_term(pair.next().unwrap(), ctx);
            Term::Abst(token_info,
                var.as_str().to_string(),
                Box::new(body)
            )
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

fn name2index(ctx: &mut Context, x: &str) -> usize {
    for (idx, v) in ctx.iter().enumerate() {
        if v.0 == x {
            return idx;
        }
    }
    add_bind(ctx, x)    // probably outer var
}

fn add_bind(ctx: &mut Context, x: &str) -> usize {
    let bind = VarContext(x.to_string(), Bind());
    ctx.push_front(bind);
    0
}

fn pick_fresh_name(ctx: &mut Context, var: &str) -> String {
    unimplemented!()
}

pub fn print_term(t: &Term, ctx: &VecDeque<VarContext>) {
    match t {
        Term::Var(_, idx, _) => {
            // let name = ctx.get(*idx).unwrap().0.as_str();
            print!("{}", idx);
        }
        Term::Abst(_, var, body) => {
            print!("lambda {}. ", var);
            print_term(body, ctx);
        }
        Term::App(_, f, arg) => {
            print!("(");
            print_term(f, ctx);
            print!(") (");
            print_term(arg, ctx);
            print!(")");
        }
    }
}