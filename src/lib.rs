//! CarLI is a framework for quickly building command line application.
//!
//! In addition to simply parsing arguments, ...

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
        pub use crate::error::Result;
    }

    /// A module to easily import frequently used testing APIs.
    pub mod test {
        pub use crate::error::Inspect;
        pub use crate::io::memory;
    }
}
