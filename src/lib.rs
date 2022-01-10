//! CarLI is a framework for quickly building command line application.
//!
//! In addition to simply parsing arguments, ...

pub mod error;
pub mod io;

/// A module to easily import frequently used APIs.
pub mod prelude {
    pub use crate::error::{Error, ErrorContext, Result};
    pub use crate::io::{Context, Standard};
    pub use crate::{err, error, errorln, outputln};
}

/// A module to easily import frequently used testing APIs.
pub mod test {
    pub use crate::error::Inspect;
    pub use crate::io::Test;
}
