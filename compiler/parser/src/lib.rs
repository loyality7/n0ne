//! n0ne Compiler - Parser implementation.
//! Recursive descent parser that converts a stream of Tokens into an AST.

pub mod precedence;
pub mod parser;
pub mod decl;
pub mod stmt;
pub mod expr;

#[cfg(test)]
mod tests;

pub use parser::Parser;
