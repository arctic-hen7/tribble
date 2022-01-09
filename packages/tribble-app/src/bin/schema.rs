use schemars::schema_for;
use tribble_app::parser::Config;

fn main() {
    let schema = schema_for!(Config);
    let schema_pretty = serde_json::to_string_pretty(&schema).unwrap();
    std::fs::write("../../schema.json", schema_pretty).expect("couldn't write json schema");
}
