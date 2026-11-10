# Changelog

## 1.0.11 — 2026-11-10

- Added caching guidance for browser/CDN policies, cached UI data, Rust, and optional Redis, with freshness, authorization, invalidation, capacity, and outage considerations.
- Bundled a standalone Rust/Redis example and isolated native/Docker/Colima verification helper covering hits, misses, expiry, invalidation, outages, and a new client after restart.
- Linked caching guidance from the README and agent entrypoint and extended optional native Linux x64/ARM64 container validation without adding required infrastructure or Oracle dependencies.

## 1.0.10 — 2026-11-10

- Added proportional containerization guidance for Docker Desktop, Colima, and Linux Engine, including contexts, architectures, image builds, secrets, storage, and recovery.
- Bundled a multi-stage Rust Dockerfile, build-context ignore file, and Compose recipe for offline agent use, with optional native Linux x64/ARM64 container checks.
- Fixed skill removal to clean empty nested resource directories while preserving modified and unrelated files.

## 1.0.9 — 2026-11-10

- Added proportional security guidance covering secrets, sessions, authorization, input, uploads/realtime/data boundaries, dependencies, deployment, and coding-agent use, with verification examples and OWASP sources.
- Bundled and linked the security guide for offline agent use without new configuration fields or required gates.
- Refreshed the README with direct skill installation, optional project tracking, task examples, and a clearer architecture diagram.

## 1.0.8 — 2026-11-10

- Added offline image-upload guidance for optimized display copies, bounded processing/storage, and user-chosen original retention and destinations.
- Linked upload guidance from the agent skill and UI entrypoint, with optimizer candidates and proportional verification for simple apps.

## 1.0.7 — 2026-11-07

- Made the UI cheatsheet and relevant Design Engineering sections explicit in the skill entrypoint, alongside task-based layout selection.
- Clarified whole-page density review so headings, summaries, and toolbars leave room for the operational comparison task.
- Added focus continuity guidance for controls replaced during interactive updates.
- Added optional live-agent trial prompts and an isolated fixture helper for simple apps and enterprise tools, with guidance for independent behavioral verification.

## 1.0.6 — 2026-11-07

- Skill discovery now covers UI creation, refinement, and review for simple apps and enterprise tools.
- Direct UI guidance works without CLI initialization or an active UX finding; layout routing follows the screen's task rather than audience alone.
- Clarified how to apply copied upstream references within Oracle's workflow, with explicit client invocation and offline reference-link coverage.

## 1.0.5 — 2026-11-07

- Pinned MIT-licensed copies of Emil Kowalski's Design Engineering skill and UI cheatsheet, with source attribution and the complete upstream license bundled offline.
- Shared UI foundation connects component craft and purposeful motion to simple-project and compact-workspace guidance.
- Recreated all three UI illustrations with clearer hierarchy, tabular quantities, focus, selection, and save-state recovery; updated UX guidance versions while retaining existing stage policies.

## 1.0.4 — 2026-11-07

- Database design, normalization, indexing, connection pooling, and caching guidance with a relationship diagram and a small reading-list specification.
- Versioned schema and performance cards cover query-driven indexes, bounded connections, and measured cache decisions while preserving existing stage policies.
- Database guidance is bundled for offline agents; simple apps can start with one suitable database and no application cache or separate pooler.

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
