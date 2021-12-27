#![allow(missing_docs)] // Prevents double-documenting some things

use clap::Parser;
use crate::TRIBBLE_VERSION;

// The documentation for the `Opts` struct will appear in the help page, hence the lack of puncutation and the lowercasing in places

/// Tribble creates seamless contribution experiences for your project.
#[derive(Parser)]
#[clap(version = TRIBBLE_VERSION)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: Subcommand,
}

#[derive(Parser)]
pub enum Subcommand {
    /// Builds your Tribble workflows. This is called by `tribble serve` automatically.
    Build,
    /// Serves your Tribble workflows locally for development.
    Serve,
    /// Builds your Tribble workflows for release deployment.
    Deploy
}
