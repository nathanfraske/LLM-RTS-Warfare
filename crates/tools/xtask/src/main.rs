//! Repo automation: structure gates and the crate generator (docs/01a-foundation.md).

mod gate;
mod newcrate;

use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    // crates/tools/xtask → workspace root
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.pop();
    root.pop();
    root.pop();
    root
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("gate") => gate::run(&workspace_root()),
        Some("new-crate") => match (args.get(1), args.get(2)) {
            (Some(layer), Some(name)) => newcrate::run(&workspace_root(), layer, name),
            _ => usage(),
        },
        _ => usage(),
    };
    std::process::exit(code);
}

fn usage() -> i32 {
    eprintln!("usage: cargo xtask gate");
    eprintln!("       cargo xtask new-crate <layer> <name>");
    2
}
