use crate::scanner::Scanner;
use std::rc::Rc;

pub fn compile(source: &Rc<String>) {
   compile_from_line(source, 1);
}

pub fn compile_from_line(source: &Rc<String>, line: usize) {
    let scanner = Scanner::new_from_line(source, line);
    scanner.debug()
}
