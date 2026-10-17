use std::{env, fs, path::PathBuf};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let source = manifest.join("../bundled.json");
    println!("cargo:rerun-if-changed={}", source.display());
    let docs: Vec<serde_json::Value> =
        serde_json::from_str(&fs::read_to_string(source).unwrap()).unwrap();
    let mut output = String::from(
        "pub const VERSION: &str = \"0.0.0-comparison\";\npub const DOCUMENTS: &[(&str, &str)] = &[\n",
    );
    for doc in docs {
        // Rust's debug string escaping produces a valid Rust string literal.
        output.push_str(&format!(
            "({:?}, {:?}),\n",
            doc["name"].as_str().unwrap(),
            doc["body"].as_str().unwrap()
        ));
    }
    output.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("documents.rs"),
        output,
    )
    .unwrap();
}
