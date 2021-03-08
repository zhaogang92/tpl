#[allow(unused)]
use std::fs;
use clap::Clap;
use parser::print_term;

mod parser;
mod eval;

#[derive(Clap)]
#[clap(version = "0.1")]
struct Args {
    src: String,
}

fn main() {
    let args = Args::parse();
    let input = fs::read_to_string(args.src)
                            .expect("Failed to read the file.");
    let mut ctx = parser::init_context();
    let stmts = parser::parse(input.as_str(), &mut ctx).expect("Parse faied.");
    for stmt in stmts.iter() {
        parser::print_term(stmt.0.as_ref(), &ctx);
        println!(";");
    }
}
