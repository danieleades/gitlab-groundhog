//! A small command line tool for generating recurring gitlab issues.

#![deny(
    clippy::all,
    missing_debug_implementations,
    missing_docs,
    missing_copy_implementations
)]
#![warn(clippy::pedantic, clippy::nursery)]

use clap::Parser;

mod cli;
mod graphql;
mod issues;
mod ledger;

fn main() -> anyhow::Result<()> {
    let command = cli::Command::parse();
    command.run()
}
