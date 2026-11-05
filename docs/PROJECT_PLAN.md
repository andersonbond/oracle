# Oracle project plan

**Status:** historical proposal · **Date:** 6 November 2026

This records the original proposal before implementation. The founder subsequently authorized v1 and prioritized Rust-first stack consideration, compact enterprise UI, repository hygiene, and lighter early-stage rules. The implemented contract is now in the [README](../README.md) and [v1 reference](V1.md). Prospective statements below are historical, not current capability claims. Pilot interviews and live client evaluations described here have not been claimed as completed.

## 1. The direction we have agreed

- Oracle will be a **tool written in Rust**, with a declarative project specification. It is not a new general-purpose programming language.
- Its first journey is **working prototype → dependable first release** for a solo builder using a coding agent.
- The long-term ambition covers the software lifecycle: product and design, technical choices, architecture, data, security, testing, delivery, operations, and scaling.
- People should eventually be able to install Oracle and connect its guidance to their coding agents, or clone its open-source repository.

The remaining choices below are recommendations to validate, not settled commitments.

## 2. The product promise

> Understand the next engineering decision, act on it with your coding agent, and keep evidence that the outcome works.

The first user is capable of making an application run but may not yet know which release risks to investigate. The experienced developer is a secondary user who wants concise findings and consistent handoffs across agents.

The first useful outcome should arrive in minutes: one relevant gap, a plain-language explanation, a bounded task, and a credible way to verify it. A long generated architecture document is not sufficient evidence of value.

Your experience becomes the project's most important asset when expressed as **conditional engineering judgment**: when to consider a technique, what alternatives exist, what it costs, how to verify it, and when to revisit it. The same format should be usable by both a new coder and an agent.

## 3. Positioning to test

Current projects already address much of this space:

| Project | Scope described in its own documentation | Implication for Oracle |
| --- | --- | --- |
| [Spec Kit](https://github.com/github/spec-kit) | Structured processes, templates, integrations, and extensible workflows | A planning command or reusable prompt alone is not a compelling distinction |
| [OpenSpec](https://github.com/Fission-AI/OpenSpec) | Specifications, change proposals, implementation tasks, and an iterative workflow | Reuse existing specifications when feasible instead of making users duplicate them |
| [BMad](https://github.com/bmad-code-org/BMAD-METHOD) | Guidance across product, architecture, implementation, and verification | Broad lifecycle coverage is shared territory |
| [Agent Skills](https://agentskills.io/specification) | A portable format for instructions and supporting resources | Use an existing integration format before inventing another one |

These descriptions were reviewed on 6 November 2026. They are not a feature-completeness comparison. Oracle's proposed distinction is a combination of **teaching at the moment of a decision**, **context-sensitive prioritization**, and **evidence that becomes stale when its inputs change**. Validate this against a plain checklist and an existing workflow tool.

## 4. The first user's experience

1. **Orient.** Inspect supported manifests and repository structure without executing project scripts. Ask the builder to confirm the app's critical journey, data sensitivity, intended users, deployment constraints, and release goal. Surface unknowns instead of guessing.
2. **Prioritize.** Show at most three useful actions at once. Rank by consequence, applicability, confidence, and dependencies. Missing context can itself be the next action.
3. **Explain.** Offer a short explanation by default, with an optional deeper lesson and references. Experienced users can choose concise output.
4. **Act.** Give the existing coding agent a small task, relevant project decisions, acceptance criteria, and evidence requirements. The builder retains authority over product and architectural trade-offs.
5. **Verify.** Evaluate supported checks and record the source, scope, and age of the evidence. Human observations remain explicitly human observations.
6. **Revisit.** Mark affected evidence stale when relevant code, dependencies, configuration, or requirements change. Resume from the repository's recorded state across agent sessions.

Example: a notes app declares private records. Oracle sees no current cross-user authorization result. It explains the difference between sign-in and authorization and asks for a two-user test. The absence of the result is an evidence gap; it does not establish an exploitable vulnerability.

## 5. A deliberately narrow MVP

Use one reference stack for executable checks, selected from the pilot projects. Keep the specification and guidance format independent of that stack. Unsupported frameworks receive an explicit coverage notice and applicable advisory guidance.

| Include | Defer |
| --- | --- |
| One first-release profile | A complete methodology for every application type |
| Roughly 8–12 carefully reviewed guidance cards | Hundreds of generic rules or an open plugin marketplace |
| A handful of deterministic checks and one evidence-report adapter | Automatic understanding of arbitrary architectures |
| Human-readable output plus stable JSON | A dashboard, hosted account system, or mobile app |
| A project-local skill and two tested client adapters | Universal agent compatibility or an agent orchestrator |
| TOML intent, Markdown decisions, versioned evidence | A custom DSL, compiler, or new app framework |
| Offline core; the user's existing agent supplies language reasoning | A mandatory model subscription or Oracle-hosted inference |

Candidate first guidance cards: critical journey and error states; keyboard accessibility; authorization boundaries; secrets handling; dependency review; schema constraints and migrations; backup and restore; repeatable build and tests; deployment and rollback; health signals and ownership.

Only cards with a well-defined mechanism should claim automated verification. A named test and its exit code alone cannot establish that it tests the intended property; evidence should include the scenario, result, scope, and review basis.

### Proposed command contract

All names and flags are provisional; no commands below are available yet.

| Command | Responsibility |
| --- | --- |
| `oracle init` | Create the project specification and selected agent adapter; show changes and preserve existing files |
| `oracle assess` | Read supported project inputs, establish context, and list findings and coverage limits |
| `oracle next` | Return the most useful next actions and their dependencies |
| `oracle explain <rule-id>` | Explain a rule, alternatives, evidence requirements, and sources |
| `oracle check` | Evaluate supported checks and evidence without running arbitrary repository commands by default |

Add machine-readable JSON with a schema version, stable rule IDs, and documented errors. Later CI mode can enforce explicitly configured requirements. Its exit status must distinguish unmet requirements from tool errors; an advisory finding should not silently become a release blocker.

## 6. What the Rust program owns

Begin with one binary and clear modules. Split into workspace crates only when reuse or compile boundaries justify it.

```text
CLI and output formatting
        ↓
Project model and specification validation
        ↓
Read-only discovery → applicability and prioritization → findings
        ↑                         ↑                        ↓
Versioned playbooks        Evidence validation       JSON / Markdown
        ↑                         ↑
Reviewed sources          Reports and attestations

Agent adapters package this workflow for the user's existing agent.
```

Rust handles parsing, validation, discovery, deterministic rule evaluation, evidence bookkeeping, and portable output. The coding agent handles conversation, source interpretation, proposed changes, and explanations beyond the curated material. The human resolves trade-offs and reviews the result.

Use existing formats: TOML for small structured configuration, Markdown for explanations and decision records, JSON for reports. Avoid embedding executable expressions in the initial specification. TOML syntax validity is separate from validation against Oracle's future schema.

Suggested project files, **not generated in this repository**:

```text
oracle.toml                   Intent and constraints; reviewed in Git
.oracle/decisions/             Short decisions: choice, reason, alternatives
.oracle/evidence/              Local reports; ignored by default
.oracle/reports/               Shareable summaries after review
<client skill directory>/     Oracle skill in a client-supported location
```

Evidence paths must stay within approved roots. Avoid reading secret values during discovery; do not collect full environment dumps. Raw evidence may contain sensitive output, so sharing must be deliberate. Reports should link to minimal, redacted artifacts when necessary.

## 7. Guidance and evidence are the core data model

Each guidance card should have:

- A stable ID, version, title, intent, and applicability conditions.
- A plain-language explanation and a concrete failure example.
- Options, trade-offs, and the circumstances in which doing nothing is reasonable.
- Required observations, a suggested agent task, and acceptance criteria.
- Its verification method and limits: automated, manually reviewed, or advisory.
- Sources, a review date, and the inputs that invalidate previous evidence.

Each finding should separate **severity**, **confidence**, and **verification status**. A severe potential consequence with unknown applicability is a request for clarification, not a confirmed defect.

| Status | Meaning |
| --- | --- |
| `pass` | The stated criterion has applicable, current evidence from its declared verification method |
| `fail` | Evidence demonstrates that the stated criterion is unmet |
| `unknown` | Insufficient context or evidence to decide |
| `stale` | Prior evidence exists, but its relevant inputs or freshness window have changed |
| `not_applicable` | The criterion does not apply, with a recorded rationale |

Risk acceptance is a separate decision record with an owner, reason, scope, and revisit trigger; it does not transform a failed or unknown check into a pass.

Evidence should include rule and tool versions, the observed code state, input digests, environment scope, collection time, result, and artifact references. A commit hash alone misses uncommitted changes. Checks must declare their inputs; if precise dependencies are not yet known, invalidate conservatively and explain why.

Recording provenance improves traceability, not trust by itself. An agent-authored statement is an attestation. An independently executed test report is a different evidence type. Neither should be represented as a comprehensive security or reliability guarantee.

## 8. Installation and agent integration

The desired adoption flow is: download a versioned native binary → initialize a project → install the compatible skill → ask the agent for the next release priority. Cloning the source remains an option for contributors.

Start with macOS, Linux, and Windows release targets if the implementation spike demonstrates manageable support. Offer checksums and documented platform support. Confirm binary, package, and crate names before promising package-manager commands. Do not publish an installation command for an unrelated package named `oracle`.

Use the [Agent Skills format](https://agentskills.io/specification) for the portable instructional core. Exact discovery paths, lifecycle hooks, and invocation behavior need client-specific verification. Select two clients used by the pilot group and run the same end-to-end scenario in each.

Keep installation project-local by default. It should be repeatable, previewable, reversible, and preserve user-authored instructions. Skill upgrades need versioning and a clear ownership boundary for generated files. Do not load every engineering topic into every agent prompt; select the relevant card and context.

Treat repository text and third-party reports as untrusted inputs. They must not gain authority to execute commands or change Oracle policy. Future command runners require explicit scope, bounded execution, timeouts, and clear network behavior. Load tests, penetration tests, migrations, and production changes need a separate, explicit execution design.

MCP can be evaluated later if a real client needs structured tool access that the CLI and skill cannot provide. It is not necessary to validate the product's central idea.

## 9. Milestones with exit criteria

| Milestone | Deliverable | Evidence needed to proceed |
| --- | --- | --- |
| 0 · Validate the problem | 5 pilot interviews and a manual walkthrough on consenting prototypes | Repeated release gaps, useful explanations, and a reason users prefer this to their existing workflow |
| 1 · Validate the guidance | A small reviewed playbook and proposed report format | Builders can apply the advice; independent reviewers agree on applicability and acceptable evidence |
| 2 · Build a vertical slice | Rust CLI: one project → one useful finding → agent task → verified report | Same scenario works on the chosen reference stack and two selected agent clients |
| 3 · Release a limited alpha | Installer, versioned schema, small rule set, evidence lifecycle, examples | Clean install/upgrade/uninstall; documented platform checks; no misleading success states in fixtures |
| 4 · Expand from evidence | Additional stacks and guidance based on pilot demand | The first workflow is used repeatedly and the maintenance burden is understood |

Proposed pilot targets: a first useful action within ten minutes after setup; most participants resolve at least one meaningful gap and can explain why the change matters; reviewers judge at least 80% of the top-three recommendations relevant. Treat these as initial targets, not achieved results or statistically established benchmarks.

Evaluate false alarms, missed known gaps, time to verified resolution, explanation usefulness, and repeat use. Include intentionally incomplete and unsupported projects. Compare against a plain checklist and the user's current agent workflow so Oracle has to demonstrate incremental value.

Before an alpha release, test malformed specifications, path boundaries, stale evidence, dirty working trees, unsupported stacks, offline use, and installation conflicts. Test platform behavior on the advertised platforms. Never count a synthetic fixture as proof that a real application is ready to release.

## 10. How to keep the open-source project useful

Prefer contributions that include a real failure scenario, applicability boundaries, references, and a reviewable verification method. Guidance deserves technical review just as code does. Give rules a documented deprecation and revision policy so updates do not silently change the meaning of previous results.

Before public installation or accepting substantive contributions, settle a license, add contribution and vulnerability-reporting instructions, define supported versions, and establish a review process. MIT or Apache-2.0 are candidates for the founder to consider; this draft makes no licensing choice. Document the project's independent identity and resolve distribution naming before launch.

Keep the first release free of hosted infrastructure unless pilots establish a need. A local core reduces operational work and lets users keep project context in their own environment. The long-term knowledge base can grow without turning every topic into a mandatory gate.

## 11. The next planning conversation

1. Which real prototype should become the first worked example, and which stack does it use?
2. What are three release failures you have repeatedly seen that a novice's coding agent would miss?
3. Which two coding-agent clients do the first pilot users actually use?
4. Which open-source license do you want to adopt before accepting contributions?

Start by walking one prototype through the proposed flow manually. That will tell us what the CLI needs to automate and where human judgment must remain visible.
