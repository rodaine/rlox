#[macro_use]
extern crate lazy_static;

mod result;

pub mod scanner;
pub mod token;
pub mod ast;
pub mod parser;

pub use result::{Result, Error};

/// Boxer converts a type into its Boxed form
pub trait Boxer {
    /// Convert to a boxed version
    fn boxed(self) -> Box<Self>;
}
