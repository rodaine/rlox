extern crate rlox;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::process::exit;
use rlox::{Result, Error};
use rlox::scanner::TokenIterator;

fn main() {
    let args: Vec<String> = env::args().collect();

    let res: Result<()> = match args.len() {
        1 => run_prompt(),         // REPL if no script file
        2 => run_file(&args[1]),   // Interpret a file otherwise
        _ => Err(Error::Usage.boxed()), // Print usage
    };

    match res {
        Ok(_) => exit(0),
        Err(e) => {
            println!("{}", e);
            exit(1);
        }
    }
}

/// Runs a script from file f and returns.
///
/// The entire script is buffered, but the resource is released immediately,
/// prior to running.
fn run_file(f: &str) -> Result<()> {
    let mut buf = String::new();
    { File::open(f)?.read_to_string(&mut buf)?; }
    run(&buf)
}

/// Runs an interactive prompt (REPL)
///
/// Each line is executed independently. Use ctrl+c to exit.
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

//
fn run(buf: &str) -> Result<()> {
    for res in buf.chars().tokens() {
        match res {
            Ok(t) => println!("{}", t),
            Err(e) => println!("{}", e),
        }
    }

    Ok(())
}
