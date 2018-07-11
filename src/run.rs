use std::fs::File;
use std::io::prelude::*;
use std::io;
use std::path::Path;
use std::rc::Rc;
use std::cell::RefCell;

use interpreter::Interpreter;
use output::{Writer, Reader};
use parser::StmtIterator;
use resolver::Resolver;
use result::Result;
use scanner::TokenIterator;
use debug::time;

pub struct Runner {
    stdout: Rc<RefCell<Writer>>,
    stderr: Rc<RefCell<Writer>>,
}

impl Default for Runner {
    fn default() -> Self {
        Runner::new(
            Rc::new(RefCell::new(Writer::StdOut(io::BufWriter::new(io::stdout())))),
            Rc::new(RefCell::new(Writer::StdErr(io::BufWriter::new(io::stderr())))),
        )
    }
}

impl Runner {
    pub fn new(stdout: Rc<RefCell<Writer>>, stderr: Rc<RefCell<Writer>>) -> Self {
        Runner {
            stdout,
            stderr,
        }
    }

    pub fn file(&mut self, f: &Path) -> Result<()> {
        let mut src = String::new();

        time("read file", ||
            File::open(f).and_then(|mut h| h.read_to_string(&mut src)))?;

        let stdout = Rc::clone(&self.stdout);
        let mut i = Interpreter::new(false, stdout);

        time("total run", || { self.run(&mut i, &src) })
    }

    pub fn prompt(&mut self, mut stdin: Reader) -> Result<()> {
        let mut src = String::new();
        let stdout = Rc::clone(&self.stdout);
        let mut i = Interpreter::new(true, stdout);

        Writer::writeln(&self.stdout, "RLOX : Press ctrl+c to exit")?;
        loop {
            Writer::write(&self.stdout, "> ")?;
            Writer::flush(&self.stdout)?;
            stdin.read_line(&mut src)?;

            if let Some(c) = src.pop() {
                if c == ';' {
                    src.push(c);
                } else {
                    src.push(c);
                    src.push(';');
                }
            }

            if let Err(e) = time("line run", || self.run(&mut i, &src)) {
                Writer::writeln(&self.stderr, &format!("{}", e))?;
                Writer::flush(&self.stderr)?;
            }

            src.clear();
        }
    }

    pub fn run(&mut self, i: &mut Interpreter, src: &str) -> Result<()> {
        for res in src.chars().tokens().statements() {
            match res {
                Err(e) => Writer::write(&self.stderr, &format!("{}", e))?,
                Ok(stmt) => {
                    let i = time("resolve", || Resolver::resolve(i, &stmt))?;
                    time("interpret", || stmt.accept(i))?
                }
            }
            Writer::flush(&self.stdout)?;
            Writer::flush(&self.stderr)?;
        }
        Ok(())
    }
}
