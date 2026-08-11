//! The sanctioned way to add a crate: correctly layered, lint-inheriting,
//! and opening with the one-sentence responsibility line.

use std::fs;
use std::path::Path;

use crate::gate::FORBIDDEN_NAMES;

const LAYERS: [&str; 8] = [
    "schema", "sim", "gpu", "io", "render", "agents", "bin", "tools",
];

pub fn run(root: &Path, layer: &str, name: &str) -> i32 {
    if !LAYERS.contains(&layer) {
        eprintln!("unknown layer '{layer}' — one of {LAYERS:?}");
        return 2;
    }
    if FORBIDDEN_NAMES.contains(&name) {
        eprintln!("'{name}' is a forbidden grab-bag name — name the direct owner");
        return 2;
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    {
        eprintln!("'{name}' must be kebab-case ascii");
        return 2;
    }
    let dir = root.join("crates").join(layer).join(name);
    if dir.exists() {
        eprintln!("{} already exists", dir.display());
        return 2;
    }

    let manifest = format!(
        "[package]\nname = \"{name}\"\nversion.workspace = true\nedition.workspace = true\nlicense.workspace = true\npublish.workspace = true\n\n[dependencies]\n\n[lints]\nworkspace = true\n"
    );
    let entry = "//! TODO: one-sentence responsibility — if it needs \"and\", split it \
                 (docs/01-architecture.md, Modularity principles).\n";

    let src = dir.join("src");
    if let Err(e) = fs::create_dir_all(&src) {
        eprintln!("create {}: {e}", src.display());
        return 1;
    }
    let entry_file = if layer == "bin" {
        src.join("main.rs")
    } else {
        src.join("lib.rs")
    };
    let entry_body = if layer == "bin" {
        format!("{entry}\nfn main() {{}}\n")
    } else {
        entry.to_string()
    };
    if let Err(e) = fs::write(dir.join("Cargo.toml"), manifest)
        .and_then(|()| fs::write(&entry_file, entry_body))
    {
        eprintln!("write crate files: {e}");
        return 1;
    }

    println!("created {} (layer {layer})", dir.display());
    println!(
        "→ fill in the responsibility line at the top of {}",
        entry_file.display()
    );
    println!("→ workspace membership is globbed; no root manifest edit needed");
    0
}
