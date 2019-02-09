use crate::chunk::{self, Chunk};
use std::fmt;
use std::result;
use crate::value::{Value, Object, Result as ValueResult, Error as ValueError};
use crate::compiler::Error as CompileError;
use std::io;
use std::collections::HashMap;
use crate::token::Lexeme;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    Compile(CompileError),
    Value(ValueError),
    Runtime,
    UndefinedVariable(Lexeme),
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

pub struct VM {
    stack: Vec<Value>,
    globals: HashMap<Lexeme, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, chunk: &Chunk) -> Result {
        VMExecution {
            chunk,
            ip: 0,
            state: self,
        }.run()
    }
}

struct VMExecution<'a> {
    chunk: &'a Chunk,
    ip: usize,
    state: &'a mut VM,
}

impl<'a> VMExecution<'a> {
    fn run(&mut self) -> Result {
        use crate::chunk::OpCode::*;

        while let Some(inst) = self.chunk.read(self.ip) {
            if cfg!(feature = "debug-instructions") {
                eprintln!("{:?}", self);
            }

            match inst.op {
                Unknown => return Err(Error::Runtime),
                Return => {
                    if cfg!(feature = "debug-instructions") {
                        eprintln!("{:?}", self);
                    }
                    return Ok(());
                }
                Constant8 | Constant16 | Constant24 => {
                    let c = self.chunk.read_const(chunk::bytes_to_usize(inst.data));
                    self.push(c);
                }
                DefineGlobal8 | DefineGlobal16 | DefineGlobal24 => {
                    let name = self.chunk.read_const(chunk::bytes_to_usize(inst.data)).into_lex();
                    let v = self.pop()?;
                    self.state.globals.insert(name, v);
                }
                GetGlobal8 | GetGlobal16 | GetGlobal24 => {
                    let name = self.chunk.read_const(chunk::bytes_to_usize(inst.data));
                    let lex = name.lex();
                    let val =  self.state.globals.get(lex)
                        .ok_or_else(|| Error::UndefinedVariable(lex.clone()))?;
                    self.push(val.clone());
                }
                SetGlobal8 | SetGlobal16 | SetGlobal24 => {
                    let name = self.chunk.read_const(chunk::bytes_to_usize(inst.data)).into_lex();
                    let val = self.peek()?;
                    if !self.state.globals.contains_key(&name) {
                        return Err(Error::UndefinedVariable(name));
                    }
                    self.state.globals.insert(name, val.clone());
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
                Less => self.run_binary_op(Value::both_numbers, Value::less_than)?,
                Print => println!("{:?}", self.pop()?),
                Pop => { self.pop()?; }
            }

            self.ip += inst.len()
        }

        Ok(())
    }

    #[inline(always)]
    fn push(&mut self, v: Value) {
        self.state.stack.push(v)
    }

    #[inline(always)]
    fn pop(&mut self) -> result::Result<Value, Error> {
        self.state.stack.pop().ok_or(Error::Runtime)
    }

    fn peek(&self) -> result::Result<&Value, Error> {
        self.state.stack.last().ok_or(Error::Runtime)
    }

    #[inline(always)]
    fn run_unary_op<C, F>(&mut self, check: C, op: F) -> Result
        where
            C: FnOnce(&Value) -> ValueResult<()>,
            F: FnOnce(&Value) -> Value,
    {
        check(self.state.stack.last().ok_or(Error::Runtime)?)?;
        let v = self.state.stack.last_mut().ok_or(Error::Runtime)?;
        *v = op(v);
        Ok(())
    }

    #[inline(always)]
    fn run_binary_op<C, F>(&mut self, check: C, op: F) -> Result
        where
            C: FnOnce(&Value, &Value) -> ValueResult<()>,
            F: FnOnce(&Value, &Value) -> Value,
    {
        let split = self.state.stack.split_last().ok_or(Error::Runtime)?;
        let left = split.1.last().ok_or(Error::Runtime)?;
        let right = split.0;
        check(left, right)?;

        let b = self.pop()?;
        let v = self.state.stack.last_mut().ok_or(Error::Runtime)?;
        *v = op(v, &b);
        Ok(())
    }
}

impl<'a> fmt::Debug for VMExecution<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.chunk.debug_inst(f, self.ip, 0)?;
        write!(f, "\ts:{:?} g:{:?}", self.state.stack, self.state.globals)?;
        Ok(())
    }
}
