# Oracle workflow

The CLI is local and offline. It does not run project scripts, call model APIs, deploy, clean files, or change Git history.

## Context and next action

`oracle assess --format json` returns schema version 1, inspected scope, coverage notes, and findings. `oracle next --limit 3 --format json` prioritizes unresolved active findings. Use `oracle explain RULE-ID` for the rationale, suggested action, acceptance criteria, and source. Update the project specification when the user's goals or stage change.

Prototype checks focus on stack decisions, UI intent, repository size/hygiene, and secret-candidate filenames. Beta introduces workflow review, builds/tests, relevant authorization and migrations. Production adds restore, deployment recovery, and health signals. `policy.required` can activate named rules sooner. Applicability follows the declared project kind and data; inspect those assumptions before trusting a not-applicable result.

## Choose the UI for the task

For a portfolio, informational site, or small single-purpose app, use [simple-project guidance](simple-projects.md): clear content, a main action, and a short working journey. For screens where comparing records and repeating actions matters, use [compact UI guidance](compact-ui.md). Choose per screen; an enterprise audience does not automatically need a dense interface, and a consumer app can have complex workflows.

Both guides are advisory. Use the existing `ux` fields to record the decision and review the actual behavior under `UX-002`. Static websites use `project.kind = "web-app"` in v1. Declare data needs accurately, and use the actual release stage; a small public site can be production software.

## Realtime features when needed

For chat, presence, collaboration, notifications, or live status, use [the realtime guide](realtime.md). Compare transport choices, evaluate Rust implementations where suitable, and design authorization and recovery before promising live behavior. Record the decision in the existing stack and workflow fields; no new configuration table or always-on realtime requirement is introduced. Use the existing evidence cards for the behavior actually exercised.

## Database work at the right scale

Use [the database guide](database.md) for schema design, normalization, indexing, connection pooling, and caching. Start from the app's reads, writes, ownership, and invariants. Keep a small schema and one suitable database when sufficient; measure before adding a cache or separate pooler. Set the existing data flags accurately, then record schema/migration evidence under `DATA-001`, restore evidence under `DATA-002`, and applicable measurements under the advisory `PERF-001` card. These guides do not add required checks or database infrastructure.

## Evidence

Run or review the relevant scenario using the project's tools within the existing authorization. Save a minimal sanitized report under `.oracle/reports/` describing commands or review steps, environment, scenario, outcomes, and limitations. Do not collect credentials or private user data. Oracle does not execute those steps for you.

Record the actual result:

```sh
oracle evidence record TEST-001 --result pass --method report \
  --summary "Build and core behavior tests passed locally; external services were mocked" \
  --artifact .oracle/reports/build-and-tests.txt
```

Use `--result fail` for a failed criterion and `--method manual` for a manual review. A supplied report is still an attestation; Oracle verifies artifact hashes, the inspected working-tree fingerprint, versions, and freshness, not semantic truth. It includes uncommitted code in the fingerprint and conservatively stales evidence after any inspected project change. Ignored files, local `.oracle/` state, and secret contents are outside that fingerprint; referenced artifacts are hashed separately. Record after relevant edits settle.

`pass`, `fail`, `unknown`, `stale`, `not_applicable`, and `deferred` are distinct. Missing evidence means unknown, not a confirmed bug. Never convert an accepted risk or skipped check into a pass. Keep the limitation explicit in the handoff.

`oracle check` is advisory and returns zero after a successful assessment. `oracle check --strict` returns 1 for unresolved applicable required checks; malformed inputs/tool errors return 2. JSON errors are written to stderr. Passing a configured gate is not proof of overall application readiness.
