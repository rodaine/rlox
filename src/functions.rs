use interpreter::Interpreter;
use object::Object;
use result::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ast::token::Literal::{Number, Nil};
use ast::stmt::Stmt;
use env::Env;
use std::rc::Rc;
use std::borrow::Borrow;
use result::Error;

#[derive(Clone, Copy, PartialEq)]
pub enum Type {
    None,
    Function,
    Method,
}

pub trait Callable {
    fn call(&self, int: &mut Interpreter, args: &[Object]) -> Result<Object>;
    fn arity(&self) -> usize;
}

pub struct Clock;

impl Callable for Clock {
    fn arity(&self) -> usize { 0 }

    #[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
    fn call(&self, _: &mut Interpreter, _: &[Object]) -> Result<Object> {
        let dur: Duration = SystemTime::now().
            duration_since(UNIX_EPOCH).expect("time went backwards");

        let ms: f64 = dur.as_secs() as f64 * 1e3 +
            dur.subsec_nanos() as f64 / 1e6;

        Ok(Object::Literal(Number(ms)))
    }
}

pub struct LoxFunction(Rc<Env>, Vec<String>, Rc<Stmt>);

impl LoxFunction {
    pub fn new(env: Rc<Env>, params: &[String], body: Rc<Stmt>) -> Rc<LoxFunction> {
        Rc::new(LoxFunction(
            env,
            params.to_owned(),
            body))
    }
}

impl Callable for LoxFunction {
    fn arity(&self) -> usize { self.1.len() }

    fn call(&self, _: &mut Interpreter, args: &[Object]) -> Result<Object> {
        let env = Env::with_parent(Rc::clone(&self.0));
        let params: &[String] = &self.1;
        let zip = params.into_iter().zip(args.into_iter());

        for (param, arg) in zip {
            env.define(param, arg.clone())?;
        }

        match Interpreter::with_env(env).interpret(self.2.borrow()) {
            Ok(()) => Ok(Object::Literal(Nil)),
            Err(Error::Return(_, res)) => Ok(res),
            Err(e) => Err(e),
        }
    }
}
