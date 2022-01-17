//! A library for rapid command line tool development.
//!
//! CarLI is a command line application framework for developing application that provide a single
//! command and multiple commands. The framework also provides error types tailored for a command
//! line experience as well as supporting test input and output streams other than those provided
//! by [`std::io`].
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
