use std::io;
use std::io::{Write, BufWriter};
use std::rc::Rc;
use std::cell::RefCell;

use result::Result;

#[derive(Debug)]
pub enum Writer {
    StdOut(BufWriter<io::Stdout>),
    StdErr(BufWriter<io::Stderr>),
    Cursor(io::Cursor<Vec<u8>>),
}

impl Writer {
    pub fn write(w: &Rc<RefCell<Self>>, txt: &str) -> Result<()> {
        write!(w.borrow_mut(), "{}", txt)?;
        Ok(())
    }

    pub fn writeln(w: &Rc<RefCell<Self>>, txt: &str) -> Result<()> {
        writeln!(w.borrow_mut(), "{}", txt)?;
        Ok(())
    }

    pub fn flush(w: &Rc<RefCell<Self>>) -> Result<()> {
        w.borrow_mut().flush()?;
        Ok(())
    }
}

impl io::Write for Writer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        use output::Writer::*;

        match *self {
            StdOut(ref mut fd) => fd.write(buf),
            StdErr(ref mut fd) => fd.write(buf),
            Cursor(ref mut c) => c.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        use output::Writer::*;

        match *self {
            StdOut(ref mut fd) => fd.flush(),
            StdErr(ref mut fd) => fd.flush(),
            Cursor(ref mut c) => c.flush(),
        }
    }
}

pub enum Reader {
    StdIn(io::BufReader<io::Stdin>),
    Cursor(io::Cursor<Vec<u8>>),
}

impl io::Read for Reader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use output::Reader::*;

        match *self {
            StdIn(ref mut fd) => fd.read(buf),
            Cursor(ref mut c) => c.read(buf),
        }
    }
}

impl io::BufRead for Reader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        use output::Reader::*;

        match *self {
            StdIn(ref mut fd) => fd.fill_buf(),
            Cursor(ref mut c) => c.fill_buf(),
        }
    }

    fn consume(&mut self, amt: usize) {
        use output::Reader::*;

        match *self {
            StdIn(ref mut fd) => fd.consume(amt),
            Cursor(ref mut c) => c.consume(amt),
        }
    }
}
