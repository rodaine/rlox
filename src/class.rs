use std::fmt;
use functions::Callable;
use result::{Result, Error};
use object::Object;
use std::rc::Rc;
use std::cell::RefCell;
use ast::token::Token;
use ast::token::Type as TokenType;
use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum Type {
    None,
    Class,
    SubClass,
}

pub struct LoxClass {
    name: String,
    parent: Option<Rc<LoxClass>>,
    methods: HashMap<String, Callable>,
}

impl fmt::Debug for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent.as_ref() {
            Some(p) => fmt::Display::fmt(&format!("{}::{}", self.name, p), f),
            None => fmt::Display::fmt(&self.name, f),
        }
    }
}

impl LoxClass {
    pub fn new(name: &str, parent: Option<Rc<LoxClass>>, methods: HashMap<String, Callable>) -> LoxClass {
        let c = LoxClass {
            name: name.to_owned(),
            parent,
            methods,
        };

        debug_create!("{} Class", c);

        c
    }

    pub fn find_method(&self, name: &str) -> Option<&Callable> {
        if let Some(method) = self.methods.get(name) {
            return Some(method);
        }

        if let Some(ref p) = self.parent {
            return p.find_method(name);
        }

        None
    }
}

impl fmt::Display for LoxClass {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.parent.as_ref() {
            Some(p) => fmt::Display::fmt(&format!("{}::{}", self.name, p), f),
            None => fmt::Display::fmt(&self.name, f),
        }
    }
}

#[cfg(feature = "debug-destructors")]
impl Drop for LoxClass {
    fn drop(&mut self) {
        match self.parent.as_ref().map_or(0, |p| Rc::strong_count(p)) {
            0 => debug_drop!("{} class", self),
            refs => debug_drop!("{} class ({} parent refs remaining)", self, refs - 1),
        }
    }
}

pub struct LoxInstance {
    loc: Token,
    class: Rc<LoxClass>,
    fields: Rc<RefCell<HashMap<String, Object>>>,
}

impl Clone for LoxInstance {
    fn clone(&self) -> Self {
        let i = LoxInstance {
            loc: self.loc.clone(),
            class: Rc::clone(&self.class),
            fields: Rc::clone(&self.fields),
        };

        debug_create!(
            "Cloned {:?} ({} class refs)",
            i, Rc::strong_count(&i.class));

        i
    }
}

impl LoxInstance {
    pub fn new(class: &Rc<LoxClass>, loc: &Token) -> LoxInstance {
        let i = LoxInstance {
            loc: loc.clone(),
            class: Rc::clone(class),
            fields: Rc::new(RefCell::new(HashMap::new())),
        };

        debug_create!("{:?} ({} class refs)", i, Rc::strong_count(&i.class));

        i
    }

    pub fn get(&self, field: &Token) -> Result<Object> {
        if let Some(obj) = self.fields.borrow().get(&field.lexeme) {
            return Ok(obj.clone());
        }

        if let Some(method) = self.class.find_method(&field.lexeme) {
            return Ok(Object::Func(method.bind(self)));
        }

        Err(Error::Runtime(
            field.line,
            format!("undefined property `{}`", field.lexeme),
            field.lexeme.to_owned()))
    }

    pub fn set(&self, field: &Token, val: Object) -> Result<Object> {
        self.fields.borrow_mut()
            .insert(field.lexeme.clone(), val.clone());
        debug_assign!("{:?}.{} => {:?}", self, field.lexeme, val);
        Ok(val)
    }
}

impl fmt::Debug for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance<{}:{}>", self.class, self.loc.line, self.loc.offset)
    }
}

impl fmt::Display for LoxInstance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} instance", self.class)
    }
}

lazy_static! {
    pub static ref THIS_ID : Token = Token {
        typ: TokenType::This,
        lexeme: "this".to_owned(),
        ..Token::default()
    };

    pub static ref SUPER_ID : Token = Token {
        typ: TokenType::Super,
        lexeme: "super".to_owned(),
        ..Token::default()
    };
}

#[cfg(feature = "debug-destructors")]
impl Drop for LoxInstance {
    fn drop(&mut self) {
        match Rc::strong_count(&self.fields) {
            1 => debug_drop!("{:?} with fields {:?}", self, self.fields.borrow().keys()),
            refs => debug_drop!("{:?} reference ({} class refs)", self, refs -1),
        }
    }
}
