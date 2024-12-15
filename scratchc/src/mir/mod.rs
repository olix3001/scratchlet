//! MIR (Middle Intermediate Representation) is scratchc's tool for making compilation of some stuff easier.
//! It assumes that all checks have already been done so It does not perform those.
//! This IR contains some basic abstractions that can be reused by the compiler to avoid reimplementing the same thing in multiple places.
//! Those abstractions are:
//! - Variables
//! - Structure types
//! - Procedure returns
//! - Conditionals and loops

mod code;
mod project;
mod refinery;
mod sprite;

pub use code::*;
pub use project::*;
pub use refinery::*;
pub use sprite::*;
