use std::collections::HashMap;
use token::Literal;
use result::*;
use std::rc::Rc;
use std::cell::RefCell;

#[derive(Default)]
pub struct Env {
    parent: Option<Rc<Env>>,
    vals: RefCell<HashMap<String, Literal>>,
}

impl Env {
    pub fn new(parent: Option<Rc<Env>>) -> Rc<Self> {
        Rc::new(Env {
            vals: RefCell::new(HashMap::new()),
            parent: parent,
        })
    }

    pub fn define(&self, name: &str, val: Literal) -> Result<()> {
        let mut vals = self.vals.borrow_mut();

        if vals.contains_key(name) {
            return Err(Error::Runtime(0,
                                      format!("variable `{}` already defined", name),
                                      "".to_string()).boxed())
        }

        let _ = vals.insert(name.to_owned(), val);
        Ok(())
    }

    pub fn assign(&self, name: &str, val: Literal) -> Result<Literal> {
        let mut vals = self.vals.borrow_mut();

        if !vals.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.assign(name, val)
            }

            return Err(Error::Runtime(0,
                                      format!("variable `{}` is undefined", name),
                                      "".to_string()).boxed())
        }

        let _ = vals.insert(name.to_owned(), val.clone());
        Ok(val)
    }

    pub fn get(&self, name: &str) -> Result<Literal> {
        let vals = self.vals.borrow();

        if !vals.contains_key(name) {
            if let Some(ref parent) = self.parent {
                return parent.get(name)
            }

            return Err(Error::Runtime(0,
                                      format!("variable `{}` is undefined", name),
                                      "".to_string()).boxed())
        }

        Ok(vals.get(name).cloned().unwrap())
    }
}
