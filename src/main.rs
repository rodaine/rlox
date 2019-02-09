extern crate rlox;

use std::env;
use std::io::{stdin, BufReader, BufRead};
use std::fs;
use std::rc::Rc;

use rlox::vm;
use rlox::compiler::Compiler;

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

    let mut vm = vm::VM::new();

    for (line, src) in input.lines().enumerate() {
        let source = Rc::new(src?);
        let chunk = Compiler::new(&source, line + 1).compile()?;
        vm.interpret(&chunk)?;
        print_cursor(line + 2);
    }

    Ok(())
}

fn print_cursor(line: usize) {
    eprint!("[{:03}]> ", line)
}

fn run_file(path: &str) -> vm::Result {
    let source = Rc::new(fs::read_to_string(path)?);
    let chunk = Compiler::new(&source, 1).compile()?;
    vm::VM::new().interpret(&chunk)
}

fn usage() -> vm::Result {
    eprintln!("Usage: rlox [path]");
    Ok(())
}
