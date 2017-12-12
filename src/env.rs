use std::collections::HashMap;
use result::*;
use std::rc::Rc;
use std::cell::RefCell;
use object::Object;
use functions::*;

#[derive(Default, Debug)]
pub struct Env {
    parent: Option<Rc<Env>>,
    vals: RefCell<HashMap<String, Object>>,
}

impl Env {
    pub fn new() -> Rc<Self> {
        let e = Rc::new(Self {
            vals: RefCell::new(HashMap::new()),
            parent: None,
        });

        e.define("clock", Object::Func(Rc::new(Clock))).
            expect("failed to register clock");

        e
    }

    pub fn with_parent(parent: Rc<Self>) -> Rc<Self> {
        Rc::new(Self {
            vals: RefCell::new(HashMap::new()),
            parent: Some(parent),
        })
    }

    pub fn with_globals(env: &Rc<Self>) -> Rc<Self> {
        match env.parent {
            None => Self::with_parent(Rc::clone(env)),
            Some(ref e) => Self::with_globals(&Rc::clone(e)),
        }
    }

    pub fn define(&self, name: &str, val: Object) -> Result<()> {
        let mut vals = self.vals.borrow_mut();

        if vals.contains_key(name) {
            return Err(Error::Runtime(0,
                                      format!("variable `{}` already defined", name),
                                      "".to_string()));
        }

        let _ = vals.insert(name.to_owned(), val);
        Ok(())
    }

    pub fn assign(&self, name: &str, val: Object) -> Result<Object> {
        let mut vals = self.vals.borrow_mut();

        if !vals.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.assign(name, val);
            }

            return Err(Error::Runtime(0,
                                      format!("variable `{}` is undefined", name),
                                      "".to_string()));
        }

        let _ = vals.insert(name.to_owned(), val.clone());
        Ok(val)
    }

    pub fn assign_at(&self, name: &str, val: Object, dist: usize) -> Result<Object> {
        if dist == 0 {
            return self.assign(name, val);
        }

        if let Some(ancestor) = self.ancestor(dist) {
            return ancestor.assign(name, val);
        }

        Err(Error::Runtime(0,
                           format!("ancestor is undefined at depth {}", dist),
                           "".to_string()))
    }

    pub fn get(&self, name: &str) -> Result<Object> {
        let vals = self.vals.borrow();

        if !vals.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.get(name);
            }

            return Err(Error::Runtime(0,
                                      format!("variable `{}` is undefined", name),
                                      "".to_string()));
        }

        Ok(vals.get(name).cloned().unwrap())
    }

    pub fn get_global(&self, name: &str) -> Result<Object> {
        match self.parent {
            None => self.get(name),
            Some(ref parent) => parent.get_global(name),
        }
    }

    pub fn get_at(&self, name: &str, dist: usize) -> Result<Object> {
        if dist == 0 {
            return self.get(name);
        }

        if let Some(ancestor) = self.ancestor(dist) {
            return ancestor.get(name);
        }

        Err(Error::Runtime(0,
                           format!("ancestor is undefined at depth {}", dist),
                           "".to_string()))
    }

    fn ancestor(&self, dist: usize) -> Option<Rc<Env>> {
        let mut env = &self.parent;

        for _ in 1..dist {
            if env.is_none() {
                return None;
            }

            env = &env.as_ref().unwrap().parent;
        }

        env.as_ref().map(|rce| Rc::clone(rce))
    }
}
