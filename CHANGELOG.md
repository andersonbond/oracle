# Changelog

## 1.0.3 — 2026-11-07

- Realtime guidance comparing polling, SSE, native WebSockets, and Socket.IO, with Rust implementation candidates.
- Event authorization, reconnection, missed-update recovery, and multi-server delivery guidance linked to existing evidence cards.
- Shared task-board specification and an offline agent reference; no networking dependencies or additional release gates.

## 1.0.2 — 2026-11-06

- Simple-website and small-app guidance with a visual example and static-site specification.
- Task-based UI recommendations for public pages, small utilities, and operational workspaces.
- Stack guidance clarifies that static websites do not need a backend to satisfy the Rust preference.
- Both UI approaches are bundled for offline agent use; existing applicability and stage policies remain unchanged.

## 1.0.1 — 2026-11-06

- Oracle compact UI guidance with desktop and adaptive layout examples, bundled for offline agent use.
- Updated UX guidance cards while retaining advisory density choices and existing stage policies.
- Public configuration and CLI reference in `REFERENCE.md`, with the sample specification in `examples/`.

## 1.0.0 — 2026-11-06

- Rust CLI with validated TOML context, human-readable and JSON output.
- Thirteen versioned guidance cards with Rust-first stack consideration, enterprise UI density, repository hygiene, and release evidence.
- Prototype, beta, and production applicability with advisory defaults and opt-in strict checks.
- Bounded local inspection with Git output limits/timeouts, tracked-file budgets including force-tracked local state, generated-path and secret-filename checks, and local Git object-size reporting.
- Evidence revisions with artifact hashes, working-tree fingerprints including Unix executable permissions, and stale/unknown states. Artifact-path restrictions apply both at recording and during reevaluation.
- Project-local Codex and Claude Code skill install/remove adapters that preserve edited content.
- Friendly Oracle identity, visual README, MIT license, automated integration tests, and cross-platform CI configuration.

This is the source implementation record. Publication of a Git tag or hosted binary release is a separate action.
