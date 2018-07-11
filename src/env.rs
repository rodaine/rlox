use std::collections::HashMap;
use result::*;
use std::rc::{Weak, Rc};
use std::cell::RefCell;
use object::Object;
use functions::*;
use ast::token::Token;

#[derive(Default, Debug)]
pub struct Env {
    parent: Option<Parent>,
    vals: RefCell<HashMap<String, Object>>,
}

impl Env {
    pub fn new() -> Rc<Env> {
        let e = Env::init(None);
        Callable::define_globals(e.as_ref());

        debug_create!("Env::Root");

        e
    }

    pub fn from(parent: &Rc<Env>) -> Rc<Env> {
        let e = Env::init(Some(Parent::Strong(Rc::clone(parent))));

        debug_create!(
            "Env::Strong (parent has {} refs now)",
            e.parent.as_ref().map_or(0, |p| p.refs()));
        e
    }

    pub fn from_weak(parent: &Rc<Env>) -> Rc<Env> {
        if parent.has_weak() {
            debug_create!("Env chain already has weak reference");
            return Env::from(parent)
        }

        let e = Env::init(Some(Parent::Weak(Rc::downgrade(parent))));

        debug_create!(
            "Env::Weak (parent has {} refs)",
            e.parent.as_ref().map_or(0, |p| p.refs()));

        e
    }

    pub fn define(&self, id: &Token, val: Object) -> Result<()> {
        let name = &id.lexeme;
        let mut vals = self.vals.borrow_mut();

        if vals.contains_key(name) {
            return Err(Error::Runtime(id.line,
                                      format!("variable `{}` already defined", name),
                                      name.to_owned()));
        }

        debug_define!("{} => {:?}", name, val);
        let _ = vals.insert(name.to_owned(), val);


        Ok(())
    }

    pub fn assign_at(&self, id: &Token, val: Object, dist: Option<&usize>) -> Result<Object> {
        if dist.map_or(0, |d| *d) == 0 {
            return self.assign(id, val);
        }

        let d: usize = *dist.unwrap();

        if let Some(ancestor) = self.ancestor(d) {
            return ancestor.assign(id, val);
        }

        Err(Error::Runtime(id.line,
                           format!("ancestor is undefined at depth {}", d),
                           id.lexeme.to_string()))
    }

    pub fn get_at(&self, id: &Token, dist: Option<&usize>) -> Result<Object> {
        if dist.is_none() {
            return self.get_global(id);
        }

        let d = *dist.unwrap();

        if d == 0 {
            return self.get(id);
        }

        if let Some(ancestor) = self.ancestor(d) {
            return ancestor.get(id);
        }

        Err(Error::Runtime(id.line,
                           format!("ancestor is undefined at depth {}", d),
                           id.lexeme.to_string()))
    }

    pub fn has_weak(&self) -> bool {
        match self.parent {
            Some(ref p) => p.has_weak(),
            None => false,
        }
    }
}

impl Env {
    fn init(parent: Option<Parent>) -> Rc<Env> {
        Rc::new(Env {
            parent,
            vals: RefCell::new(HashMap::new()),
        })
    }

    fn ancestor(&self, dist: usize) -> Option<Parent> {
        let mut env = self.parent.clone();

        for _ in 1..dist {
            env = env?.parent();
        }

        env
    }

    fn assign(&self, id: &Token, val: Object) -> Result<Object> {
        let name = &id.lexeme;
        let mut vals = self.vals.borrow_mut();

        if !vals.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.assign(id, val);
            }

            return Err(Error::Runtime(id.line,
                                      format!("variable `{}` is undefined", name),
                                      name.to_owned()));
        }

        debug_assign!("{} => {:?}", name, val);
        let _ = vals.insert(name.to_owned(), val.clone());
        Ok(val)
    }

    fn get(&self, id: &Token) -> Result<Object> {
        let name = &id.lexeme;
        let vals = self.vals.borrow();

        if !vals.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.get(id);
            }

            return Err(Error::Runtime(id.line,
                                      format!("variable `{}` is undefined", name),
                                      name.to_string()));
        }

        Ok(vals.get(name).cloned().unwrap())
    }

    fn get_global(&self, id: &Token) -> Result<Object> {
        match self.parent {
            None => self.get(id),
            Some(ref parent) => parent.get_global(id),
        }
    }
}

#[cfg(feature = "debug-destructors")]
impl Drop for Env {
    fn drop(&mut self) {
        let details = match self.parent {
            Some(ref p) => match *p {
                Parent::Strong(ref e) => format!(
                    "Env::Strong (parent now has {} refs)",
                    e.parent.as_ref().map_or(0, |p| p.refs()-1)),
                Parent::Weak(ref w) => match w.upgrade() {
                    Some(ref e) => format!(
                        "Env::Weak (parent has {} refs)",
                        e.parent.as_ref().map_or(0, |p| p.refs())),
                    None => "Env::Unknown (parent dropped out of scope)".to_owned(),
                }
            },
            None => "Env::Root".to_owned(),
        };

        debug_drop!("{} with keys {:?}", details, self.vals.borrow().keys());
    }
}

#[derive(Debug, Clone)]
enum Parent {
    Strong(Rc<Env>),
    Weak(Weak<Env>),
}

macro_rules! parent_call {
    ($self:ident$(.$member:ident)+ $(, $arg:expr)* ) => {
        match *$self {
            Parent::Strong(ref e) => e$(.$member)+($($arg,)*),
            Parent::Weak(ref w) => match w.upgrade() {
                Some(ref e) => e$(.$member)+($($arg,)*),
                None => panic!("parent env went out of scope"),
            }
        }
    };
}


impl Parent {
    fn parent(&self) -> Option<Parent> { parent_call!(self.parent.clone) }
    fn assign(&self, id: &Token, val: Object) -> Result<Object> { parent_call!(self.assign, id, val) }
    fn get(&self, id: &Token) -> Result<Object> { parent_call!(self.get, id) }
    fn get_global(&self, id: &Token) -> Result<Object> { parent_call!(self.get_global, id) }

    fn refs(&self) -> usize {
        match *self {
            Parent::Strong(ref e) => Rc::strong_count(e),
            Parent::Weak(ref w) => match w.upgrade() {
                Some(ref e) => Rc::strong_count(e) - 1,
                None => 0,
            }
        }
    }

    fn has_weak(&self) -> bool {
        match *self {
            Parent::Strong(ref e) => e.has_weak(),
            Parent::Weak(_) => true,
        }
    }
}
