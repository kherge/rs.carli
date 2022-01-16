//! An example for a single command application.
//!
//! In this example, you can see that the application is broken down into three parts: the command
//! line options, the entrypoint, and the function that does the real work. For processing of the
//! command line options, [`clap`] is being used. The entrypoint and function both use parts of
//! [`carli`] to handle IO and errors that may result from the function.

use carli::io::Streams;
use carli::prelude::app::*;
use carli::prelude::cmd::*;
use clap::Parser;
use std::io::Write;

/// An example application that is a single command.
#[derive(Parser)]
pub struct Application {
    /// A flag to determine if the command should result in an error.
    #[clap(short, long)]
    error: bool,
}

/// A simple function that does the real work in the application.
///
/// This is the meat-and-potatoes" of the application. The function takes the parsed command line
/// arguments, input and output streams, and does something with it. In this example, the function
/// will simply use the `--error` flag to determine if the application should exit with an error or
/// success.
fn example(app: &Application, streams: &Streams) -> Result<()> {
    // Requesting that we produce an error?
    if app.error {
        // Write to the error output stream.
        writeln!(streams.error(), "The command is about to fail!")?;

        // Returns with an error that has a message and exit status code.
        err!(1, "The command failed.");

    // Expecting success?
    } else {
        // Write to the global output stream.
        writeln!(streams.output(), "Hello, world!")?;

        // Since the result ultimately ends up in the entrypoint function (main), we cannot
        // return any result. So, we return nothing and let the operating environment take
        // it from there.
        Ok(())
    }
}

/// Sets up and executes the command.
fn main() {
    // Parse the command line options into an instance of a type.
    let app = Application::parse();

    // Uses the standard input and output streams.
    let streams = standard();

    // Do the real work, and exit if there is an error.
    if let Err(error) = example(&app, &streams) {
        error.exit();
    }

    // If there is no error, the application will naturaly exit with a 0 (zero) status code.
}

#[cfg(test)]
mod test {
    use super::{example, Application};
    use carli::prelude::test::*;
    use std::io::{Seek, SeekFrom};

    /// Verifies that when the `error` flag is used, the function returns a failing response.
    #[test]
    fn example_should_fail() {
        // Create the `Application` instance with the error flag set.
        let app = Application { error: true };

        // Create streams that we can debug.
        let streams = memory();

        // Do the real work.
        let result = example(&app, &streams);

        // Make sure we got the error we were expecting.
        assert!(result.is_err());

        let error = result.unwrap_err();

        assert_eq!(error.get_message(), Some("The command failed."));
        assert_eq!(error.get_status(), 1);

        // Make sure the expected error output was written.
        streams.error().seek(SeekFrom::Start(0)).unwrap();

        assert_eq!(
            streams.error().to_string_lossy(),
            "The command is about to fail!\n"
        );
    }

    /// Verifies that the function returns a successful response if the `error` flag is not used.
    #[test]
    fn example_should_succeed() {
        // Create the `Application` instance without the error flag set.
        let app = Application { error: false };

        // Create streams that we can debug.
        let streams = memory();

        // Do the real work.
        let result = example(&app, &streams);

        // Make sure we got the success result we were expecting.
        assert!(result.is_ok());

        // Make sure the expected output was written.
        streams.output().seek(SeekFrom::Start(0)).unwrap();

        assert_eq!(streams.output().to_string_lossy(), "Hello, world!\n");
    }
}
