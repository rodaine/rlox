use ast::token;
use class::{LoxClass,LoxInstance};
use functions::Callable;
use std::cmp::Ordering;
use std::fmt;
use std::cmp;
use std::rc::Rc;

#[derive(Clone, Debug)]
pub enum Object {
    Literal(token::Literal),
    Func(Callable),
    Class(Rc<LoxClass>),
    Instance(LoxInstance),
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        use ast::token::Literal::*;

        match *self {
            Object::Func(_) | Object::Class(_) | Object::Instance(_) => true,
            Object::Literal(ref lit) => match *lit {
                Nil => false,
                Boolean(b) => b,
                Number(n) => n != 0.0,
                String(ref s) => !s.is_empty(),
            },
        }
    }
}

#[cfg(feature = "debug-destructors")]
impl Drop for Object{
    fn drop(&mut self) {
        match *self {
            Object::Literal(ref lit) =>
                debug_drop!("Object::Literal {:?}", lit),
            Object::Func(ref c) =>
                debug_drop!("Object::Func {:?}", c),
            Object::Class(ref c) =>
                debug_drop!("Object::Class {:?} ({} refs remain)", c, Rc::strong_count(&c)-1),
            Object::Instance(ref i) =>
                debug_drop!("Object::Instance {:?}", i),
        }
    }
}

impl cmp::PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        use object::Object::Literal as ObjLit;
        match (self, other) {
            (&ObjLit(ref lhs), &ObjLit(ref rhs)) => lhs.eq(rhs),
            _ => false
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Literal(ref lit) => fmt::Display::fmt(lit, f),
            Object::Func(_) => write!(f, "<function>"),
            Object::Class(ref cls) => fmt::Display::fmt(cls, f),
            Object::Instance(ref inst) => fmt::Display::fmt(inst, f),
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use object::Object::Literal as ObjLit;
        match (self, other) {
            (&ObjLit(ref l), &ObjLit(ref r)) => l.partial_cmp(r),
            _ => None,
        }
    }
}

