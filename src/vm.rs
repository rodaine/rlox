use crate::chunk::Chunk;
use std::fmt;
use std::result;
use crate::value::{Value, Result as ValueResult, Error as ValueError};
use crate::compiler::Error as CompileError;
use std::io;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Compile(CompileError),
    Value(ValueError),
    Runtime,
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self { Error::IO(err) }
}

impl From<CompileError> for Error {
    fn from(err: CompileError) -> Self { Error::Compile(err) }
}

impl From<ValueError> for Error {
    fn from(err: ValueError) -> Self { Error::Value(err) }
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
            match inst.op {
                Unknown => return Err(Error::Runtime),
                Return => {
                    let v = self.pop()?;
                    if cfg!(feature = "debug-instructions") {
                        eprintln!("{:?}", self);
                    }
                    println!("{:?}", v);
                    return Ok(());
                }
                Constant8 | Constant16 | Constant24 => {
                    let c = self.chunk.read_const(Chunk::read_index(inst.data));
                    self.push(c);
                }

                True => self.push(Value::Bool(true)),
                False => self.push(Value::Bool(false)),
                Nil => self.push(Value::Nil),
                Not => self.run_unary_op(Value::any, Value::is_not)?,
                Negate => self.run_unary_op(Value::is_number, Value::negate)?,
                Add => self.run_binary_op(Value::both_any, Value::add)?,
                Subtract => self.run_binary_op(Value::both_numbers, Value::subtract)?,
                Multiply => self.run_binary_op(Value::both_numbers, Value::multiply)?,
                Divide => self.run_binary_op(Value::both_numbers, Value::divide)?,
                Equal => self.run_binary_op(Value::both_any, Value::equals)?,
                Greater => self.run_binary_op(Value::both_numbers, Value::greater_than)?,
                Less => self.run_binary_op(Value::both_numbers, Value::less_than)?
            }

            if cfg!(feature = "debug-instructions") {
                eprintln!("{:?}", self);
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
    fn run_unary_op<C, F>(&mut self, check: C, op: F) -> Result
        where
            C: FnOnce(&Value) -> ValueResult<()>,
            F: FnOnce(&Value) -> Value,
    {
        check(self.stack.last().ok_or(Error::Runtime)?)?;
        let v = self.stack.last_mut().ok_or(Error::Runtime)?;
        *v = op(v);
        Ok(())
    }

    #[inline(always)]
    fn run_binary_op<C, F>(&mut self, check: C, op: F) -> Result
        where
            C: FnOnce(&Value, &Value) -> ValueResult<()>,
            F: FnOnce(&Value, &Value) -> Value,
    {
        let split = self.stack.split_last().ok_or(Error::Runtime)?;
        let left = split.1.last().ok_or(Error::Runtime)?;
        let right = split.0;
        check(left, right)?;

        let b = self.pop()?;
        let v = self.stack.last_mut().ok_or(Error::Runtime)?;
        *v = op(v, &b);
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
