extern crate rlox;

use std::env;
use std::io::{stdin, BufReader, BufRead};
use std::fs;

use rlox::vm;

fn main() -> vm::Result {
    let mut args = env::args();

    match args.len() {
        1 => repl(),
        2 => run_file(&(args.nth(1).unwrap())),
        _ => usage(),
    }
}


fn repl() -> vm::Result {
    let input = BufReader::new(stdin());
    print_cursor(1);

    for (line, src) in input.lines().enumerate() {
        vm::VM::interpret(src?, line+1)?;
        print_cursor(line+2);
    }

    Ok(())
}

fn print_cursor(line: usize) {
    eprint!("[{:03}]> ", line)
}

fn run_file(path: &str) -> vm::Result {
    let input = fs::read_to_string(path)?;
    vm::VM::interpret(input)
}

fn usage() -> vm::Result {
    eprintln!("Usage: rlox [path]");
    Ok(())
}
