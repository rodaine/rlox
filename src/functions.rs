use interpreter::Interpreter;
use object::Object;
use result::Result;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use ast::token::Token;
use ast::token::Type as TokenType;
use ast::token::Literal::{Number, Nil};
use ast::stmt::Stmt;
use env::Env;
use std::rc::Rc;
use result::Error;
use class::{LoxInstance, THIS_ID, LoxClass};
use std::fmt;

pub const INITIALIZER_FUNC: &str = "init";

#[derive(Clone, Copy, PartialEq)]
pub enum Type {
    None,
    Function,
    Initializer,
    Method,
}

#[derive(Clone, Debug)]
pub enum Callable {
    Runtime(LoxFunction),
    Initializer(InitFunction),
    Static(StaticFunction),
}

impl Callable {
    pub fn new(env: Rc<Env>, params: &[Token], body: &Rc<Stmt>, init: bool) -> Callable {
        debug_create!("LoxFunction with arity {}", params.len());
        Callable::Runtime(LoxFunction::new(env, params, body, init))
    }

    pub fn init(cls: &Rc<LoxClass>) -> Callable {
        debug_create!("{} Initializer", cls);
        Callable::Initializer(InitFunction(Rc::clone(cls)))
    }

    pub fn define_globals(env: &Env) {
        let clock = Object::Func(Callable::Static(StaticFunction::clock()));
        env.define(&CLOCK_ID, clock).expect("unable to attach clock()");
    }

    pub fn call(&self, int: &Interpreter, args: &[Object], paren: &Token) -> Result<Object> {
        match *self {
            Callable::Runtime(ref f) => f.call(int, args),
            Callable::Static(ref f) => f.call(int, args),
            Callable::Initializer(ref cls) => cls.call(int, args, paren),
        }
    }

    pub fn arity(&self) -> usize {
        match *self {
            Callable::Runtime(ref f) => f.arity(),
            Callable::Static(ref f) => f.arity(),
            Callable::Initializer(ref cls) => cls.arity(),
        }
    }

    pub fn bind(&self, inst: &LoxInstance) -> Callable {
        match *self {
            Callable::Runtime(ref f) => Callable::Runtime(f.bind(inst)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone)]
pub struct LoxFunction {
    scope: Rc<Env>,
    params: Vec<Token>,
    body: Rc<Stmt>,
    initializer: bool,
}

impl LoxFunction {
    fn new(scope: Rc<Env>, params: &[Token], body: &Rc<Stmt>, init: bool) -> LoxFunction {
        LoxFunction {
            scope,
            params: params.to_owned(),
            body: Rc::clone(body),
            initializer: init,
        }
    }

    fn bind(&self, inst: &LoxInstance) -> LoxFunction {
        let scope = Env::from(&self.scope);

        scope.define(&THIS_ID, Object::Instance(inst.clone()))
            .expect("failed to define `this`");

        LoxFunction::new(scope, &self.params, &self.body, self.initializer)
    }

    fn arity(&self) -> usize { self.params.len() }

    fn call(&self, int: &Interpreter, args: &[Object]) -> Result<Object> {
        let env = Env::from(&self.scope);
        let zip = (&self.params).into_iter().zip(args.into_iter());

        for (param, arg) in zip {
            env.define(param, arg.clone())?;
        }

        match self.body.accept(&mut int.with_env(env)) {
            Ok(()) | Err(Error::Return(_, _)) if self.initializer =>
                self.scope.get_at(&THIS_ID, Some(&0)),
            Ok(()) => Ok(Object::Literal(Nil)),
            Err(Error::Return(_, res)) => Ok(res),
            Err(e) => Err(e),
        }
    }
}

impl fmt::Debug for LoxFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LoxFunction<TODO>")
    }
}

#[cfg(feature = "debug-destructors")]
impl Drop for LoxFunction {
    fn drop(&mut self) {
        debug_drop!(
            "LoxFunction (scope has {} refs now)",
            Rc::strong_count(&self.scope)-1);
    }
}

#[derive(Debug, Clone)]
pub struct InitFunction(Rc<LoxClass>);

impl InitFunction {
    fn call(&self, int: &Interpreter, args: &[Object], paren: &Token) -> Result<Object> {
        let inst = LoxInstance::new(&self.0, paren);

        if let Some(method) = self.0.find_method(INITIALIZER_FUNC) {
            method.bind(&inst).call(int, args, paren)?;
        }

        Ok(Object::Instance(inst))
    }

    fn arity(&self) -> usize {
        self.0.find_method(INITIALIZER_FUNC)
            .map_or(0, |m| m.arity())
    }
}

#[cfg(feature = "debug-destructors")]
impl Drop for InitFunction {
    fn drop(&mut self) {
        debug_drop!("Initializer with arity {}", self.arity());
    }
}

#[derive(Clone)]
pub struct StaticFunction {
    name: String,
    _arity: usize,
    func: fn(&Interpreter, &[Object]) -> Result<Object>,
}

impl StaticFunction {
    fn new(name: &str, arity: usize, func: fn(&Interpreter, &[Object]) -> Result<Object>) -> StaticFunction {
        debug_create!("StaticFunction {}", name);
        StaticFunction {
            name: name.to_owned(),
            _arity: arity,
            func,
        }
    }

    fn clock() -> StaticFunction { StaticFunction::new("clock", 0, clock) }

    fn call(&self, int: &Interpreter, args: &[Object]) -> Result<Object> {
        (self.func)(int, args)
    }

    fn arity(&self) -> usize { self._arity }
}

impl fmt::Debug for StaticFunction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "StaticFunction<{}>", self.name)
    }
}

#[cfg(feature = "debug-destructors")]
impl Drop for StaticFunction {
    fn drop(&mut self) {
        debug_drop!("{:?}", self);
    }
}

lazy_static! {
    pub static ref CLOCK_ID : Token = Token {
        typ: TokenType::Identifier,
        lexeme: "clock".to_owned(),
        ..Token::default()
    };
}

#[cfg_attr(feature = "cargo-clippy", allow(cast_lossless))]
fn clock(_: &Interpreter, _: &[Object]) -> Result<Object> {
    let dur: Duration = SystemTime::now().
        duration_since(UNIX_EPOCH).expect("time went backwards");

    let ms: f64 = dur.as_secs() as f64 * 1e3 +
        dur.subsec_nanos() as f64 / 1e6;

    Ok(Object::Literal(Number(ms)))
}
