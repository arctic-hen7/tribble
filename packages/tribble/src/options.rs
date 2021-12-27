#![allow(missing_docs)] // Prevents double-documenting some things

use crate::TRIBBLE_VERSION;
use clap::Parser;

// The documentation for the `Opts` struct will appear in the help page, hence the lack of puncutation and the lowercasing in places

/// Tribble creates seamless contribution experiences for your project.
#[derive(Parser)]
#[clap(version = TRIBBLE_VERSION)]
pub struct Opts {
    /// The path to your Tribble configuration file
    #[clap(long, short, default_value = "./tribble.yml")]
    pub config: String,
    #[clap(subcommand)]
    pub subcmd: Subcommand,
}

#[derive(Parser)]
pub enum Subcommand {
    /// Builds your Tribble workflows. This is called by `tribble serve` automatically
    Build,
    /// Serves your Tribble workflows locally for development
    Serve,
    /// Builds your Tribble workflows for release deployment
    Deploy,
    /// Deletes the `.tribble/` directory in the case of a corruption
    Clean,
}
