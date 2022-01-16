//! A subcommand that says goodbye to the user.

use super::Application;
use carli::prelude::cmd::*;
use std::io::Write;

/// The subcommand options.
#[derive(clap::Parser)]
pub struct Subcommand {
    /// Do we want to yell it?
    #[clap(short, long)]
    yell: bool,
}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> carli::error::Result<()> {
        writeln!(
            context.output(),
            "Goodbye, {}{}",
            context.name(),
            if self.yell { "!" } else { "." }
        )?;

        Ok(())
    }
}
