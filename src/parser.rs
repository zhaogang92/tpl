use std::{collections::{HashMap, VecDeque}, fmt::{Display, Formatter}, usize};
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
    True(TokenInfo),
    False(TokenInfo),
    Zero(TokenInfo),
    Pred(TokenInfo, Box<Term>),
    Succ(TokenInfo, Box<Term>),
    IsZero(TokenInfo, Box<Term>),
    If(TokenInfo, Box<Term>, Box<Term>, Box<Term>),
    Var(TokenInfo, i32, i32),
    App(TokenInfo, Box<Term>, Box<Term>),
    Abst(TokenInfo, String, Type, Box<Term>),
    Proj(TokenInfo, Box<Term>, String),
    Record(TokenInfo, HashMap<String, Box<Term>>)
}

#[derive(Debug)]
pub struct Statement(pub Box<Term>);

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    TyBool,
    TyArr(Box<Type>, Box<Type>),
    TyTop,
    TyRecord(HashMap<String, Box<Type>>)
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::TyBool => write!(f, "Bool"),
            Type::TyArr(ty1, ty2) => {
                write!(f, "{}->{}", ty1, ty2)
            },
            Type::TyTop => write!(f, "Top"),
            Type::TyRecord(elements) => {
                write!(f, "{{").unwrap();
                for (name, box ty) in elements {
                    write!(f, "{}: {},", name, ty).unwrap();
                }
                write!(f, "}}")
            }
        } 
    }
}

#[derive(Debug, Clone)]
pub enum Bind {
    NameBind,
    VarBind(Type)
}

#[derive(Debug, Clone)]
pub struct VarContext(pub String, pub Bind);

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
        Rule::TRUE => Term::True(token_info),
        Rule::FALSE => Term::False(token_info),
        Rule::ZERO => Term::Zero(token_info),
        Rule::iszero => {
            let mut pair = pair.into_inner();
            let t = parse_term(pair.next().unwrap(), ctx);
            Term::IsZero(token_info, Box::new(t))
        }
        Rule::pred => {
            let mut pair = pair.into_inner();
            let t = parse_term(pair.next().unwrap(), ctx);
            Term::Pred(token_info, Box::new(t))
        }
        Rule::succ => {
            let mut pair = pair.into_inner();
            let t = parse_term(pair.next().unwrap(), ctx);
            Term::Succ(token_info, Box::new(t))
        }
        Rule::proj => {
            let mut pair = pair.into_inner();
            let record_p = pair.next().unwrap();
            let index = pair.next().unwrap().as_str();
            let record = parse_term(record_p, ctx);
            Term::Proj(token_info, Box::new(record), index.to_string()) 
        }
        Rule::record => {
            let map = parse_record(pair, ctx);
            Term::Record(token_info, map)
        }
        Rule::var =>  {
            let var = pair.as_str();
            let var_index = name2index(ctx, var);
            Term::Var(token_info, var_index, ctx.len() as i32)
        }
        Rule::if_expr => {
            let mut pair = pair.into_inner();
            let cond = parse_term(pair.next().unwrap(), ctx);
            let body = parse_term(pair.next().unwrap(), ctx);
            let alt = parse_term(pair.next().unwrap(), ctx);
            Term::If(token_info, Box::new(cond), Box::new(body), Box::new(alt))
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
            let ty = parse_ty(pair.next().unwrap(), ctx);
            add_bind(ctx, var.as_str(), Bind::VarBind(ty.clone()));
            let body = parse_term(pair.next().unwrap(), ctx);
            pop_bind(ctx);
            Term::Abst(token_info,
                var.as_str().to_string(),
                ty,
                Box::new(body)
            )
        }
        _ => panic!("Unexpected term: {}", pair.as_str())
    }
}

fn parse_record_item(pair: Pair<Rule>, ctx: &mut Context) -> (String, Box<Term>) {
    let mut pair = pair.into_inner();
    let name = pair.next().unwrap().as_str();
    let value = parse_term(pair.next().unwrap(), ctx);
    (name.to_string(), Box::new(value))
}

fn parse_record(pair: Pair<Rule>, ctx: &mut Context) -> HashMap<String, Box<Term>> {
    let mut pair = pair.into_inner();
    let mut map = HashMap::new();
    while let Some(p) = pair.next() {
        let (name, val) = parse_record_item(p, ctx);
        map.insert(name, val);
    }
    map
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

fn parse_ty(pair: Pair<Rule>, ctx: &mut Context) -> Type {
    match pair.as_rule() {
        Rule::basic_ty => {
            if pair.as_str() == "Bool" {
                Type::TyBool
            } else {
                Type::TyTop
            }
        }
        Rule::lambda_ty => {
            let mut pair = pair.into_inner();
            let arg_ty = parse_ty(pair.next().unwrap(), ctx);
            let ret_ty = parse_ty(pair.next().unwrap(), ctx);
            Type::TyArr(Box::new(arg_ty), Box::new(ret_ty))
        }
        Rule::record_ty => {
            let mut pair = pair.into_inner();
            let mut map = HashMap::new();
            while let Some(p) = pair.next() {
                let mut item = p.into_inner();
                let name = item.next().unwrap().as_str();
                let item_ty = parse_ty(item.next().unwrap(), ctx);
                map.insert(name.to_string(), Box::new(item_ty));
            }
            Type::TyRecord(map)
        }
        _ => panic!("Not type token: {}", pair.as_str())
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
    add_bind(ctx, x, Bind::NameBind)    // probably outer var, and they will stay there forever
}

pub fn index2name(ctx: &mut Context, x: i32) -> &str {
    let item = ctx.get(x as usize).unwrap();
    item.0.as_ref()
}

pub fn add_bind(ctx: &mut Context, x: &str, b: Bind) -> i32 {
    let bind = VarContext(x.to_string(), b);
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
    add_bind(ctx, var, Bind::NameBind);
    var.to_string()
}

pub fn print_term(t: &Term, ctx: &mut Context) {
    match t {
        Term::True(_) => print!("true"),
        Term::False(_) => print!("false"),
        Term::Var(_, idx, _) => {
            let x = index2name(ctx, *idx);
            print!("{}", x);
        }
        Term::Abst(_, var, ty,  body) => {
            let name = pick_fresh_name(ctx, var);
            print!("lambda {}:{}. ", name, ty);
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
        Term::If(_, cond, body, alt) => {
            print!("if ");
            print_term(cond, ctx);
            print!(" then ");
            print_term(body, ctx);
            print!(" else ");
            print_term(alt, ctx);
        }
        Term::Record(_, map) => {
            print!("{{");
            for (name, box value) in map {
                print!("{}=", name);
                print_term(value, ctx);
                print!(",");
            }
            print!("}}");
        }
        Term::Proj(_, box record, index) => {
            print_term(record, ctx);
            print!(".{}", index);
        }
        _ => unimplemented!(),
    }
}