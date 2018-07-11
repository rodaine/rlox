#[macro_use]
extern crate lazy_static;

#[macro_use]
mod debug;

mod result;

pub mod ast;

pub mod functions;
pub mod object;

#[macro_use]
pub mod env;

pub mod class;

pub mod scanner;
pub mod parser;
pub mod interpreter;
pub mod resolver;

pub mod output;
pub mod run;

pub use result::{Result, Error};

/// Boxer converts a type into its Boxed form
pub trait Boxer {
    /// Convert to a boxed version
    fn boxed(self) -> Box<Self> where Self : Sized { Box::new(self) }
}
