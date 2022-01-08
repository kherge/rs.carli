//! CarLI is a framework for quickly building command line application.
//!
//! In addition to simply parsing arguments, ...

pub mod error;

/// A module to easily import frequently used APIs.
pub mod prelude {
    pub use crate::error::{Error, ErrorContext, Result};
    pub use crate::{err, error};
}
