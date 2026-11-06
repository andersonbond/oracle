use crate::{
    Result, VERSION,
    config::{Audience, Config, Density, Kind, Stage},
    evidence,
    project::{self, Snapshot},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Pass,
    Fail,
    Unknown,
    Stale,
    NotApplicable,
    Deferred,
}

#[derive(Debug, Serialize)]
pub struct Rule {
    pub id: &'static str,
    pub version: u32,
    pub title: &'static str,
    pub why: &'static str,
    pub action: &'static str,
    pub done_when: &'static str,
    pub source: &'static str,
    pub starts_at: Stage,
    pub required_at: Option<Stage>,
    pub accepts_evidence: bool,
}

macro_rules! card {
    ($id:literal, $title:literal, $why:literal, $action:literal, $done:literal, $source:literal, $start:ident, $required:expr, $evidence:literal) => {
        card!(
            1, $id, $title, $why, $action, $done, $source, $start, $required, $evidence
        )
    };
    ($version:literal, $id:literal, $title:literal, $why:literal, $action:literal, $done:literal, $source:literal, $start:ident, $required:expr, $evidence:literal) => {
        Rule {
            id: $id,
            version: $version,
            title: $title,
            why: $why,
            action: $action,
            done_when: $done,
            source: $source,
            starts_at: Stage::$start,
            required_at: $required,
            accepts_evidence: $evidence,
        }
    };
}

pub const RULES: &[Rule] = &[
    card!(
        2,
        "STACK-001",
        "Choose the stack deliberately; consider Rust first",
        "Rust is the preferred starting point for new services, CLIs, and systems components. Ownership, explicit errors, and native deployment can help agents build maintainable software. Ecosystem fit and the team's constraints still decide the choice.",
        "Choose the smallest implementation that serves the actual feature. A static website may need only HTML, CSS, and minimal JavaScript; do not add a backend solely to use Rust. When a new backend or CLI is needed, evaluate Rust first. Use a suitable browser stack and consider Rust/Wasm when a concrete need justifies it. Compare library maturity, delivery time, deployment, and team skills. Keep an effective existing stack unless a migration has a measured benefit. Record stack.language and stack.rationale; prefer_rust=false opts out of the preference.",
        "A language and meaningful rationale are recorded. This checks the decision record, not architectural correctness.",
        "https://rust-lang.org/what/cli/",
        Prototype,
        None,
        false
    ),
    card!(
        3,
        "UX-001",
        "Match the interface to its purpose",
        "A portfolio, small utility, and inventory desk serve different tasks. Content and a clear main action help simple websites and apps; comparison and repeated operations benefit from compact workspaces. Audience alone does not determine complexity or density.",
        "Choose per screen using Oracle's simple-project and compact UI guides. For a simple website or small app, prioritize clear content, a primary action, responsive layouts, and intentional whitespace. For operational screens, bring comparison data and repeated actions close together. Adapt density to input method and user preference; compact, comfortable, and adaptive choices are valid. Preserve readable text, keyboard access, and usable targets. Record project.audience, ux.density, ux.primary_workflow, and ux.rationale. Suggestions are advisory, not pixel-based gates or a frontend-stack requirement.",
        "Audience, density, workflow, and rationale are recorded. Actual usability remains a separate review under UX-002.",
        "https://github.com/andersonbond/oracle/blob/main/skills/oracle/references/workflow.md#choose-the-ui-for-the-task",
        Prototype,
        None,
        false
    ),
    card!(
        "REPO-001",
        "Keep generated dependencies and builds out of Git",
        "Local build caches are normal. Tracking dependency trees and generated output makes clones, reviews, and agent context unnecessarily large.",
        "Review the listed tracked generated paths. Add appropriate ignore patterns and, after preserving needed files, remove generated content from the index. Do not delete source, credentials, data, or evidence. A new ignore rule does not untrack an existing file; Oracle only reports and never deletes or rewrites Git history.",
        "No recognized generated/dependency directories are tracked in the inspected project scope.",
        "https://git-scm.com/docs/gitignore",
        Prototype,
        Some(Stage::Beta),
        false
    ),
    card!(
        "REPO-002",
        "Keep the tracked working tree within a size budget",
        "Large tracked assets slow cloning and reviews; deleted files can remain in Git history. Local build-cache size, tracked working-tree size, and Git object storage are different measurements.",
        "Inspect the largest tracked files. Prefer release assets, external data storage, or Git LFS for appropriate large binaries. Keep intentional source assets. Set explicit repository.max_file_mib and max_tracked_mib budgets. Inspect history separately before considering any rewrite; Oracle does not measure remote GitHub or LFS storage.",
        "Current tracked regular files fit the configured budgets. This is not a full historical-blob or remote-storage audit.",
        "https://docs.github.com/en/repositories/working-with-files/managing-large-files/about-large-files-on-github",
        Prototype,
        Some(Stage::Beta),
        false
    ),
    card!(
        "SEC-001",
        "Review secret-looking files tracked by Git",
        "Environment files and private-key filenames can indicate accidentally tracked credentials. Filename matching is a heuristic, not a secret-content scanner.",
        "Inspect flagged files privately. Keep real credentials outside Git and use placeholder example files. If a secret was exposed, rotate it and review its history. Never paste secret values into an Oracle report. Legitimate fixtures may be renamed to clearly identified .example files.",
        "No secret-candidate filenames are tracked in scope. Passing this check does not prove the repository is secret-free.",
        "https://docs.github.com/en/code-security/secret-scanning/introduction/about-secret-scanning",
        Prototype,
        Some(Stage::Prototype),
        false
    ),
    card!(
        3,
        "UX-002",
        "Review the real workflow and accessibility",
        "A density setting cannot prove a screen is usable. Review the actual task with realistic data and error states.",
        "Use the guide matching the screen's task. For a website or small app, review content, navigation, the primary action, forms or outputs, and persistence when promised. For operational screens, review comparison, selection scope, search/filter/actions, and access to details. Use realistic content at intended viewport sizes and input methods. Check keyboard order, focus, labels, zoom/reflow, target sizes, and relevant loading/empty/error states. Keep the review proportional to the features and consequences. Attach observations and remaining issues; illustrations and spacing values do not prove usability or accessibility.",
        "A current review artifact describes the scenario, viewport/input methods, observations, and remaining limitations.",
        "https://github.com/andersonbond/oracle/blob/main/skills/oracle/references/workflow.md#choose-the-ui-for-the-task",
        Beta,
        Some(Stage::Beta),
        true
    ),
    card!(
        "TEST-001",
        "Demonstrate a repeatable build and meaningful tests",
        "Configuration and test files are not proof of a successful build or that important behavior was exercised.",
        "Run the project's documented build and focused behavioral tests in an authorized environment. For Rust, start with cargo test --locked and cargo build --locked; use the project's own commands for other stacks. Record commands, versions, scope, results, and relevant failures in a small report. Broaden testing when changed behavior warrants it, not after every text edit.",
        "A current report records a successful build and meaningful tests for the stated application scope, with limitations acknowledged.",
        "https://doc.rust-lang.org/book/ch11-00-testing.html",
        Beta,
        Some(Stage::Beta),
        true
    ),
    card!(
        "SEC-002",
        "Test access to private user-owned records",
        "Sign-in establishes identity; authorization determines which records that identity can access.",
        "Use two test users. Attempt to read and modify User A's private records as User B through the server API. Include unauthenticated access and relevant administrative roles. Attach results showing denial without private content. Set data.private_user_records accurately; missing evidence alone does not establish a vulnerability.",
        "Current evidence covers successful legitimate access and denied cross-user/unauthenticated access to the relevant operations.",
        "https://owasp.org/www-project-application-security-verification-standard/",
        Beta,
        Some(Stage::Beta),
        true
    ),
    card!(
        "DATA-001",
        "Verify schema constraints and migrations",
        "A schema should enforce the application's data invariants. Normalization, constraints, and migrations need to fit the actual relationships and change path.",
        "Review ownership, keys, uniqueness, required values, and normalization against real writes and queries. Test migrations on a disposable representative database, including failure/recovery behavior. Inspect query plans before adding indexes; balance write cost and read benefit. Record what was tested, never production data.",
        "Current evidence describes schema invariants, migration results, and recovery behavior for a representative database.",
        "https://www.postgresql.org/docs/current/ddl-constraints.html",
        Beta,
        Some(Stage::Beta),
        true
    ),
    card!(
        "DATA-002",
        "Demonstrate a backup restore",
        "A scheduled backup is useful only if the data can be restored within the project's recovery needs.",
        "Restore a representative backup into an isolated authorized environment. Check record integrity and application access. Record recovery time, backup age, procedure, and remaining gaps. Do not include credentials or sensitive records in evidence artifacts.",
        "A current restore report demonstrates recoverable data and documents the recovery procedure and scope.",
        "https://www.postgresql.org/docs/current/backup.html",
        Production,
        Some(Stage::Production),
        true
    ),
    card!(
        "OPS-001",
        "Rehearse deployment and rollback",
        "A successful local build does not show that a release can be deployed and recovered safely.",
        "Test a repeatable deployment and rollback in a suitable environment. Identify configuration, migration compatibility, release identity, and recovery steps. Use containers when portability or deployment needs justify them. Keep the prototype workflow light; enable only the release gates the project needs.",
        "A current report identifies the release, deployment environment, verification, and tested rollback or recovery procedure.",
        "https://sre.google/workbook/canarying-releases/",
        Production,
        Some(Stage::Production),
        true
    ),
    card!(
        "OPS-002",
        "Make failures visible and assign an owner",
        "Operators need to notice failed requests, stalled jobs, and data problems after launch.",
        "Choose meaningful health signals, structured logs without sensitive values, actionable alerts, and an owner. Trigger a representative failure in an authorized environment and confirm the signal reaches the intended operator. Avoid adding monitoring infrastructure without an operational purpose.",
        "A current report demonstrates a useful failure signal, delivery route, owner, and response steps.",
        "https://sre.google/sre-book/monitoring-distributed-systems/",
        Production,
        Some(Stage::Production),
        true
    ),
    card!(
        "PERF-001",
        "Measure before pooling, caching, or scaling",
        "Caching, connection pools, and additional replicas each introduce limits and failure modes. They should address an observed workload and bottleneck.",
        "Define a representative workload and latency/error/resource budgets. Run bounded load tests only against an authorized environment. Inspect queries and pool saturation. Add caching with explicit freshness and invalidation rules. Compare vertical and horizontal scaling against measurements, state sharing, cost, and recovery needs.",
        "A current performance report records workload, environment, measurements, bottlenecks, and justified next steps. This advisory card does not require scaling every app.",
        "https://sre.google/sre-book/monitoring-distributed-systems/",
        Beta,
        None,
        true
    ),
];

pub fn rule(id: &str) -> Result<&'static Rule> {
    RULES
        .iter()
        .find(|r| r.id == id)
        .ok_or_else(|| format!("unknown rule {id}; run `oracle rules`").into())
}

#[derive(Debug, Serialize)]
pub struct Finding {
    pub id: &'static str,
    pub title: &'static str,
    pub status: Status,
    pub required: bool,
    pub detail: String,
    pub action: &'static str,
    pub done_when: &'static str,
    pub evidence: Option<evidence::Evidence>,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub oracle_version: &'static str,
    pub project: String,
    pub stage: Stage,
    pub snapshot: Snapshot,
    pub findings: Vec<Finding>,
}

fn applicability(rule: &Rule, c: &Config) -> bool {
    match rule.id {
        "UX-001" | "UX-002" => c.project.kind == Kind::WebApp,
        "SEC-002" => c.data.private_user_records,
        "DATA-001" | "DATA-002" => c.data.database,
        "OPS-001" | "OPS-002" | "PERF-001" => {
            matches!(c.project.kind, Kind::WebApp | Kind::Service)
        }
        _ => true,
    }
}

fn direct(rule: &Rule, c: &Config, s: &Snapshot) -> (Status, String) {
    match rule.id {
        "STACK-001" => {
            let recorded =
                !c.stack.language.trim().is_empty() && !c.stack.rationale.trim().is_empty();
            (
                if recorded {
                    Status::Pass
                } else {
                    Status::Unknown
                },
                if recorded {
                    format!(
                        "Stack decision recorded: {}. This validates the record, not the stack's fitness.",
                        c.stack.language
                    )
                } else if c.stack.prefer_rust {
                    "Record the language and rationale; consider Rust first for a new service or tool.".into()
                } else {
                    "Rust preference is disabled. Record the selected language and rationale."
                        .into()
                },
            )
        }
        "UX-001" => {
            let recorded = c.project.audience != Audience::Unspecified
                && c.ux.density != Density::Unspecified
                && !c.ux.primary_workflow.trim().is_empty()
                && !c.ux.rationale.trim().is_empty();
            (
                if recorded {
                    Status::Pass
                } else {
                    Status::Unknown
                },
                if recorded {
                    "Design intent recorded. UX-002 separately reviews actual usability.".into()
                } else if c.project.audience == Audience::Enterprise {
                    "For comparison or repeated operations, consider compact layouts; public pages may benefit from comfortable spacing. Record the actual task and density rationale.".into()
                } else {
                    "Record audience, density, primary workflow, and rationale before choosing layout spacing.".into()
                },
            )
        }
        "REPO-001" | "SEC-001" => {
            if !s.git {
                return (
                    Status::Unknown,
                    "No usable Git index; tracking state is unknown.".into(),
                );
            }
            let affected: Vec<_> = s
                .files
                .iter()
                .filter(|f| {
                    f.tracked
                        && if rule.id == "SEC-001" {
                            f.secret_candidate
                        } else {
                            f.generated
                        }
                })
                .collect();
            if !affected.is_empty() {
                (
                    Status::Fail,
                    format!(
                        "{} tracked candidate(s): {}. Review before changing anything.",
                        affected.len(),
                        affected
                            .iter()
                            .take(8)
                            .map(|f| f.path.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            } else if !s.complete {
                (
                    Status::Unknown,
                    "No candidates found among inspected files, but the snapshot is incomplete."
                        .into(),
                )
            } else {
                (Status::Pass, "No matching tracked paths in the inspected project scope; this is a bounded filename check.".into())
            }
        }
        "REPO-002" => {
            if !s.git {
                return (
                    Status::Unknown,
                    "Tracked size is unknown without a usable Git index.".into(),
                );
            }
            let limit = c.repository.max_file_mib * 1024 * 1024;
            let over: Vec<_> = s
                .files
                .iter()
                .filter(|f| f.tracked && f.bytes > limit)
                .collect();
            let failed =
                !over.is_empty() || s.tracked_bytes > c.repository.max_tracked_mib * 1024 * 1024;
            let mut largest: Vec<_> = s.files.iter().filter(|f| f.tracked).collect();
            largest.sort_by_key(|f| std::cmp::Reverse(f.bytes));
            (
                if failed {
                    Status::Fail
                } else if s.complete {
                    Status::Pass
                } else {
                    Status::Unknown
                },
                format!(
                    "Tracked working tree: {:.2} MiB / {} MiB budget; {} files exceed {} MiB. Largest: {}. Git history and remote storage are separate.",
                    s.tracked_bytes as f64 / 1048576.0,
                    c.repository.max_tracked_mib,
                    over.len(),
                    c.repository.max_file_mib,
                    largest
                        .iter()
                        .take(3)
                        .map(|f| format!("{} ({:.2} MiB)", f.path, f.bytes as f64 / 1048576.0))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            )
        }
        _ => (
            Status::Unknown,
            "No automated evaluator for this rule.".into(),
        ),
    }
}

pub fn assess(root: &Path, config: &Config) -> Result<Report> {
    let snapshot = project::scan(root)?;
    let mut findings = vec![];
    for rule in RULES {
        let explicit = config.policy.required.iter().any(|id| id == rule.id);
        let required = explicit
            || rule
                .required_at
                .is_some_and(|stage| config.project.stage >= stage);
        let mut proof = None;
        let (status, detail) = if !applicability(rule, config) {
            (Status::NotApplicable, "Not applicable to the declared project kind/data. Revisit if these assumptions change.".into())
        } else if config.project.stage < rule.starts_at && !explicit {
            (
                Status::Deferred,
                format!(
                    "Introduced at {:?}; the current {:?} stage stays lightweight. Explicit policy.required can activate this earlier.",
                    rule.starts_at, config.project.stage
                ),
            )
        } else if rule.accepts_evidence {
            let (state, detail, evidence) = evidence::evaluate(root, rule, config, &snapshot)?;
            proof = evidence;
            (state, detail)
        } else {
            direct(rule, config, &snapshot)
        };
        findings.push(Finding {
            id: rule.id,
            title: if rule.id == "STACK-001" && !config.stack.prefer_rust {
                "Choose the stack deliberately"
            } else {
                rule.title
            },
            status,
            required,
            detail,
            action: if rule.id == "STACK-001" && !config.stack.prefer_rust {
                "Choose the smallest implementation that serves the actual feature; a static website may not need a backend. Compare suitable languages using library maturity, delivery time, deployment needs, and team experience. Preserve a working existing stack unless a migration has a measured benefit. Record stack.language and stack.rationale. This project has disabled the Rust preference."
            } else {
                rule.action
            },
            done_when: rule.done_when,
            evidence: proof,
        });
    }
    Ok(Report {
        schema_version: 1,
        oracle_version: VERSION,
        project: config.project.name.clone(),
        stage: config.project.stage,
        snapshot,
        findings,
    })
}

pub fn blocking(report: &Report) -> usize {
    report
        .findings
        .iter()
        .filter(|f| {
            f.required && matches!(f.status, Status::Fail | Status::Unknown | Status::Stale)
        })
        .count()
}

pub fn next(report: &Report, limit: usize) -> Vec<&Finding> {
    let mut findings: Vec<_> = report
        .findings
        .iter()
        .filter(|f| matches!(f.status, Status::Fail | Status::Unknown | Status::Stale))
        .collect();
    findings.sort_by_key(|f| {
        (
            if f.status == Status::Fail {
                0
            } else if f.status == Status::Stale {
                1
            } else {
                2
            },
            !f.required,
        )
    });
    findings.truncate(limit);
    findings
}
