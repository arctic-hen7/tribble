mod options;
mod prep;
mod errors;

/// The current version of the CLI, extracted from the crate version.
pub const TRIBBLE_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    println!("Hello, world!");
}
