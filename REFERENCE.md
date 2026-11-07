# Oracle v1 reference

Oracle is a local command-line guide. Its schema and rule versions make outputs inspectable by both people and agents. It does not orchestrate agents, run project scripts, or require an online account.

## Configuration

`oracle init` creates `oracle.toml` in an existing project directory. It does not initialize Git. Re-running it validates and preserves the current file; command-line defaults never overwrite decisions. `--dry-run` previews initialization without writes.

Unknown TOML fields and invalid enums fail clearly instead of silently changing behavior. Optional tables use the defaults below.

| Field | Allowed values / default |
| --- | --- |
| `schema_version` | Required; `1` |
| `project.name` | Required; 1–128 bytes, no control characters |
| `project.kind` | `web-app` (default), `service`, `cli`, `library` |
| `project.stage` | `prototype` (default), `beta`, `production` |
| `project.audience` | `unspecified` (default), `enterprise`, `consumer`, `developer` |
| `stack.prefer_rust` | `true`; set false to disable the preference in recommendations |
| `stack.language` / `stack.rationale` | Empty by default; both needed to complete the recorded stack decision |
| `ux.density` | `unspecified` (default), `compact`, `comfortable`, `adaptive` |
| `ux.primary_workflow` / `ux.rationale` | Empty by default; document the task and density trade-off |
| `data.personal_data` | `false`; descriptive context, not an automatic privacy audit |
| `data.database` | `false`; activates database/migration/restore guidance at the appropriate stage |
| `data.private_user_records` | `false`; activates cross-user authorization guidance at beta |
| `repository.max_tracked_mib` | `100`; positive, at most 1,000,000 |
| `repository.max_file_mib` | `50`; positive, at most 100 |
| `policy.required` | `[]`; list of known rule IDs to activate and require earlier |
| `policy.evidence_max_age_days` | `30`; 1–3650 |

The context is user-declared. Oracle does not infer that a project has no database or private records just because those declarations are absent. Review these defaults during initialization with your agent. A well-formed declaration can still be wrong.

Rust is preferred for a new service or tool when its ecosystem, deployment, and delivery costs fit. A browser UI may use another language, and an established stack should not be rewritten simply to satisfy a preference. For comparison and repeated operations, compactness is a useful starting point; a recorded comfortable/adaptive choice with a rationale is valid regardless of audience.

[UI that fits the project](skills/oracle/references/ui-design.md) connects a pinned MIT-licensed copy of Emil Kowalski's Design Engineering skill and UI cheatsheet to Oracle's task choices. The [simple-project guide](skills/oracle/references/simple-projects.md) covers small websites and apps; the [compact UI guide](skills/oracle/references/compact-ui.md) covers comparison and repeated operations. Choose per task and screen, not audience alone. These references supply guidance and illustrative layouts, not extra schema fields or automated pixel checks. `UX-001` checks the recorded decision; `UX-002` accepts a supplied review of the actual workflow, feedback, and relevant motion/accessibility behavior.

In v1, `web-app` also covers static websites and does not require a server or database. See the [simple-site example](examples/simple-site/oracle.toml). A site's size does not determine its stage: use `production` when that is its actual deployment stage. Declare data features accurately; small apps with databases or private records still need the corresponding evidence. No separate simple-project mode bypasses the existing applicability or stage policies.

For realtime features, use the [WebSocket and Socket.IO guide](skills/oracle/references/realtime.md) and [example specification](examples/realtime/oracle.toml). Record transport choices in the existing stack and workflow fields. There is no `[realtime]` table, dependency installation, transport detection, or live connection check. The existing cards accept the relevant supplied behavioral, authorization, and deployment evidence.

For persistent records or slow data access, use the [database guide](skills/oracle/references/database.md) and [small reading-list specification](examples/simple-data/oracle.toml). Describe storage choices in `stack.rationale` and set the existing data flags from actual behavior. No additional database, pooling, or caching fields are introduced. `DATA-001` accepts schema/migration and relevant index-review evidence, `DATA-002` covers restore, and `PERF-001` remains advisory. Oracle does not inspect a live schema or execute queries; a small app can use one database without a cache or separate pooler.

## Checks and stages

| Rule | Earliest stage | Required by default in strict mode | Evaluator |
| --- | --- | --- | --- |
| `STACK-001` | Prototype | Never | Language and rationale present |
| `UX-001` | Prototype | Never | Audience, density, workflow, rationale present; web apps only |
| `REPO-001` | Prototype | Beta | Tracked generated/dependency directory names |
| `REPO-002` | Prototype | Beta | Current tracked regular-file sizes versus budgets |
| `SEC-001` | Prototype | Prototype | Tracked secret-candidate filenames |
| `UX-002` | Beta | Beta | Supplied UX review; web apps only |
| `TEST-001` | Beta | Beta | Supplied build/test report |
| `SEC-002` | Beta | Beta | Supplied authorization evidence; declared private records only |
| `DATA-001` | Beta | Beta | Supplied schema/migration evidence; declared databases only |
| `DATA-002` | Production | Production | Supplied restore evidence; declared databases only |
| `OPS-001` | Production | Production | Supplied deployment/recovery evidence; web apps and services |
| `OPS-002` | Production | Production | Supplied monitoring evidence; web apps and services |
| `PERF-001` | Beta | Never | Supplied measurement report; web apps and services |

Applicability is evaluated before stage policy. An explicitly required database rule is still inapplicable when the specification declares no database. Oracle shows that assumption; it does not silently manufacture evidence.

`next` selects unresolved active findings, placing failures before stale evidence and unknowns, with required findings first within each group. Equal-ranked findings retain the curated card order. Deferred and inapplicable cards are visible in `assess`, not added to every next-action list.

`check` defaults to advisory behavior. `--strict` enforces only applicable required cards. This avoids making every suggestion a mandatory gate or imposing production operations work on a prototype.

| Exit | Meaning |
| --- | --- |
| `0` | Command completed; advisory assessment may still contain problems |
| `1` | Strict check has required findings in `fail`, `unknown`, or `stale` |
| `2` | Invalid command/input, malformed evidence/configuration, or operational error |

With `--format json`, runtime errors use a schema-versioned JSON object on stderr. CLI argument parsing/help/version are handled by Clap and remain conventional text. Successful command JSON goes to stdout. An advisory exit 0 is never a readiness certificate.

## Evidence

Before recording, actually run or review the scenario. Save a minimal sanitized artifact describing the steps, environment, scope, outcome, and limitations. The application can be in any supported language; Oracle does not execute the steps or parse framework-specific test results.

```sh
oracle evidence record TEST-001 --result pass --method report \
  --summary "Build and core behavior tests passed locally; integrations used mocks" \
  --artifact .oracle/reports/build-and-tests.txt
```

- `--result`: `pass` or `fail`.
- `--method`: `manual` (default) or `report`; both are supplied attestations.
- `--summary`: 10–2000 bytes. Do not include credentials or private records.
- `--artifact`: repeatable, 1–8 existing project-relative regular files, each at most 16 MiB. Absolute paths, parent traversal, symlinks, Git internals, evidence records, and secret-candidate files are rejected.
- Only evidence cards accept records. A supplied record cannot override an automated repository check.

Records are appended under `.oracle/evidence/RULE-ID/` with timestamped names. Initialization adds `.oracle/.gitignore` containing `*` when absent, preserving any existing ignore file; local evidence is not intended for source control. Ignore rules do not untrack files already in Git. Previous revisions remain available. The newest record is evaluated. Malformed newest records produce an error rather than falling back to an older successful result. Artifact-path restrictions are reapplied when reading stored records, before opening their artifacts.

A record includes the schema, Oracle and rule versions, collection time, inspected-project fingerprint, outcome, method, summary, and SHA-256 hashes of artifacts. Evidence becomes stale if those versions differ, the snapshot is incomplete or changed, an artifact changes/disappears, the record is future-dated, or its age exceeds policy.

V1 invalidates conservatively after any inspected-file content change, including uncommitted and unignored changes. It does not yet calculate rule-specific dependency graphs. Record evidence after the relevant changes settle. There are no forged-proof prevention claims: people or agents can write incorrect reports, and local records are editable. Hashes detect drift, not honesty.

## Inspection and trust boundaries

In a Git working tree, Oracle uses `git ls-files` to inspect tracked and unignored paths under the selected project root. It disables Git filesystem-monitor hooks for these read operations. It does not run repository scripts or fetch anything. Without a usable Git index, it falls back to a filesystem walk that prunes known generated/dependency folders and reports tracking state as unknown.

Inspection is bounded to 20,000 paths (including visited directory entries in filesystem fallback). Each Git process has a ten-second deadline; its output is limited to 8 MiB while reading, with the process stopped if a limit is exceeded. A complete fingerprint hashes regular-file contents up to 16 MiB per file and 128 MiB in total, including executable permission bits on Unix. Secret-candidate contents are not read. Local `.oracle/` state is excluded from the source fingerprint, while explicitly attached artifacts are hashed separately. Force-tracked `.oracle/` files remain visible to repository hygiene, size, and secret-filename checks. Unsafe paths, symlinks, submodules, missing/unreadable files, and hash limits make the snapshot incomplete; Oracle cannot then record fresh evidence. This is conservative and may require a smaller project scope.

**Repository measurements are distinct:**

- Inspected bytes include tracked and unignored regular files in scope.
- Tracked bytes reflect the current working-tree contents of tracked paths. They do not examine staged blob contents or every historical object.
- Local Git object bytes use `git count-objects -v` for a project owning a `.git` directory/file. They are not remote GitHub, Git LFS, releases, caches, or total filesystem usage.
- Ignored build artifacts can occupy substantial local disk without inflating the tracked repository. Oracle never removes them.

Generated-directory and secret-filename checks are heuristics. A recognized name can be intentional; an unrecognized generated path or secret can be missed. `SEC-001` is not a content scanner. Check the detail before taking action. Git ignore rules, application context, and supplied report truth remain trust inputs.

V1 is not an OS sandbox and does not promise protection against a concurrently malicious process replacing filesystem paths during a command. It rejects observed symlinks/traversal and uses non-overwriting creation. Run it on a project you are authorized to inspect, and keep the agent's execution permissions separate from report contents.

## Agent installation and updates

```sh
oracle agent install codex --dry-run
oracle agent install codex
oracle agent install claude
```

Codex/generic uses `.agents/skills/oracle/`; Claude Code uses `.claude/skills/oracle/`. The binary embeds the short skill, workflow, UI, realtime, and database guides, the pinned Emil Kowalski references and license, and three SVG layout examples. It does not require a source checkout or network access at runtime. Both adapters share the same instructions and respect the user's scope.

Installation requires an existing destination directory, but no Git repository or `oracle.toml`. For guidance without the CLI, copy the complete source skill into the appropriate location while preserving existing customizations. Only `SKILL.md` is the Oracle discovery entry point; copied upstream Markdown files are supporting references. UI tasks route directly to `references/ui-design.md`, then the applicable layout guide and relevant upstream sections. CLI assessment is optional for UI work, including when `UX-001` already passes or `UX-002` is deferred.

Codex can select skills from their descriptions; in Codex CLI/IDE, use an explicit `$oracle` mention or `/skills`. Claude Code supports explicit `/oracle` invocation. If discovery fails, check the installed skill location, client settings, and skill selector, or ask the agent to read the installed `SKILL.md` directly. See the [Codex discovery documentation](https://learn.chatgpt.com/docs/build-skills) and [Claude Code skill documentation](https://code.claude.com/docs/en/skills). Oracle does not register the copied Emil reference as a second skill or change the client's automatic-invocation settings.

Every destination is preflighted. Exact matches are left alone; different contents cause an error before mutation. This also protects user edits during removal. `oracle agent remove CLIENT` removes only files identical to the current binary's embedded package and then attempts to remove empty package-owned directories. It leaves client roots and unrelated files alone.

For upgrades with changed skill content, review and back up your existing skill, remove the old unmodified package using the old binary, then install the new one. Merge custom changes manually. V1 does not provide forced overwrites or global installation. Restart the client session if needed and check its skill selector; automated adapter tests do not establish live model behavior or universal client support.

## Building, packaging, and publication

`cargo install --path . --locked` builds the CLI from this checkout. `cargo build --release --locked` leaves a native binary at `target/release/oracle` (`oracle.exe` on Windows). `cargo package --locked --allow-dirty` can verify a local source package during development; review all included files before publication.

CI is configured on Linux, macOS, and Windows with an additional Rust 1.85 compatibility job. Manual workflow runs upload native binary artifacts. There is no automatic registry publish, GitHub release, deployment, or Git history modification. Publishing is a separate maintainer action.

Check the [CI results](https://github.com/andersonbond/oracle/actions/workflows/ci.yml) for a particular revision. Filesystem adapter tests do not establish live agent behavior, and illustrative layouts do not establish usability in a real application.
