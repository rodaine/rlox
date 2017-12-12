extern crate rlox;

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::io::stderr;
use std::process::exit;

use rlox::{Result, Error};
use rlox::scanner::TokenIterator;
use rlox::parser::StmtIterator;
use rlox::interpreter::Interpreter;
use rlox::resolver::Resolver;

fn main() {
    let args: Vec<String> = env::args().collect();

    let res: Result<()> = match args.len() {
        1 => run_prompt(),         // REPL if no script file
        2 => run_file(&args[1]),   // Interpret a file otherwise
        _ => Err(Error::Usage), // Print usage
    };

    match res {
        Ok(_) => exit(0),
        Err(e) => {
            writeln!(&mut stderr(), "{}", e).expect("problem writing to stderr");
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
    run(&mut Interpreter::new(false), &buf).map(|_| ())
}

/// Runs an interactive prompt (REPL)
///
/// Each line is executed independently. Use ctrl+c to exit.
fn run_prompt() -> Result<()> {
    let mut buf = String::new();
    let mut i = Interpreter::new(true);

    println!("RLOX : Press ctrl+c to exit");
    loop {
        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut buf)?;
        if let Err(e) = run(&mut i, &format!("{};", buf)) { println!("{}", e); }
        buf.clear()
    }
}

fn run(i: &mut Interpreter, buf: &str) -> Result<()> {
    for res in buf.chars().tokens().statements() {
        match res {
            Err(e) => writeln!(&mut stderr(), "{}", e)?,
            Ok(stmt) => Resolver::resolve(i, &stmt)?.interpret(&stmt)?,
        }
    }
    Ok(())
}
