//! Mathematical Functional Interpreter

mod interpreter;
mod lexer;
mod parser;

pub type Real = f64;

pub use interpreter::{InputError, InputState, Interpreter};
pub use lexer::InvalidToken;
