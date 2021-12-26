// This file serves as a testing ground for working with the parser.
// Anything in here is non-integral to the project.

use tribble::parser::Config;

fn main() {
    let cfg = Config::new("./tribble.yml");
    println!("{:#?}", cfg);
}
