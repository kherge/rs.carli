//! Provides the primary application type for processing the command line arguments.

mod subcommand;

use carli::prelude::app::*;
use std::cell::RefCell;
use std::io::{stderr, stdin, stdout};

/// Contains the shared application state and IO streams.
///
/// `Application` is an implementation of [`Shared`] which allows any instance of it to be used
/// as context for the executing subcommand, and [`Main`] to enforce consistency in how subcommands
/// are handled. In addition to managing the different streams, instances are also used to share
/// the globally used command line options.
#[derive(clap::Parser)]
#[clap(
    name = "multiple",
    about = "An example application that offers multiple subcommands."
)]
pub struct Application {
    /// The error output stream.
    #[clap(skip = RefCell::new(stderr().into()))]
    error: RefCell<Stream>,

    /// The input stream.
    #[clap(skip = RefCell::new(stdin().into()))]
    input: RefCell<Stream>,

    /// The name of the user.
    #[clap(short, long, global = true, default_value = "world")]
    name: String,

    /// The global output stream.
    #[clap(skip = RefCell::new(stdout().into()))]
    output: RefCell<Stream>,

    /// The subcommand requested by the user.
    #[clap(subcommand)]
    subcommand: subcommand::Subcommand,
}

impl Application {
    /// Returns the name of the user.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Main for Application {
    fn subcommand(&self) -> &dyn Execute<Self> {
        &self.subcommand
    }
}

impl Shared for Application {
    fn error(&self) -> std::cell::RefMut<Stream> {
        self.error.borrow_mut()
    }

    fn input(&self) -> std::cell::RefMut<Stream> {
        self.input.borrow_mut()
    }

    fn output(&self) -> std::cell::RefMut<Stream> {
        self.output.borrow_mut()
    }
}
