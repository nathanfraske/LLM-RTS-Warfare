# Foundation and Developer Experience

How the repo is built, developed, and kept tight — across Windows (primary dev box), Linux, and macOS. Companion to [01-architecture](01-architecture.md); the modularity principles there are enforced by tooling defined here.

## Decisions

| Concern | Decision | Why |
|---|---|---|
| Task runner | **`just`** | First-class on Windows, discoverable (`just --list`), no Make/PHONY arcana. Every recipe is a thin wrapper over `cargo` — the project survives without `just` installed. |
| Repo automation | **`cargo xtask`** | Structure gates and generators as plain Rust (`crates/tools/xtask`) — zero extra installs, identical on every OS. `just` recipes call it. |
| Toolchain | `rust-toolchain.toml` pinning an **exact stable** (currently 1.97.1) + clippy/rustfmt | Deterministic builds and lints for everyone; bump deliberately, in its own commit. |
| Edition | 2024, workspace-inherited | — |
| Dependencies | **Pure-Rust only** unless physically impossible; lockfile committed | `cargo build` after clone must succeed with no system libraries, SDKs, or C toolchains at M0. GPU milestones add only driver requirements (wgpu), never SDKs by default; CUDA stays an opt-in feature forever. |
| Unsafe | `unsafe_code = "deny"` workspace-wide | Perf crates may locally `allow` with a written justification — the kernel-escape-hatch contract ([01 §5](01-architecture.md)). |
| Audits | `cargo-deny` (licenses, advisories, duplicate versions) | CI + occasional local `just deny`; kept out of the default `just check` to keep the inner loop fast. |

## Bootstrap (any platform)

1. Install rustup — Windows: `winget install Rustlang.Rustup`; elsewhere: rustup.rs. The pinned toolchain auto-installs on first `cargo` invocation.
2. `cargo install just` (or `winget install Casey.Just` / `scoop install just` / `brew install just`).
3. `just setup` — installs the remaining dev tools (`cargo-deny`).
4. `just check` — if this passes, your environment is correct. It is the same gate CI runs.

## Tunables live in `tuning`

Every sim-behavior number — ecology rates, channel efficiencies, mandate costs, demographic factors — lives in the `tuning` schema crate as typed, serde-ready structs with today's values as `Default`. Systems receive their domain struct by reference; **no sim crate declares a tunable constant locally.** `RunConfig.tuning` carries it, so loading a RON tuning file (or per-world variants) is one `serde` call away, and deepening a system never means hunting hardcoded numbers.

## The recipe surface

```text
just              # list recipes
just build        # cargo build --workspace
just test         # cargo test --workspace
just check        # THE gate: fmt-check + clippy -D warnings + tests + xtask gate
just gate         # structure gates only (names, layering, file-length audit)
just run 42 8760  # run a world: seed 42, one sim-year of hourly ticks
just replay 42    # determinism proof: run twice, compare event-log hashes
just deny         # license/advisory/duplicate audit
just new-crate sim weather   # generate a correctly-wired crate in a layer
```

## What xtask enforces (the modularity principles, mechanized)

- **Forbidden names** ([01 §2](01-architecture.md)): no crate directory or `.rs` file named `utils`, `util`, `common`, `helpers`, `helper`, `misc`, `shared`, or `types`. Hard failure.
- **Layering** ([01 §4](01-architecture.md)): reads every crate's manifest; `schema/*` may depend only on external crates; `sim/*` only on `schema/*` + `sim/*`; `render`/`io`/`agents` never appear in a sim crate's tree. Hard failure. (Crate-level *cycles* are already impossible — cargo rejects them.)
- **File-length audit** ([01 §2](01-architecture.md)): lists any file over 300 lines as an audit item (non-fatal — the rule is "audit for a hidden second concern," not "split blindly").
- **`new-crate` generator**: the only sanctioned way to add a crate — validates the name, places it in a layer directory, writes a manifest that inherits workspace lints/metadata, and a `lib.rs` whose first line is the one-sentence responsibility doc comment ([01 §1](01-architecture.md)). Globbed workspace members mean no root-manifest edit.

## Workspace conventions

- Layer directories mirror [01's layout](01-architecture.md): `crates/{schema,sim,gpu,io,render,agents,bin,tools}/<crate>/`. Members are globbed; creating a crate never touches the root manifest.
- Shared `[workspace.package]` metadata and `[workspace.dependencies]` versions; every crate sets `lints.workspace = true`. Dependency versions are declared once, at the root.
- Crates are created **when their first real responsibility arrives** — never speculatively. An empty crate is a grab bag waiting to happen. (M0 starts with seven crates, not the full planned tree.)
- Tests: golden-replay determinism lives in `sim-server` as an integration test and in `just replay`; conservation properties live beside the systems they check; criterion benches arrive with the first optimization work, per the escalation ladder.

## CI plan (when the repo gets a remote)

GitHub Actions, matrix `{windows, ubuntu, macos}` × pinned toolchain, running exactly `just check` plus `just deny` and (nightly job, not per-commit) a long-run replay determinism soak. Local `just check` passing must predict CI passing — no CI-only steps that can't be run locally.

## Git

Repo initialized at the workspace root; `target/` ignored. Solo trunk-based flow for now; history stays clean by keeping commits scoped to one concern (the commit log inherits the no-grab-bags rule).
