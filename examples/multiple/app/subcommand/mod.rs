//! Contains the subcommands that can be executed by the user.
//!
//! The subcommands are broken up into their own packages so that they are a little more self-
//! contained. The benefit of this design is that it compartmentalizes code that is otherwise
//! irrelevant to the other subcommands. Additionally, you may have some subcommands that are
//! considerably large and would be better placed in their own modules.

mod goodbye;
mod hello;

use crate::app::Application;
use carli::prelude::cmd::*;

/// The subcommands that are offered by the application.
#[derive(clap::Subcommand)]
pub enum Subcommand {
    /// A subcommand for saying goodbye.
    Goodbye(goodbye::Subcommand),

    /// A subcommand for saying hello.
    Hello(hello::Subcommand),
}

impl Execute<Application> for Subcommand {
    fn execute(&self, context: &Application) -> Result<()> {
        match self {
            Self::Goodbye(cmd) => cmd.execute(context),
            Self::Hello(cmd) => cmd.execute(context),
        }
    }
}
