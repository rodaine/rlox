#![feature(proc_macro)]
extern crate test_case_derive;
extern crate rlox;

use test_case_derive::test_case;

use std::cell::RefCell;
use std::fs::File;
use std::io::{Cursor, SeekFrom};
#[allow(unused_imports)]
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::string::String;

use rlox::run::Runner;
use rlox::output::Writer;

const TEST_DATA: &str = "testdata";

#[test_case("expr.lox", "expr.lox.out")]
#[test_case("break.lox", "break.lox.out")]
#[test_case("class.lox", "class.lox.out")]
#[test_case("counter.lox", "counter.lox.out")]
#[test_case("loops.lox", "loops.lox.out")]
#[test_case("function.lox", "function.lox.out")]
#[test_case("lambda.lox", "lambda.lox.out")]
#[test_case("scopes.lox", "scopes.lox.out")]
#[test_case("stmts.lox", "stmts.lox.out")]
#[test_case("inheritance.lox", "inheritance.lox.out")]
fn golden_masters(input: &str, output: &str) {
    let i: PathBuf = [TEST_DATA, input].iter().collect();
    let o: PathBuf = [TEST_DATA, output].iter().collect();

    let stdout =
        Rc::new(RefCell::new(Writer::Cursor(Cursor::new(Vec::new()))));
    let stderr =
        Rc::new(RefCell::new(Writer::Cursor(Cursor::new(Vec::new()))));

    {
        let mut r = Runner::new(Rc::clone(&stdout), Rc::clone(&stderr));
        r.file(&i).expect("file should interpret successfully");
    }

    let mut expected = String::new();
    {
        File::open(&o)
            .expect("failed to open output file")
            .read_to_string(&mut expected)
            .expect("failed to read output file");
    }


    let mut actual = String::new();
    {
        match Rc::try_unwrap(stdout)
            .expect("unable to unwrap stdout")
            .into_inner() {
            Writer::Cursor(ref mut c) => {
                c.seek(SeekFrom::Start(0)).expect("cannot seek to head of cursor");
                c.read_to_string(&mut actual).expect("cannot read actual output");
            },
            _ => unreachable!(),
        };
    }

    assert_eq!(&expected, &actual)
}
