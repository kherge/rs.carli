//! Provides a means to handle errors intended for command line interfaces.
//!
//! Command line applications are able to report errors that may occur to users by printing a
//! message to `STDERR` and exiting its process with a status code. The exit status code makes
//! it possible to determine if an error has occur, what type of error has occurred, and the
//! output to `STDERR` may shed more light on what the error is.
//!
//! ```no_run
//! use std::process::exit;
//!
//! # fn main() {
//! eprintln!("Something went wrong!");
//!
//! exit(1);
//! # }
//! ```
//!
//! This module provides a structured way of producing these errors while allowing other operations
//! to take place before ultimately exiting the process. Some things you may want to do is include
//! a little bit more information on the context on what the error could mean, which is especially
//! important for very generic error messages that may originate from the operating system.
//!
//! ```no_run
//! use carli::err;
//! use carli::error::{Context, Error, Result};
//! use std::fs::read_to_string;
//! use std::path::PathBuf;
//!
//! fn read_file() -> Result<String> {
//!     let path = PathBuf::from("/does/not/exist");
//!
//!     match read_to_string(&path) {
//!         Ok(string) => Ok(string),
//!         Err(error) => Err(Error::from(error)
//!             .context("Could not read from: /does/not/exist"))
//!     }
//! }
//!
//! fn needed_that_string() -> Result<bool> {
//!     let contents = read_file()
//!         .context(|| "Unable to get contents for comparison.")?;
//!
//!     Ok(contents == "example")
//! }
//!
//! fn main() {
//!     let result = needed_that_string();
//!
//!     match result {
//!         Ok(matched) => println!(
//!             "The value {}.",
//!             if matched { "matched" } else { "did not match" }
//!         ),
//!         Err(error) => error.exit(),
//!     }
//! }
//! ```
//!
//! In the above example, `read_file()` will failed to read from a file that does not exist and
//! will return an [`Error`]. However, before returning it some additional context is added in
//! the form of a description of what the operation was. The `needed_that_string()` function will
//! the add its own context before ultimately handing the error to `main()`. The following output
//! is expected before the process exits with the same status code as the raw OS error code:
//!
//! ```text
//! Unable to get contents for comparison.
//!   Could not read from: /does/not/exist
//!     No such file or directory (os error 2)
//! ```

use std::{fmt, process};

/// A trait to add context to an error result.
///
/// This trait simplifies conditionally adding context to a [`Result`] that may be an [`Err`]. The
/// function used to create the context message is only invoked if the result is an [`Err`], which
/// optimizes away the call if the result is [`Ok`].
pub trait Context {
    /// Adds context to an error result using a closure that produces a string.
    ///
    /// ```no_run
    /// use carli::error::{Context, Error, Result};
    ///
    /// fn creates_error() -> Result<()> {
    ///     Err(Error::new(1).message("The original error message."))
    /// }
    ///
    /// fn adds_context() -> Result<()> {
    ///     creates_error().context(|| "Some additional context.")?;
    ///
    ///     Ok(())
    /// }
    ///
    /// fn main() {
    ///     if let Err(error) = adds_context() {
    ///         error.exit();
    ///     }
    /// }
    /// ```
    fn context<F, S: Into<String>>(self, message: F) -> Self
    where
        F: FnOnce() -> S;
}

impl<T> Context for Result<T> {
    fn context<F, S: Into<String>>(self, message: F) -> Self
    where
        F: FnOnce() -> S,
    {
        self.map_err(|error| error.context(message()))
    }
}

/// An error with an exit status.
///
/// This error is designed to make it easy to exit a processes with an error message and
/// exit status code. The type manages the exit status code, an optional error message, and
/// any additional context messages that may be added as the error travels up the call stack.
///
/// ```no_run
/// use carli::error::Error;
///
/// # fn main() {
/// // The exit status is always required.
/// let error = Error::new(1)
///
///     // We can skip the message if it was already displayed to the user.
///     .message("The original error message.")
///
///     // We can add some additional context in case the original message is confusing.
///     .context("Some additional context.")
///
///     // We can add even more context to narrow down where the error is occurring.
///     .context("Even more specific context.");
///
/// error.exit();
/// # }
/// ```
///
/// The above example, if run in an application, would result in the following being printed to
/// `STDERR` before exiting the process with a status code of `1`:
///
/// ```text
/// Even more specific context.
///   Some additional context.
///     The original error message.
/// ```
#[derive(Debug)]
pub struct Error {
    /// The additional context messages for the error.
    context: Option<Vec<String>>,

    /// The original error message.
    message: Option<String>,

    /// The exit status code.
    status: i32,
}

impl Default for Error {
    fn default() -> Self {
        Self {
            context: None,
            message: None,
            status: 1,
        }
    }
}

impl Error {
    /// Adds context to the error.
    ///
    /// A context message should be added when the original error message may be confusing. An
    /// example is [`io::Error`] which, when formatted for display, simply prints the underlying
    /// OS error message (e.g. `No such file or directory`). By adding a little bit of context,
    /// we can clarify to the user where this error could be occurring.
    ///
    /// ```
    /// # use carli::error::Error;
    /// # fn main() {
    /// let error = Error::new(1)
    ///     .message("The original error message.")
    ///     .context("Some added context.");
    /// # }
    pub fn context<S: Into<String>>(mut self, message: S) -> Self {
        if self.context.is_none() {
            self.context = Some(Vec::new())
        }

        self.context.as_mut().unwrap().push(message.into());

        self
    }

    /// Exits the process using this error.
    ///
    /// When the application has reached a point where the only remaining task is to exit, this
    /// method may be called to print the error and context messages to `STDERR` and finally exit
    /// with the appropriate exit status code.
    ///
    /// ```no_run
    /// # use carli::error::Error;
    /// # fn main() {
    /// let error = Error::new(1).message("An example error.");
    ///
    /// error.exit();
    /// # }
    /// ```
    pub fn exit(self) -> ! {
        if self.context.is_some() || self.message.is_some() {
            eprintln!("{}", self);
        }

        process::exit(self.status);
    }

    /// Sets the original error message.
    ///
    /// ```
    /// # use carli::error::Error;
    /// # fn main() {
    /// let error = Error::new(1).message("The error message.");
    /// # }
    /// ```
    pub fn message<S: Into<String>>(mut self, message: S) -> Self {
        self.message = Some(message.into());

        self
    }

    /// Creates a new error with the given exit status code.
    ///
    /// ```
    /// # use carli::error::Error;
    /// # fn main() {
    /// let error = Error::new(1);
    /// # }
    /// ```
    pub fn new(status: i32) -> Self {
        Self {
            context: None,
            message: None,
            status,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut depth = 0;

        if let Some(context) = self.context.as_ref() {
            for message in context.iter().rev() {
                writeln!(f, "{}{}", " ".repeat(depth * 2), message)?;

                depth += 1;
            }
        }

        if let Some(message) = &self.message {
            writeln!(f, "{}{}", " ".repeat(depth * 2), message)?;
        }

        Ok(())
    }
}

impl<T: std::error::Error + 'static> From<T> for Error {
    fn from(error: T) -> Self {
        let mut context = None;
        let mut current = &error as &dyn std::error::Error;
        let message;
        let mut status = 1;

        // Allow for error source traversal.
        loop {
            // If not at the lowest level, capture the error as context.
            if let Some(next) = current.source() {
                context
                    .get_or_insert_with(|| Vec::new())
                    .push(current.to_string());

                current = next;

            // If at the lowest level, capture the message.
            } else {
                message = Some(current.to_string());

                // If std::io::Error, capture the OS error code as the status.
                if let Some(other) = current.downcast_ref::<std::io::Error>() {
                    if let Some(code) = other.raw_os_error() {
                        status = code;
                    }
                }

                break;
            }
        }

        Self {
            context,
            message,
            status,
        }
    }
}

/// A trait for inspecting the contents of error with exit statuses.
///
/// When this trait is brought into scope, access to the inner context, message, and status code
/// for an instance of [`Error`] becomes accessible. The trait is primarily useful when testing
/// error conditions of a command, and should probably not be used during normal application
/// procedures.
///
/// ```
/// use carli::error::{Error, Inspect, Result};
///
/// fn example() -> Result<()> {
///     Err(Error::new(1)
///         .message("An example error.")
///         .context("With some context."))
/// }
///
/// #[cfg(test)]
/// mod test {
///     use super::*;
///
///     fn main() {
///         let error = example().unwrap_err();
///
///         assert_eq!(error.get_context(), Some(vec!["With some context."]));
///         assert_eq!(error.get_message(), Some("An example error."));
///         assert_eq!(error.get_status(), 1);
///     }
/// }
/// ```
pub trait Inspect {
    /// Returns the additional context messages.
    ///
    /// ```
    /// use carli::error::{Error, Inspect, Result};
    ///
    /// fn example() -> Result<()> {
    ///     Err(Error::new(1).context("Some context."))
    /// }
    ///
    /// #[cfg(test)]
    /// mod test {
    ///     use super::*;
    ///
    ///     fn example_context() {
    ///         let error = example().unwrap_err();
    ///
    ///         assert_eq!(error.get_context(), Some(vec!["Some context."]));
    ///     }
    /// }
    /// ```
    fn get_context(&self) -> Option<Vec<&str>>;

    /// Returns the original error message.
    ///
    /// ```
    /// use carli::error::{Error, Inspect, Result};
    ///
    /// fn example() -> Result<()> {
    ///     Err(Error::new(1).message("An example error."))
    /// }
    ///
    /// #[cfg(test)]
    /// mod test {
    ///     use super::*;
    ///
    ///     fn example_message() {
    ///         let error = example().unwrap_err();
    ///
    ///         assert_eq!(error.get_message(), Some("An example error."));
    ///     }
    /// }
    /// ```
    fn get_message(&self) -> Option<&str>;

    /// Returns the exit status code.
    ///
    /// ```
    /// use carli::error::{Error, Inspect, Result};
    ///
    /// fn example_status() -> Result<()> {
    ///     Err(Error::new(1))
    /// }
    ///
    /// #[cfg(test)]
    /// mod test {
    ///     use super::*;
    ///
    ///     fn main() {
    ///         let error = example().unwrap_err();
    ///
    ///         assert_eq!(error.get_status(), 1);
    ///     }
    /// }
    /// ```
    fn get_status(&self) -> i32;
}

impl Inspect for Error {
    fn get_context(&self) -> Option<Vec<&str>> {
        self.context
            .as_ref()
            .map(|context| context.iter().map(|message| message.as_str()).collect())
    }

    fn get_message(&self) -> Option<&str> {
        self.message.as_deref()
    }

    fn get_status(&self) -> i32 {
        self.status
    }
}

/// A specialized [`Result`] that may be an error with an exit status.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Immediately returns an error.
///
/// This macro is a shortcut to creating and returning an error with an exit status.
///
/// ### Only with a status code
///
/// ```
/// use carli::err;
/// use carli::error::Result;
///
/// fn example() -> Result<()> {
///     eprintln!("An example error message.");
///
///     err!(1);
/// }
/// ```
///
/// ### With a message
///
/// ```
/// use carli::err;
/// use carli::error::Result;
///
/// fn example() -> Result<()> {
///     err!(1, "An example error message.");
/// }
/// ```
///
/// ### With a formatted message
///
/// ```
/// use carli::err;
/// use carli::error::Result;
///
/// fn example() -> Result<()> {
///     err!(1, "An example, {}, error message.", "formatted");
/// }
/// ```
#[macro_export]
macro_rules! err {
    ($status:expr) => {
        return Err($crate::error::Error::new($status))
    };
    ($status:expr, $message:expr) => {
        return Err($crate::error::Error::new($status).message($message))
    };
    ($status:expr, $message:expr, $($args:tt)*) => {
        return Err($crate::error::Error::new($status).message(format!($message, $($args)*)))
    };
}

/// Creates a new error.
///
/// This macro is a shortcut to creating a new error with an exit status.
///
/// ### Only with a status code
///
/// ```no_run
/// use carli::error;
///
/// # fn main() {
/// let error = error!(1);
///
/// eprintln!("An example error message.");
///
/// error.exit();
/// # }
/// ```
///
/// ### With a message
///
/// ```no_run
/// use carli::error;
///
/// # fn main() {
/// let error = error!(1, "An example error message.");
///
/// error.exit();
/// # }
/// ```
///
/// ### With a formatted message
///
/// ```no_run
/// use carli::error;
///
/// # fn main() {
/// let error = error!(1, "An example, {}, error message.", "formatted");
///
/// error.exit();
/// # }
/// ```
#[macro_export]
macro_rules! error {
    ($status:expr) => {
        $crate::error::Error::new($status)
    };
    ($status:expr, $message:expr) => {
        $crate::error::Error::new($status).message($message)
    };
    ($status:expr, $message:expr, $($args:tt)*) => {
        $crate::error::Error::new($status).message(format!($message, $($args)*))
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_context_message() {
        let error = Error::default().context("The context message.");

        assert_eq!(error.context.unwrap(), vec!["The context message."])
    }

    #[test]
    fn create_default_error() {
        let error = Error::default();

        assert_eq!(error.status, 1);
    }

    #[test]
    fn create_error_with_formatted_message() {
        let error = error!(1, "The {} message.", "error");

        assert_eq!(error.message, Some("The error message.".to_string()));
        assert_eq!(error.status, 1);
    }

    #[test]
    fn create_error_with_message() {
        let error = error!(1, "The error message.");

        assert_eq!(error.message, Some("The error message.".to_string()));
        assert_eq!(error.status, 1);
    }

    #[test]
    fn create_error_only_status() {
        let error = error!(1);

        assert_eq!(error.message, None);
        assert_eq!(error.status, 1);
    }

    #[test]
    fn create_new_error() {
        let error = Error::new(123);

        assert_eq!(error.status, 123);
    }

    #[test]
    fn display_error_with_context() {
        let error = Error::default()
            .context("The lower level context message.")
            .context("The higher level context message.");

        assert_eq!(
            format!("{}", error),
            "The higher level context message.\n  The lower level context message.\n"
        );
    }

    #[test]
    fn display_error_only_status() {
        let error = Error::default();

        assert_eq!(format!("{}", error), "");
    }

    #[test]
    fn display_error_with_message() {
        let error = Error::default().message("The original message.");

        assert_eq!(format!("{}", error), "The original message.\n");
    }

    #[test]
    fn display_error_with_message_and_context() {
        let error = Error::default()
            .message("The original message.")
            .context("The lower level context message.")
            .context("The higher level context message.");

        assert_eq!(
            format!("{}", error),
            "The higher level context message.\n  The lower level context message.\n    The original message.\n"
        );
    }

    #[test]
    fn from_error() {
        fn generate_error() -> Result<()> {
            fn source_error() -> Result<()> {
                let _ = std::fs::File::open("/should/not/exist")?;

                Ok(())
            }

            source_error().context(|| "The lower level message.")?;

            Ok(())
        }

        let error = generate_error()
            .context(|| "The higher level message.")
            .unwrap_err();

        assert_eq!(
            error.context,
            Some(vec![
                "The lower level message.".to_string(),
                "The higher level message.".to_string()
            ])
        );
        assert_eq!(
            error.message,
            Some("No such file or directory (os error 2)".to_string())
        );
        assert_eq!(error.status, 2);
    }

    #[test]
    fn result_context() {
        let err: Result<()> = Err(Error::default()).context(|| "The context message.");

        assert_eq!(
            err.unwrap_err().context.unwrap(),
            vec!["The context message."]
        );
    }

    #[test]
    fn return_err_with_formatted_message() {
        let test = |fail| {
            if fail {
                err!(1, "The {} message.", "error");
            }

            Ok(())
        };

        let error = test(true).unwrap_err();

        assert_eq!(error.message, Some("The error message.".to_string()));
        assert_eq!(error.status, 1);
    }

    #[test]
    fn return_err_with_message() {
        let test = |fail| {
            if fail {
                err!(1, "The error message.");
            }

            Ok(())
        };

        let error = test(true).unwrap_err();

        assert_eq!(error.message, Some("The error message.".to_string()));
        assert_eq!(error.status, 1);
    }

    #[test]
    fn return_err_only_status() {
        let test = |fail| {
            if fail {
                err!(1);
            }

            Ok(())
        };

        let error = test(true).unwrap_err();

        assert_eq!(error.message, None);
        assert_eq!(error.status, 1);
    }

    #[test]
    fn set_original_message() {
        let error = Error::default().message("The original message.");

        assert_eq!(error.message, Some("The original message.".to_string()));
    }
}
