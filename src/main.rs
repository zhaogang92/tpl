#[allow(unused)]
use std::fs;
use clap::Clap;

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
    let stmts = parser::parse(input.as_str()).expect("Parse faied.");
    print!("{:#?}", stmts);
}
