# Contributing to Oracle

Start with a real engineering decision or failure scenario. Explain who is affected, when a recommendation applies, its alternatives, and what evidence would justify a result.

Keep changes proportional to the work. Documentation edits need link and accuracy checks. Behavioral changes need focused tests covering the changed behavior, followed by the existing suite. Avoid new dependencies, mandatory gates, or broad rewrites without a concrete benefit.

## Develop

Install Rust 1.85+ and Git. Use the committed lockfile:

```sh
cargo test --locked
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
```

The CLI is in `src/main.rs`; configuration, inspection, rules, evidence, and skill installation are separate small modules. Guidance is embedded in `src/rules.rs`; the portable skill is in `skills/oracle/` and embedded into the binary. Edit the source skill before rebuilding.

## Guidance changes

A card needs an explicit criterion, applicability, useful next action, acceptance criteria, an authoritative source, and honest verification limits. Bump its rule version if the meaning changes. Evidence also becomes stale across Oracle versions. Do not count a configuration declaration as behavioral verification or convert an accepted risk into a pass.

Oracle's UI recommendations live in `skills/oracle/references/simple-projects.md` and `skills/oracle/references/compact-ui.md`, alongside bundled visual examples in `skills/oracle/assets/`. Keep recommendations independent of frontend frameworks and distinguish illustrative layouts from tested product behavior.

## Pull requests

Describe the problem, resulting behavior, and validation. Include a small reproduction for bugs. Do not include build directories, credentials, private reports, large generated artifacts, or unrelated formatting changes. Keep the friendly identity and compact, readable documentation.

Commit messages and public documentation should describe the product change and relevant validation. Keep private conversations, internal planning notes, and local working reports out of commits and packages. Store local working material under the ignored `.oracle/` directory.

Contributions are accepted under the repository's MIT License. The current v1 is a local CLI; proposed hosted services or new integrations need a demonstrated use case.
