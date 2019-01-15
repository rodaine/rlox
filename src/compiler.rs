use crate::scanner::Scanner;
use std::rc::Rc;
use std::result;

use crate::chunk::Chunk;

#[derive(Debug, Copy, Clone)]
pub enum Error {}

type Result = result::Result<Chunk, Error>;

pub fn compile(source: &Rc<String>, line: usize) -> Result {
    let scanner = Scanner::new_from_line(source, line);
    unimplemented!()
}
