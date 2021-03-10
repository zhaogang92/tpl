use std::{collections::VecDeque, usize};
use pest::{Parser, Span};
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
    Var(TokenInfo, i32, i32),
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
            Term::Var(token_info, var_index, ctx.len() as i32)
        }
        Rule::app => {
            let mut pair = pair.into_inner();
            let t1 = parse_app_template(pair.next().unwrap(), ctx);
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
            pop_bind(ctx);
            Term::Abst(token_info,
                var.as_str().to_string(),
                Box::new(body)
            )
        }
        _ => panic!("Unexpected term: {}", pair.as_str())
    }
}

fn parse_app_template(pair: Pair<Rule>, ctx: &mut Context) -> Term {
    let token_info = get_token_info(pair.as_span());
    match pair.as_rule() {
        Rule::var => {
            let var = pair.as_str();
            let var_index = name2index(ctx, var);
            Term::Var(token_info, var_index, ctx.len() as i32)
        }
        _ => {
            parse_term(pair, ctx)
        }
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

fn name2index(ctx: &mut Context, x: &str) -> i32 {
    for (idx, v) in ctx.iter().enumerate() {
        if v.0 == x {
            return idx as i32;
        }
    }
    add_bind(ctx, x)    // probably outer var, and they will stay there forever
}

fn index2name(ctx: &mut Context, x: i32) -> &str {
    let item = ctx.get(x as usize).unwrap();
    item.0.as_ref()
}

fn add_bind(ctx: &mut Context, x: &str) -> i32 {
    let bind = VarContext(x.to_string(), Bind());
    ctx.push_front(bind);
    0
}

/// After a lambda ends, pop its bind
fn pop_bind(ctx: &mut Context) {
    ctx.pop_front();
}

fn pick_fresh_name(ctx: &mut Context, var: &str) -> String {
    for VarContext(x, _) in ctx.iter() {
        if x == var {
            return pick_fresh_name(ctx, format!("{}'", var).as_str());
        }
    }
    add_bind(ctx, var);
    var.to_string()
}

pub fn print_term(t: &Term, ctx: &mut Context) {
    match t {
        Term::Var(_, idx, _) => {
            let x = index2name(ctx, *idx);
            print!("{}", x);
        }
        Term::Abst(_, var, body) => {
            let name = pick_fresh_name(ctx, var);
            print!("lambda {}. ", name);
            print_term(body, ctx);
            pop_bind(ctx);
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