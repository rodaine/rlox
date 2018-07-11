extern crate rlox;

use std::cell::RefCell;
use std::fs::File;
use std::io::{Cursor, SeekFrom};
use std::io::prelude::*;
use std::path::PathBuf;
use std::rc::Rc;
use std::string::String;

use rlox::run::Runner;
use rlox::output::Writer;

const TEST_DATA: &str = "testdata";

macro_rules! test_case {
    ($name:ident, $input:expr, $output:expr) => {
        #[test]
        fn $name() { run_golden_master($input, $output) }
    };
}

fn run_golden_master(input: &str, output: &str) {
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
            }
            _ => unreachable!(),
        };
    }

    assert_eq!(&expected, &actual)
}

test_case!(expr, "expr.lox", "expr.lox.out");
test_case!(brk, "break.lox", "break.lox.out");
test_case!(class, "class.lox", "class.lox.out");
test_case!(counter, "counter.lox", "counter.lox.out");
test_case!(loops, "loops.lox", "loops.lox.out");
test_case!(function, "function.lox", "function.lox.out");
test_case!(lambda, "lambda.lox", "lambda.lox.out");
test_case!(scopes, "scopes.lox", "scopes.lox.out");
test_case!(stmts, "stmts.lox", "stmts.lox.out");
test_case!(inheritance, "inheritance.lox", "inheritance.lox.out");
