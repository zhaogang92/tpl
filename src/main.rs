#[allow(unused)]
use std::fs;
use clap::Clap;

mod parser;
mod eval;
mod ts;

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
        // let tt = eval(&mut ctx, stmt.0.as_ref());
        parser::print_term(&stmt.0, &mut ctx);
        // parser::print_term(stmt.0.as_ref(), &mut ctx);
        println!(";");
    }
}
