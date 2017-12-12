use ast::token;
use functions::Callable;
use std::cmp::Ordering;
use std::fmt;
use std::cmp;
use std::rc::Rc;

#[derive(Clone)]
pub enum Object {
    Literal(token::Literal),
    Func(Rc<Callable>),
}

impl Object {
    pub fn is_truthy(&self) -> bool {
        use ast::token::Literal::*;

        match *self {
            Object::Func(_) => true,
            Object::Literal(ref lit) => match *lit {
                Nil => false,
                Boolean(tf) => tf,
                Number(n) => n != 0.0,
                String(ref s) => !s.is_empty(),
            },
        }
    }
}

impl cmp::PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (&Object::Literal(ref lhs), &Object::Literal(ref rhs)) => lhs.eq(rhs),
            _ => false
        }
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Literal(ref lit) => lit.fmt(f),
            Object::Func(_) => write!(f, "<function>"),
        }
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Object::Literal(ref lit) => lit.fmt(f),
            Object::Func(_) => write!(f, "<function>"),
        }
    }
}

impl PartialOrd for Object {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use std::cmp::Ordering::*;
        use ast::token::Literal::*;
        use object::Object::Literal as ObjLit;

        match (self, other) {
            (&ObjLit(ref l), &ObjLit(ref r)) => match (l, r) {
                (&Nil, &Nil) => Some(Equal),
                (&Number(l), &Number(r)) => l.partial_cmp(&r),
                (&String(ref l), &String(ref r)) => l.partial_cmp(r),
                _ => None,
            },
            _ => None,
        }
    }
}

