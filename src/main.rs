#[macro_use]
extern crate lazy_static;

mod result;
mod token;
mod scanner;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::process::exit;
use result::Result;
use scanner::TokenIterator;

fn main() {
    use result::Error::*;

    let args: Vec<String> = env::args().collect();

    let res: Result<()> = match args.len() {
        1 => run_prompt(),         // REPL if no script file
        2 => run_file(&args[1]),   // Interpret a file otherwise
        _ => Err(Box::new(Usage)), // Print usage
    };

    match res {
        Ok(_) => exit(0),
        Err(e) => {
            println!("{}", e);
            exit(1);
        },
    }
}

fn run_file(f: &str) -> Result<()> {
    let mut buf = String::new();
    {
        File::open(f)?.read_to_string(&mut buf)?;
    }

    run(&buf)
}

fn run_prompt() -> Result<()> {
    let mut buf = String::new();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        io::stdin().read_line(&mut buf)?;
        run(&buf).unwrap();

        buf.clear()
    }
}

fn run(buf: &str) -> Result<()> {
    let mut tokens = buf.chars().tokens();

    while let Some(res) = tokens.next() {
        match res {
            Ok(t) => println!("{}", t),
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}
