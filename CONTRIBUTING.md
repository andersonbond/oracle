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

Oracle's UI integration starts in `skills/oracle/references/ui-design.md`, with task guidance in `simple-projects.md` and `compact-ui.md` and visual examples in `skills/oracle/assets/`. Keep recommendations independent of frontend frameworks and distinguish illustrative layouts from tested product behavior. The copied Emil Kowalski references remain unchanged; put Oracle-specific interpretation in the integration guide. When updating the upstream snapshot, review the diff, update its pinned revision and source paths, preserve the MIT notice, and verify that the binary and package include every reference and license file.

### Live agent trials

Installation and link tests do not prove that a model discovers or follows a skill. For UI guidance changes, the optional [simple-app](tests/agent_trials/simple.txt) and [enterprise-tool](tests/agent_trials/enterprise.txt) prompts exercise real implementation work without mentioning Oracle. Run them in separate fresh sessions, with a rebuilt binary and no prior conversation or supplied expected answer.

The fixture helper uses Python 3 and Git, installs the skill, and creates a plain HTML/CSS/JavaScript starter without `oracle.toml`. A local `AGENTS.md` scopes project reads and writes to the fixture without naming Oracle or supplying design answers. The helper refuses an existing destination; it does not launch a model or change agent settings:

```sh
cargo build --release --locked
trial=$(mktemp -d)
python3 tests/agent_trials/prepare.py simple "$trial/simple" --oracle ./target/release/oracle
python3 tests/agent_trials/prepare.py enterprise "$trial/enterprise" --oracle ./target/release/oracle
# Run each prompt from its corresponding workspace in a fresh coding-agent session.
# For Claude Code, prepare separate destinations with --client claude.
```

Use the client's normal skill discovery and record its version, invocation mode, prompt, tool trace, and limitations. Keep transcripts and generated apps outside the repository or under ignored `.oracle/`; do not publish account details. Live trials require an authenticated client and may consume its usage allowance, so they are not part of offline tests or CI. If implicit selection fails, test explicit invocation in another fresh workspace and report those results separately.

Review the tool trace for selection of Oracle, reading `SKILL.md` and `ui-design.md`, the appropriate layout guide, the UI cheatsheet, and relevant upstream sections. A reference being installed, mentioned, or read does not establish that it was applied. Inspect the implementation and exercise it independently:

- **Simple app:** add, check, edit, and remove items; reload to verify device persistence; simulate unavailable storage; check truthful feedback, keyboard operation, and a narrow layout. Keep infrastructure proportional to a local utility.
- **Enterprise tool:** compare realistic rows, search/filter, select records, review and cancel adjustments, fail and retry a save without losing drafts, and reach essential fields and actions on a narrow screen. Check selection scope, focus after row updates, legibility, and tabular quantities.
- **Both:** inspect desktop and narrow layouts, visible focus, relevant reduced-motion behavior, and any runtime errors. Distinguish checks that ran from claims in the agent's summary. Record untested cases; a few successful trials are observations, not a general success rate or accessibility certification.

Make narrow changes supported by observed failures, then repeat affected trials in fresh workspaces. Avoid tests that merely match generated wording or require a particular component implementation.

## Pull requests

Describe the problem, resulting behavior, and validation. Include a small reproduction for bugs. Do not include build directories, credentials, private reports, large generated artifacts, or unrelated formatting changes. Keep the friendly identity and compact, readable documentation.

Commit messages and public documentation should describe the product change and relevant validation. Keep private conversations, internal planning notes, and local working reports out of commits and packages. Store local working material under the ignored `.oracle/` directory.

Contributions are accepted under the repository's MIT License. The current v1 is a local CLI; proposed hosted services or new integrations need a demonstrated use case.
