set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# list recipes
default:
    just --list

# one-time dev-tool install (the toolchain itself auto-installs via rust-toolchain.toml)
setup:
    rustup show
    cargo install cargo-deny --locked

build:
    cargo build --workspace

test:
    cargo test --workspace

fmt:
    cargo fmt --all

# THE gate — CI runs exactly this (docs/01a-foundation.md)
check:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo xtask gate

# structure gates only: forbidden names, layering, file-length audit
gate:
    cargo xtask gate

# license / advisory / duplicate-version audit (CI + occasional local)
deny:
    cargo deny check

# run a world: just run 42 8640
run seed="42" ticks="8640":
    cargo run -p sim-server --release -- --seed {{seed}} --ticks {{ticks}}

# determinism proof: two runs must print identical hashes
replay seed="42" ticks="8640":
    cargo run -p sim-server --release -- --seed {{seed}} --ticks {{ticks}} --hash-only
    cargo run -p sim-server --release -- --seed {{seed}} --ticks {{ticks}} --hash-only

# render a world map to maps/terrain-<seed>.bmp: just map 42 512
map seed="42" size="512":
    cargo run -p map-export --release -- --seed {{seed}} --size {{size}}

# a council session: replay world + directives.json, write per-nation reports
council seed="42" ticks="43200":
    cargo run -p sim-server --release -- --seed {{seed}} --ticks {{ticks}} --directives directives.json --report-dir reports

# open the live spectator viewer: just view 42
view seed="42":
    cargo run -p viewer --release -- --seed {{seed}} --directives directives.json

# generate a correctly-wired crate: just new-crate sim weather
new-crate layer name:
    cargo xtask new-crate {{layer}} {{name}}
