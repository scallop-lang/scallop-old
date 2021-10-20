#![feature(map_first_last)]

mod dataflow;
pub mod dataflows;
mod element;
pub mod error;
pub mod interpreter;
mod iteration;
mod program;
mod relation;
mod semiring;
pub mod tags;
mod tuple;
mod utils;
mod variable;
mod variable_handle;
pub mod wmc;

pub use dataflow::*;
pub use element::*;
pub use iteration::*;
pub use program::*;
pub use relation::*;
pub use semiring::*;
pub use tags::*;
pub use tuple::*;
pub use variable::*;
pub use variable_handle::*;
