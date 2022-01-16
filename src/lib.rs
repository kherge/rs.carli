//! CarLI is a framework for quickly building command line application.
//!
//! In addition to simply parsing arguments, ...

pub mod command;
pub mod error;
pub mod io;

/// A module to easily import frequently used APIs.
pub mod prelude {
    pub use crate::error::{Context, Error, Result};
    pub use crate::io::{Shared, Stream, Streams};
    pub use crate::{err, error};
}

/// A module to easily import frequently used testing APIs.
pub mod test {
    pub use crate::error::Inspect;
}
