//! Provides opinionated scaffolding for creating a command line application.
//!
//! An application using this library can be implemented in a myriad number of ways. For a library
//! idiomatic way of designing an application, this module provides a collection of traits that may
//! be implemented for a more consistent development experience. The added benefit of using these
//! traits could be that other command line application developers could recognize the design and
//! more easily contribute changes.

use crate::{error, io};

/// A trait for objects which can be executed as commands in an application.
///
/// In larger applications with multiple (even nested) subcommands, it may be appropriate to make
/// use of a structured design for implementing and executing those commands. This trait offers one
/// way of doing so by defining the entry point and expected result of all subcommands. The trait
/// implemented on the following principles:
///
/// 1. There is a context in which the subcommand must operate in.
/// 2. The result should only be used to indicate failure.
///
/// ### Providing context to subcommands
///
/// At minimum, the subcommand should have access to the input and output streams for the process.
/// However, we do not want to limit the information shared with all subcommands. To address these
/// concerns, the trait uses a generic type that implements [`io::Shared`] for context. This allows
/// developers to use [`io::Streams`] as the context, or a custom type.
///
/// ### Handling subcommand results
///
/// Ultimately, all results that are returned by subcommands should bubble up to the main function
/// of the application. Once a result reaches the main function, it should either be discarded or
/// used to determine if the subcommand has encountered an error and should exit with it. Since our
/// options are limited, [`error::Result<()>`] is the preferred return type. Nothing is returned
/// other than an empty [`Ok`] or the subcommand error, [`error::Error`] in [`Err`] which we can
/// exit with.
///
/// ### Example command
///
/// While this example demonstrates how it could be used with a single command, the trait is
/// primarily intended to be used in an application with multiple subcommands. In cases where
/// only a single command exists, it may be simpler skip the use of this trait and define the
/// function to accept any arguments you may need.
///
/// ```
/// use carli::command::Execute;
/// use carli::error::Result;
/// use carli::io::{standard, Shared, Streams};
/// use std::io::Write;
///
/// /// An example command.
/// struct Command {}
///
/// impl Execute<Streams> for Command {
///     fn execute(&self, context: &Streams) -> Result<()> {
///         writeln!(context.output(), "Hello, world!")?;
///
///         Ok(())
///     }
/// }
///
/// /// Executes the example command.
/// fn main() {
///     let command = Command {};
///     let streams = standard();
///
///     if let Err(error) = command.execute(&streams) {
///         error.exit();
///     }
/// }
/// ```
///
pub trait Execute<T>
where
    T: io::Shared,
{
    /// Executes the command using the given context.
    ///
    /// The command may mutably borrow any stream from the context in order to read input, or
    /// write to the error or global output streams. When the command encounters an error, it
    /// returns an [`Err`] containing [`crate::error::Error`]. If the command is successful,
    /// then it will return [`Ok`].
    fn execute(&self, context: &T) -> error::Result<()>;
}

/// A trait for objects which manage the context for application subcommands.
///
/// This trait enforces a particular structure and flexibility to applications that implement it.
/// Implementations of this trait are also required to implement [`io::Shared`], making instances
/// effectively all that is needed to execute a subcommand. When coupled with subcommands that use
/// [`Execute`], instances may also be used to share a global state.
///
/// ```
/// use carli::command::{Execute, Main};
/// use carli::error::Result;
/// use carli::io::{Shared, Stream};
/// use std::cell::{RefCell, RefMut};
/// use std::io::{stderr, stdin, stdout, Write};
///
/// /// An example context with global data.
/// struct Application {
///     /// The error output stream.
///     error: RefCell<Stream>,
///
///     /// The input stream.
///     input: RefCell<Stream>,
///
///     /// The name of the user.
///     name: String,
///
///     /// The global output stream.
///     output: RefCell<Stream>,
///
///     /// The user requested subcommand.
///     subcommand: Subcommand,
/// }
///
/// impl Application {
///     /// Returns the name of the user.
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     /// Creates a new instance of the application.
///     fn new(name: String, subcommand: Subcommand) -> Self {
///         Self {
///             error: RefCell::new(stderr().into()),
///             input: RefCell::new(stdin().into()),
///             name,
///             output: RefCell::new(stdout().into()),
///             subcommand,
///         }
///     }
/// }
///
/// impl Main for Application {
///     fn subcommand(&self) -> &dyn Execute<Self> {
///         &self.subcommand
///     }
/// }
///
/// impl Shared for Application {
///     fn error(&self) -> RefMut<Stream> {
///         self.error.borrow_mut()
///     }
///
///     fn input(&self) -> RefMut<Stream> {
///         self.input.borrow_mut()
///     }
///
///     fn output(&self) -> RefMut<Stream> {
///         self.output.borrow_mut()
///     }
/// }
///
/// /// The example subcommands.
/// enum Subcommand {
///     /// Say goodbye to the user.
///     Goodbye,
///
///     /// Say hello to the user.
///     Hello,
/// }
///
/// impl Execute<Application> for Subcommand {
///     fn execute(&self, context: &Application) -> Result<()> {
///         match self {
///             Self::Goodbye => writeln!(context.output(), "Goodbye, {}!", context.name())?,
///             Self::Hello => writeln!(context.output(), "Hello, {}!", context.name())?,
///         }
///
///         Ok(())
///     }
/// }
///
/// /// Executes the application.
/// fn main() {
///     // You can manage the instantiation of the application, or use a library to do it.
///     let app = Application::new("world".to_string(), Subcommand::Hello);
///
///     // Execute the subcommand and handle any error appropriately.
///     if let Err(error) = app.execute() {
///         error.exit();
///     }
/// }
/// ```
pub trait Main: io::Shared + Sized {
    /// Executes the requested subcommand for the application.
    ///
    /// The implementation of this method should be very simple in that it should only execute the
    /// requested subcommand and return its result. Any additional steps required to put the context
    /// in a more usable state for the subcommand should probably be done elsewhere in order to keep
    /// testing simple.
    fn execute(&self) -> error::Result<()> {
        self.subcommand().execute(self)
    }

    /// Returns the subcommand to be executed.
    fn subcommand(&self) -> &dyn Execute<Self>;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::io::Shared;
    use std::cell;
    use std::io::{Seek, Write};

    /// An example application context.
    struct Application {
        /// The error output stream.
        error: cell::RefCell<io::Stream>,

        /// The input stream.
        input: cell::RefCell<io::Stream>,

        /// The name of the user.
        name: String,

        /// The global output stream.
        output: cell::RefCell<io::Stream>,

        /// The subcommand to execute.
        subcommand: Subcommand,
    }

    impl Application {
        /// Creates a new instance using in-memory buffers.
        fn new(name: String, subcommand: Subcommand) -> Self {
            Self {
                error: cell::RefCell::new(Vec::new().into()),
                input: cell::RefCell::new(Vec::new().into()),
                name,
                output: cell::RefCell::new(Vec::new().into()),
                subcommand,
            }
        }
    }

    impl Main for Application {
        fn subcommand(&self) -> &dyn Execute<Self> {
            &self.subcommand
        }
    }

    impl io::Shared for Application {
        fn error(&self) -> cell::RefMut<io::Stream> {
            self.error.borrow_mut()
        }

        fn input(&self) -> cell::RefMut<io::Stream> {
            self.input.borrow_mut()
        }

        fn output(&self) -> cell::RefMut<io::Stream> {
            self.output.borrow_mut()
        }
    }

    /// An example subcommand that says goodbye.
    struct Goodbye {}

    impl Execute<Application> for Goodbye {
        fn execute(&self, context: &Application) -> error::Result<()> {
            writeln!(context.output(), "Goodbye, {}.", context.name)?;

            Ok(())
        }
    }

    /// An example subcommand that says hello.
    struct Hello {}

    impl Execute<Application> for Hello {
        fn execute(&self, context: &Application) -> error::Result<()> {
            writeln!(context.output(), "Hello, {}!", context.name)?;

            Ok(())
        }
    }

    /// The subcommands available.
    enum Subcommand {
        /// A command to say good bye.
        Goodbye(Goodbye),

        /// A command to say hello.
        Hello(Hello),
    }

    impl Execute<Application> for Subcommand {
        fn execute(&self, context: &Application) -> error::Result<()> {
            match self {
                Self::Goodbye(cmd) => cmd.execute(context),
                Self::Hello(cmd) => cmd.execute(context),
            }
        }
    }

    #[test]
    fn execute_goodbye() {
        let app = Application::new("world".to_string(), Subcommand::Goodbye(Goodbye {}));

        app.execute().unwrap();

        let mut output = app.output();

        output.rewind().unwrap();

        assert_eq!(output.to_string_lossy(), "Goodbye, world.\n");
    }

    #[test]
    fn execute_hello() {
        let app = Application::new("world".to_string(), Subcommand::Hello(Hello {}));

        app.execute().unwrap();

        let mut output = app.output();

        output.rewind().unwrap();

        assert_eq!(output.to_string_lossy(), "Hello, world!\n");
    }
}
