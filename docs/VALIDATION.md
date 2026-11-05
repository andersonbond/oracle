# v1 validation record

Date: 6 November 2026. Source version: 1.0.0. Local environment: Apple Silicon macOS, Rust/Cargo 1.93.1. This records implementation verification, not user-study results or application certification.

## Completed locally

- All 31 CLI integration tests and 2 process-boundary tests passed, including initialization/idempotence, configuration errors, stage policies, explicit requirements, Rust-preference opt-out, justified non-Rust/density choices, applicability, tracked generated files, size budgets, secret-filename redaction, evidence drift/expiry/history, malformed records, scope boundaries, and safe skill install/remove behavior.
- The pre-publication review added regressions for force-tracked `.oracle/` content, executable-permission changes, and forbidden artifacts in edited evidence records. All four regressions failed before their fixes and passed afterward. Process tests cover output limits and timeouts.
- Formatting passed and Clippy passed for all targets with warnings denied.
- An optimized native release binary was built successfully (approximately 1.64 MiB on this machine).
- `cargo install --path . --locked --offline --root <temporary-directory>` succeeded without changing the user's normal Cargo installation.
- The installed binary reported `oracle 1.0.0`, initialized a fresh Git project, returned `STACK-001` and `UX-001` as the first two actions, and passed the prototype strict check.
- Codex and Claude project skill installation and removal succeeded through the installed binary in that temporary project.
- The skill-creator validator accepted the portable skill. TOML examples and the CI YAML parsed successfully.

## Limits

The two adapters were tested as filesystem integrations. Live Codex/Claude model behavior, automatic discovery in every client configuration, and user learning outcomes were not evaluated. Linux, Windows, and minimum-Rust compatibility jobs are configured in CI but were not executed on remote runners during this local implementation.

Evidence tests use clearly synthetic fixtures to verify status transitions and path handling. They do not establish that a real application's UX, authorization, migrations, backups, or deployment are correct. Oracle v1 accepts those conclusions as supplied attestations and checks their recorded provenance/freshness.

## Reproduce

```sh
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --release --locked
cargo package --locked --allow-dirty
```

`--allow-dirty` is for validating a local working-tree package. It does not commit, push, or publish anything. Review the package file list before release. Package artifacts and local evidence belong in ignored directories, not Git.
