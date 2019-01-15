use crate::chunk::Chunk;
use std::fmt;
use std::ops::*;
use std::result;
use crate::value::Value;
use crate::compiler::{compile, Error as CompileError};
use std::rc::Rc;
use std::io;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Compile(CompileError),
    Runtime,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self { Error::IO(err) }
}

impl From<CompileError> for Error {
    fn from(err: CompileError) -> Self { Error::Compile(err) }
}

pub type Result = result::Result<(), Error>;

pub struct VM<'a> {
    chunk: &'a Chunk,
    ip: usize,
    stack: Vec<Value>,
}

impl<'a> VM<'a> {
    pub fn interpret(chunk: &'a Chunk) -> Result {
        Self {
            chunk,
            ip: 0,
            stack: Vec::new(),
        }.run()
    }

    #[allow(dead_code)]
    fn run(&mut self) -> Result {
        use crate::chunk::OpCode::*;

        while let Some(inst) = self.chunk.read(self.ip) {
            if cfg!(feature = "debug-instructions") {
                eprintln!("{:?}", self);
            }

            match inst.op {
                Unknown => return Err(Error::Runtime),
                Return => {
                    let v = self.pop()?;
                    println!("{:?}", v);
                    return Ok(());
                }
                Constant8 | Constant16 | Constant24 => {
                    let c = self.chunk.read_const(Chunk::read_index(inst.data));
                    self.push(c);
                }
                Negate => self.run_unary_op(Value::neg)?,
                Add => self.run_binary_op(Value::add)?,
                Subtract => self.run_binary_op(Value::sub)?,
                Multiply => self.run_binary_op(Value::mul)?,
                Divide => self.run_binary_op(Value::div)?,
            }
            self.ip += inst.len()
        }

        Ok(())
    }

    #[inline(always)]
    fn push(&mut self, v: Value) {
        self.stack.push(v)
    }

    #[inline(always)]
    fn pop(&mut self) -> result::Result<Value, Error> {
        self.stack.pop().ok_or(Error::Runtime)
    }

    #[inline(always)]
    fn run_unary_op<F>(&mut self, op: F) -> Result
        where
            F: FnOnce(Value) -> Value,
    {
        let v = self.stack.last_mut().ok_or(Error::Runtime)?;
        *v = op(*v);
        Ok(())
    }

    #[inline(always)]
    fn run_binary_op<F>(&mut self, op: F) -> Result
        where
            F: FnOnce(Value, Value) -> Value,
    {
        let b = self.pop()?;
        let v = self.stack.last_mut().ok_or(Error::Runtime)?;
        *v = op(*v, b);
        Ok(())
    }
}

impl<'a> fmt::Debug for VM<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.chunk.debug_inst(f, self.ip, 0)?;
        write!(f, "\ts:{:?}", self.stack)?;
        Ok(())
    }
}
