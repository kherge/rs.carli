//! A library for rapid command line tool development.
//!
//! CarLI is a framework for creating single-command and multi-command CLI applications in Rust.
//! The framework provides error and IO types better suited for the command line environment,
//! especially in cases where unit testing is needed. Opinionated traits are also provided to
//! enforce a consistent way of structuring the application and its subcommands.
//!
//! See [`command::Main`] for a complete example.

pub mod command;
pub mod error;
pub mod io;

/// Provides preludes for different contexts in command line application development.
pub mod prelude {
    /// A module to easily import APIs frequently used by applications.
    pub mod app {
        pub use crate::command::{Execute, Main};
        pub use crate::error::Result;
        pub use crate::io::{standard, Shared, Stream};
    }

    /// A module to easily import APIs frequently used by subcommands.
    pub mod cmd {
        pub use crate::command::Execute;
        pub use crate::err;
        pub use crate::error::{Context, Result};
        pub use crate::io::Shared;
    }

    /// A module to easily import frequently used testing APIs.
    pub mod test {
        pub use crate::error::Inspect;
        pub use crate::io::memory;
    }
}
