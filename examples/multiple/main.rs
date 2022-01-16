//! An example of a command line application with multiple subcommands.
//!
//! In this example, we take a more advanced approach into how we handle creating an application
//! that can perform different operations depending on the subcommand that the user has requested.
//! In addition to using [`carli`], we will also be using [`clap`] to parse the command line
//! arguments and create a new instance of the application.

mod app;

use app::Application;
use carli::command::Main;
use clap::Parser;

/// Sets up and executes the application.
///
/// This function is straightforward:
///
/// 1. Use [`clap`] to parse the command line options.
/// 2. Use the resulting [`Application`] instance to execute the desired subcommand.
/// 3. If the subcommand results in an error, exit using the error.
fn main() {
    let app = Application::parse();

    if let Err(error) = app.execute() {
        error.exit();
    }
}
