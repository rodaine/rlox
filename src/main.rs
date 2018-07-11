extern crate rlox;

use std::env;
use std::io::{stdin, BufReader};
use std::path::Path;
use std::process::exit;

use rlox::{Result, Error};
use rlox::output::Reader::StdIn;
use rlox::run::Runner;

fn main() {
    let mut r = Runner::default();
    let args: Vec<String> = env::args().collect();

    let res: Result<()> = match args.len() {
        1 => r.prompt(StdIn(BufReader::new(stdin()))), // REPL if no script file
        2 => r.file(Path::new(&args[1])),                       // Interpret a file otherwise
        _ => Err(Error::Usage),                                      // Print usage
    };

    match res {
        Ok(_) => exit(0),
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
