//! Structure gates: forbidden names, layer rules, file-length audit
//! (mechanizing docs/01-architecture.md "Modularity principles").

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub const FORBIDDEN_NAMES: [&str; 8] = [
    "utils", "util", "common", "helpers", "helper", "misc", "shared", "types",
];

const FILE_LENGTH_AUDIT: usize = 300;

/// Allowed internal-dependency layers per layer (docs/01-architecture.md §4).
/// `None` means unrestricted (binaries and tools compose everything).
fn allowed_layers(layer: &str) -> Option<&'static [&'static str]> {
    match layer {
        "schema" | "gpu" | "render" => Some(&["schema"]),
        "sim" | "io" => Some(&["schema", "sim"]),
        "agents" => Some(&["schema", "io"]),
        _ => None, // bin, tools
    }
}

pub fn run(root: &Path) -> i32 {
    let crates = discover_crates(&root.join("crates"));
    let mut violations = Vec::new();
    let mut audits = Vec::new();

    for info in &crates {
        check_names(info, &mut violations);
        check_layering(info, &crates, &mut violations);
        audit_file_lengths(info, &mut audits);
    }

    for line in &audits {
        println!("audit: {line}");
    }
    if violations.is_empty() {
        println!(
            "gate: OK — {} crates, {} audit item(s)",
            crates.len(),
            audits.len()
        );
        0
    } else {
        for line in &violations {
            eprintln!("violation: {line}");
        }
        eprintln!("gate: FAILED — {} violation(s)", violations.len());
        1
    }
}

struct CrateInfo {
    name: String,
    layer: String,
    dir: PathBuf,
}

fn discover_crates(crates_dir: &Path) -> Vec<CrateInfo> {
    let mut found = Vec::new();
    for layer_entry in read_dir_sorted(crates_dir) {
        if !layer_entry.is_dir() {
            continue;
        }
        let layer = file_name(&layer_entry);
        for crate_entry in read_dir_sorted(&layer_entry) {
            if crate_entry.join("Cargo.toml").is_file() {
                found.push(CrateInfo {
                    name: file_name(&crate_entry),
                    layer: layer.clone(),
                    dir: crate_entry,
                });
            }
        }
    }
    found
}

fn check_names(info: &CrateInfo, violations: &mut Vec<String>) {
    let mut offenders = Vec::new();
    if FORBIDDEN_NAMES.contains(&info.name.as_str()) {
        offenders.push(info.dir.clone());
    }
    walk_rs(&info.dir.join("src"), &mut |path| {
        let stem = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        if FORBIDDEN_NAMES.contains(&stem.as_str()) {
            offenders.push(path.to_path_buf());
        }
    });
    for path in offenders {
        violations.push(format!(
            "forbidden name (grab bag) at {} — name the direct owner instead",
            path.display()
        ));
    }
}

fn check_layering(info: &CrateInfo, all: &[CrateInfo], violations: &mut Vec<String>) {
    let Some(allowed) = allowed_layers(&info.layer) else {
        return;
    };
    let layers: BTreeMap<&str, &str> = all
        .iter()
        .map(|c| (c.name.as_str(), c.layer.as_str()))
        .collect();
    let manifest = fs::read_to_string(info.dir.join("Cargo.toml")).unwrap_or_default();
    let Ok(parsed) = manifest.parse::<toml::Value>() else {
        violations.push(format!("{}: unparseable Cargo.toml", info.name));
        return;
    };
    let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) else {
        return;
    };
    for dep in deps.keys() {
        if let Some(dep_layer) = layers.get(dep.as_str())
            && !allowed.contains(dep_layer)
        {
            violations.push(format!(
                "{} (layer {}) depends on {} (layer {}) — allowed: {:?}",
                info.name, info.layer, dep, dep_layer, allowed
            ));
        }
    }
}

fn audit_file_lengths(info: &CrateInfo, audits: &mut Vec<String>) {
    walk_rs(&info.dir.join("src"), &mut |path| {
        let lines = fs::read_to_string(path).map_or(0, |s| s.lines().count());
        if lines > FILE_LENGTH_AUDIT {
            audits.push(format!(
                "{} is {lines} lines (> {FILE_LENGTH_AUDIT}) — audit for a hidden second concern",
                path.display()
            ));
        }
    });
}

fn walk_rs(dir: &Path, visit: &mut impl FnMut(&Path)) {
    for entry in read_dir_sorted(dir) {
        if entry.is_dir() {
            walk_rs(&entry, visit);
        } else if entry.extension().is_some_and(|e| e == "rs") {
            visit(&entry);
        }
    }
}

fn read_dir_sorted(dir: &Path) -> Vec<PathBuf> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .map(|iter| iter.flatten().map(|e| e.path()).collect())
        .unwrap_or_default();
    entries.sort();
    entries
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default()
}
