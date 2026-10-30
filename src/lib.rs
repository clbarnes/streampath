#![doc = include_str!("../README.md")]

// Must be declared before the modules that use its macros.
#[macro_use]
mod macros;

mod reader;
mod stream_path;
mod writer;

pub use reader::StreamPathRead;
pub use stream_path::StreamPath;
pub use writer::StreamPathWrite;
