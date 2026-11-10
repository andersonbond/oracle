use oracle_guide::config::{Config, Stage};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
use tempfile::TempDir;

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oracle"))
        .arg("--root")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}
fn ok(root: &Path, args: &[&str]) -> Output {
    let out = run(root, args);
    assert!(
        out.status.success(),
        "{:?}\nstdout: {}\nstderr: {}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    out
}
fn git(root: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}
fn fixture() -> TempDir {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    ok(
        root.path(),
        &["init", "--name", "example", "--audience", "enterprise"],
    );
    write(root.path(), "src/main.rs", "fn main() {}\n");
    write(root.path(), ".gitignore", "target/\n.oracle/\n");
    git(root.path(), &["add", "."]);
    root
}
fn configure(root: &Path, f: impl FnOnce(&mut Config)) {
    let mut config = oracle_guide::config::load(root).unwrap();
    f(&mut config);
    fs::write(
        root.join("oracle.toml"),
        toml::to_string_pretty(&config).unwrap(),
    )
    .unwrap();
}
fn assess(root: &Path) -> Value {
    serde_json::from_slice(&ok(root, &["assess", "--format", "json"]).stdout).unwrap()
}
fn finding<'a>(report: &'a Value, id: &str) -> &'a Value {
    report["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == id)
        .unwrap()
}
fn proof(root: &Path, outcome: &str) {
    ok(
        root,
        &[
            "evidence",
            "record",
            "TEST-001",
            "--result",
            outcome,
            "--method",
            "report",
            "--summary",
            "Build and core tests executed locally; external services mocked",
            "--artifact",
            ".oracle/reports/test.txt",
        ],
    );
}

#[test]
fn help_version_and_rules_are_available_without_a_project() {
    let root = tempfile::tempdir().unwrap();
    assert!(
        String::from_utf8(ok(root.path(), &["--version"]).stdout)
            .unwrap()
            .contains(oracle_guide::VERSION)
    );
    ok(root.path(), &["--help"]);
    let rules: Value =
        serde_json::from_slice(&ok(root.path(), &["rules", "--format", "json"]).stdout).unwrap();
    assert_eq!(rules["rules"].as_array().unwrap().len(), 13);
    ok(root.path(), &["explain", "STACK-001"]);
}

#[test]
fn init_preview_is_read_only_and_reinit_preserves_choices() {
    let root = tempfile::tempdir().unwrap();
    ok(root.path(), &["init", "--dry-run"]);
    assert!(!root.path().join("oracle.toml").exists());
    assert!(!root.path().join(".oracle").exists());
    ok(root.path(), &["init", "--name", "original"]);
    let original = fs::read(root.path().join("oracle.toml")).unwrap();
    ok(
        root.path(),
        &["init", "--name", "replacement", "--stage", "production"],
    );
    assert_eq!(original, fs::read(root.path().join("oracle.toml")).unwrap());
}

#[test]
fn malformed_or_unknown_configuration_fails_with_machine_readable_error() {
    let root = fixture();
    let text = fs::read_to_string(root.path().join("oracle.toml")).unwrap();
    write(
        root.path(),
        "oracle.toml",
        &format!("{text}\n[stak]\nlanguage = 'rust'\n"),
    );
    let out = run(root.path(), &["assess", "--format", "json"]);
    assert_eq!(out.status.code(), Some(2));
    let error: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert!(
        error["error"]
            .as_str()
            .unwrap()
            .contains("invalid oracle.toml")
    );
}

#[test]
fn invalid_schema_and_unknown_required_rules_are_rejected() {
    let root = fixture();
    configure(root.path(), |c| c.schema_version = 99);
    assert_eq!(run(root.path(), &["assess"]).status.code(), Some(2));
    write(
        root.path(),
        "oracle.toml",
        &toml::to_string_pretty(&Config::new(
            "x".into(),
            Default::default(),
            Default::default(),
            Default::default(),
        ))
        .unwrap(),
    );
    configure(root.path(), |c| c.policy.required = vec!["MADE-UP".into()]);
    assert_eq!(run(root.path(), &["check"]).status.code(), Some(2));
}

#[test]
fn prototypes_stay_light_and_strict_beta_has_real_exit_codes() {
    let root = fixture();
    let report = assess(root.path());
    assert_eq!(finding(&report, "TEST-001")["status"], "deferred");
    assert_eq!(finding(&report, "STACK-001")["status"], "unknown");
    ok(root.path(), &["check", "--strict"]);
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    ok(root.path(), &["check"]);
    assert_eq!(
        run(root.path(), &["check", "--strict"]).status.code(),
        Some(1)
    );
    assert_eq!(
        finding(&assess(root.path()), "TEST-001")["status"],
        "unknown"
    );
}

#[test]
fn explicit_required_rule_can_activate_early() {
    let root = fixture();
    configure(root.path(), |c| c.policy.required = vec!["TEST-001".into()]);
    assert_eq!(
        run(root.path(), &["check", "--strict"]).status.code(),
        Some(1)
    );
    assert_eq!(
        finding(&assess(root.path()), "TEST-001")["status"],
        "unknown"
    );
}

#[test]
fn documented_non_rust_and_comfortable_ui_choices_are_respected() {
    let root = fixture();
    configure(root.path(), |c| {
        c.stack.language = "Python".into();
        c.stack.rationale = "Existing analytics ecosystem and team experience".into();
        c.ux.density = oracle_guide::config::Density::Comfortable;
        c.ux.primary_workflow = "Warehouse staff picking items on tablets".into();
        c.ux.rationale = "Touch operation calls for larger targets".into();
    });
    let report = assess(root.path());
    assert_eq!(finding(&report, "STACK-001")["status"], "pass");
    assert_eq!(finding(&report, "UX-001")["status"], "pass");
    assert_eq!(finding(&report, "UX-002")["status"], "deferred");
}

#[test]
fn simple_site_example_stays_light_without_bypassing_release_or_data_checks() {
    let root = fixture();
    write(
        root.path(),
        "oracle.toml",
        include_str!("../examples/simple-site/oracle.toml"),
    );
    let report = assess(root.path());
    assert_eq!(finding(&report, "STACK-001")["status"], "pass");
    assert_eq!(finding(&report, "UX-001")["status"], "pass");
    assert_eq!(finding(&report, "UX-002")["status"], "deferred");
    for id in ["DATA-001", "DATA-002", "SEC-002"] {
        assert_eq!(finding(&report, id)["status"], "not_applicable");
    }
    ok(root.path(), &["check", "--strict"]);
    configure(root.path(), |c| c.project.stage = Stage::Production);
    let report = assess(root.path());
    for id in ["UX-002", "TEST-001", "OPS-001", "OPS-002"] {
        assert_eq!(finding(&report, id)["status"], "unknown");
        assert_eq!(finding(&report, id)["required"], true);
    }
    assert_eq!(
        run(root.path(), &["check", "--strict"]).status.code(),
        Some(1)
    );
    configure(root.path(), |c| {
        c.data.database = true;
        c.data.private_user_records = true;
    });
    let report = assess(root.path());
    for id in ["DATA-001", "DATA-002", "SEC-002"] {
        assert_eq!(finding(&report, id)["status"], "unknown");
        assert_eq!(finding(&report, id)["required"], true);
    }
}

#[test]
fn disabling_rust_preference_changes_the_next_action_without_passing_the_decision() {
    let root = fixture();
    let preferred = assess(root.path());
    configure(root.path(), |c| c.stack.prefer_rust = false);
    let opted_out = assess(root.path());
    assert_ne!(
        finding(&preferred, "STACK-001")["action"],
        finding(&opted_out, "STACK-001")["action"]
    );
    assert_eq!(finding(&opted_out, "STACK-001")["status"], "unknown");
}

#[test]
fn realtime_example_requires_actual_behavior_and_authorization_evidence_at_beta() {
    let root = fixture();
    write(
        root.path(),
        "oracle.toml",
        include_str!("../examples/realtime/oracle.toml"),
    );
    let report = assess(root.path());
    assert_eq!(finding(&report, "STACK-001")["status"], "pass");
    assert_eq!(finding(&report, "UX-001")["status"], "pass");
    for id in ["TEST-001", "SEC-002", "DATA-001"] {
        assert_eq!(finding(&report, id)["status"], "deferred");
    }
    ok(root.path(), &["check", "--strict"]);
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    let report = assess(root.path());
    for id in ["TEST-001", "SEC-002", "DATA-001"] {
        assert_eq!(finding(&report, id)["status"], "unknown");
        assert_eq!(finding(&report, id)["required"], true);
    }
    assert_eq!(
        run(root.path(), &["check", "--strict"]).status.code(),
        Some(1)
    );
}

#[test]
fn small_database_example_keeps_performance_advisory_and_restore_required_at_production() {
    let root = fixture();
    write(
        root.path(),
        "oracle.toml",
        include_str!("../examples/simple-data/oracle.toml"),
    );
    let report = assess(root.path());
    assert_eq!(finding(&report, "STACK-001")["status"], "pass");
    assert_eq!(finding(&report, "UX-001")["status"], "pass");
    for id in ["DATA-001", "DATA-002", "PERF-001", "SEC-002"] {
        assert_eq!(finding(&report, id)["status"], "deferred");
    }
    ok(root.path(), &["check", "--strict"]);
    for stage in [Stage::Beta, Stage::Production] {
        configure(root.path(), |c| c.project.stage = stage);
        let report = assess(root.path());
        for id in ["DATA-001", "SEC-002"] {
            assert_eq!(finding(&report, id)["status"], "unknown");
            assert_eq!(finding(&report, id)["required"], true);
        }
        assert_eq!(finding(&report, "PERF-001")["status"], "unknown");
        assert_eq!(finding(&report, "PERF-001")["required"], false);
        let production = stage == Stage::Production;
        assert_eq!(finding(&report, "DATA-002")["required"], production);
        assert_eq!(
            finding(&report, "DATA-002")["status"],
            if production { "unknown" } else { "deferred" }
        );
        assert_eq!(
            run(root.path(), &["check", "--strict"]).status.code(),
            Some(1)
        );
    }
}

#[test]
fn data_and_interface_applicability_are_explicit() {
    let root = fixture();
    configure(root.path(), |c| {
        c.project.kind = oracle_guide::config::Kind::Service;
        c.project.stage = Stage::Production;
        c.data.database = true;
        c.data.private_user_records = true;
    });
    let report = assess(root.path());
    assert_eq!(finding(&report, "UX-001")["status"], "not_applicable");
    assert_eq!(finding(&report, "DATA-002")["status"], "unknown");
    assert_eq!(finding(&report, "SEC-002")["status"], "unknown");
}

#[test]
fn ignored_build_cache_is_distinguished_from_tracked_generated_content() {
    let root = fixture();
    write(root.path(), "target/cache.bin", "regenerable local cache");
    assert_eq!(finding(&assess(root.path()), "REPO-001")["status"], "pass");
    git(root.path(), &["add", "-f", "target/cache.bin"]);
    let report = assess(root.path());
    assert_eq!(finding(&report, "REPO-001")["status"], "fail");
    assert!(root.path().join("target/cache.bin").exists());
}

#[test]
fn tracked_size_budget_is_checked_without_deleting_assets() {
    let root = fixture();
    configure(root.path(), |c| {
        c.repository.max_file_mib = 1;
    });
    fs::File::create(root.path().join("large.bin"))
        .unwrap()
        .set_len(1_048_577)
        .unwrap();
    git(root.path(), &["add", "large.bin"]);
    assert_eq!(finding(&assess(root.path()), "REPO-002")["status"], "fail");
    assert!(root.path().join("large.bin").exists());
}

#[test]
fn tracked_secret_candidates_are_reported_without_reading_the_value() {
    let root = fixture();
    write(
        root.path(),
        ".env",
        "SYNTHETIC_SENTINEL_DO_NOT_PRINT=not-a-real-secret",
    );
    git(root.path(), &["add", ".env"]);
    let out = ok(root.path(), &["assess", "--format", "json"]);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(!text.contains("SYNTHETIC_SENTINEL_DO_NOT_PRINT"));
    assert_eq!(
        finding(&serde_json::from_str(&text).unwrap(), "SEC-001")["status"],
        "fail"
    );
    assert_eq!(
        run(root.path(), &["check", "--strict"]).status.code(),
        Some(1)
    );
}

#[test]
fn evidence_tracks_dirty_code_and_changed_artifacts() {
    let root = fixture();
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "Build and behavior checks passed; test fixture only.",
    );
    proof(root.path(), "pass");
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "pass");
    write(
        root.path(),
        "src/main.rs",
        "fn main() { println!(\"changed\"); }\n",
    );
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "stale");
    proof(root.path(), "pass");
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "pass");
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "The report changed after recording.",
    );
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "stale");
}

#[test]
fn failed_evidence_stays_failed_and_records_are_retained() {
    let root = fixture();
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "A real behavioral test failed in this synthetic fixture.",
    );
    proof(root.path(), "pass");
    proof(root.path(), "fail");
    let report = assess(root.path());
    assert_eq!(finding(&report, "TEST-001")["status"], "fail");
    assert_eq!(finding(&report, "TEST-001")["evidence"]["method"], "report");
    assert_eq!(
        fs::read_dir(root.path().join(".oracle/evidence/TEST-001"))
            .unwrap()
            .count(),
        2
    );
}

#[test]
fn expired_evidence_is_not_current() {
    let root = fixture();
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "Fixture report for testing expiry.",
    );
    proof(root.path(), "pass");
    let path = fs::read_dir(root.path().join(".oracle/evidence/TEST-001"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let mut record: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    record["recorded_unix"] = 1.into();
    fs::write(path, serde_json::to_vec(&record).unwrap()).unwrap();
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "stale");
}

#[test]
fn evidence_cannot_override_direct_checks_or_escape_the_project() {
    let root = fixture();
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "synthetic fixture evidence",
    );
    let out = run(
        root.path(),
        &[
            "evidence",
            "record",
            "REPO-001",
            "--result",
            "pass",
            "--summary",
            "Attempted override of an automated check",
            "--artifact",
            ".oracle/reports/test.txt",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    let out = run(
        root.path(),
        &[
            "evidence",
            "record",
            "TEST-001",
            "--result",
            "pass",
            "--summary",
            "Attempted outside-project artifact",
            "--artifact",
            "../outside.txt",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
}

#[test]
fn skill_install_is_idempotent_and_remove_preserves_unrelated_files() {
    let root = tempfile::tempdir().unwrap();
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("skills/oracle");
    for (client, dir) in [
        ("codex", ".agents"),
        ("claude", ".claude"),
        ("generic", ".agents"),
    ] {
        write(root.path(), &format!("{dir}/keep.txt"), "user data");
        ok(root.path(), &["agent", "install", client, "--dry-run"]);
        assert!(
            !root
                .path()
                .join(format!("{dir}/skills/oracle/SKILL.md"))
                .exists()
        );
        ok(root.path(), &["agent", "install", client]);
        ok(root.path(), &["agent", "install", client]);
        let installed = root.path().join(format!("{dir}/skills/oracle"));
        // A binary-only install must include the complete portable package,
        // including references and images, without needing the source checkout.
        for entry in walkdir::WalkDir::new(&source) {
            let entry = entry.unwrap();
            if entry.file_type().is_file() {
                let relative = entry.path().strip_prefix(&source).unwrap();
                assert_eq!(
                    fs::read(installed.join(relative)).unwrap(),
                    fs::read(entry.path()).unwrap(),
                    "missing or changed bundled resource: {}",
                    relative.display()
                );
            }
        }
        ok(root.path(), &["agent", "remove", client, "--dry-run"]);
        assert!(installed.join("references/compact-ui.md").exists());
        ok(root.path(), &["agent", "remove", client]);
        assert!(!root.path().join(format!("{dir}/skills/oracle")).exists());
        assert_eq!(
            fs::read_to_string(root.path().join(format!("{dir}/keep.txt"))).unwrap(),
            "user data"
        );
    }
}

#[test]
fn skill_guides_are_reachable_without_configuration_or_active_findings() {
    for (client, dir) in [
        ("codex", ".agents"),
        ("claude", ".claude"),
        ("generic", ".agents"),
    ] {
        let root = tempfile::tempdir().unwrap();
        ok(root.path(), &["agent", "install", client]);
        ok(root.path(), &["explain", "UX-001"]);
        assert!(!root.path().join("oracle.toml").exists());
        assert!(!root.path().join(".oracle").exists());

        let installed = root
            .path()
            .join(format!("{dir}/skills/oracle"))
            .canonicalize()
            .unwrap();
        let mut pending = vec![installed.join("SKILL.md")];
        let mut reachable = std::collections::HashSet::new();
        // Follow the actual inline Markdown links from the installed entry point.
        // Files merely existing in the bundle is not enough for an agent to find them.
        while let Some(file) = pending.pop() {
            if !reachable.insert(file.clone()) || file.extension().is_none_or(|e| e != "md") {
                continue;
            }
            let content = fs::read_to_string(&file).unwrap();
            let mut in_code = false;
            for line in content.lines() {
                if line.trim_start().starts_with("```") {
                    in_code = !in_code;
                    continue;
                }
                if in_code {
                    continue;
                }
                for link in line.split("](").skip(1) {
                    let target = link.split(')').next().unwrap().split('#').next().unwrap();
                    if target.is_empty() || target.contains(':') {
                        continue;
                    }
                    let path = file.parent().unwrap().join(target);
                    let path = path.canonicalize().unwrap_or_else(|e| {
                        panic!("broken installed link {} -> {target}: {e}", file.display())
                    });
                    assert!(
                        path.starts_with(&installed),
                        "link escapes the skill bundle"
                    );
                    pending.push(path);
                }
            }
        }
        for resource in [
            "references/ui-design.md",
            "references/image-uploads.md",
            "references/security.md",
            "references/containerization.md",
            "references/caching.md",
            "assets/caching/Cargo.toml.example",
            "assets/caching/Cargo.lock.example",
            "assets/caching/main.rs",
            "assets/caching/Dockerfile",
            "assets/caching/Dockerfile.dockerignore",
            "assets/caching/compose.yaml",
            "assets/caching/verify.py",
            "assets/containerization/Dockerfile",
            "assets/containerization/Dockerfile.dockerignore",
            "assets/containerization/compose.yaml",
            "references/simple-projects.md",
            "references/compact-ui.md",
            "references/emil-design-engineering.md",
            "references/emil-ui-cheatsheet.md",
            "references/emil-LICENSE.txt",
            "assets/simple-projects.svg",
            "assets/compact-workspace.svg",
            "assets/adaptive-workspace.svg",
        ] {
            assert!(
                reachable.contains(&installed.join(resource)),
                "unreachable: {resource}"
            );
        }

        for example in [
            include_str!("../examples/simple-site/oracle.toml"),
            include_str!("../examples/oracle.toml"),
        ] {
            write(root.path(), "oracle.toml", example);
            let report = assess(root.path());
            assert_eq!(finding(&report, "UX-001")["status"], "pass");
            assert_eq!(finding(&report, "UX-002")["status"], "deferred");
            let next: Value = serde_json::from_slice(
                &ok(root.path(), &["next", "--limit", "13", "--format", "json"]).stdout,
            )
            .unwrap();
            assert!(
                next["actions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|f| { f["id"] != "UX-001" && f["id"] != "UX-002" })
            );
            assert!(installed.join("references/ui-design.md").is_file());
        }
    }
}

#[test]
fn skill_conflicts_are_preflighted_without_partial_installation() {
    for resource in [
        "references/workflow.md",
        "references/compact-ui.md",
        "references/simple-projects.md",
        "references/realtime.md",
        "references/database.md",
        "references/image-uploads.md",
        "references/security.md",
        "references/containerization.md",
        "references/caching.md",
        "assets/caching/Cargo.toml.example",
        "assets/caching/Cargo.lock.example",
        "assets/caching/main.rs",
        "assets/caching/Dockerfile",
        "assets/caching/Dockerfile.dockerignore",
        "assets/caching/compose.yaml",
        "assets/caching/verify.py",
        "assets/containerization/Dockerfile",
        "assets/containerization/Dockerfile.dockerignore",
        "assets/containerization/compose.yaml",
        "references/ui-design.md",
        "references/emil-design-engineering.md",
        "references/emil-ui-cheatsheet.md",
        "references/emil-LICENSE.txt",
        "assets/simple-projects.svg",
        "assets/adaptive-workspace.svg",
    ] {
        let root = tempfile::tempdir().unwrap();
        let destination = format!(".agents/skills/oracle/{resource}");
        write(root.path(), &destination, "user-authored resource");
        assert_eq!(
            run(root.path(), &["agent", "install", "codex"])
                .status
                .code(),
            Some(2)
        );
        assert!(!root.path().join(".agents/skills/oracle/SKILL.md").exists());
        assert_eq!(
            fs::read_to_string(root.path().join(destination)).unwrap(),
            "user-authored resource"
        );
    }
}

#[test]
fn removing_skill_preserves_modified_and_unrelated_resources() {
    let root = tempfile::tempdir().unwrap();
    ok(root.path(), &["agent", "install", "claude"]);
    let installed = root.path().join(".claude/skills/oracle");
    for resource in [
        "assets/compact-workspace.svg",
        "assets/containerization/Dockerfile",
    ] {
        let asset = installed.join(resource);
        let original = fs::read(&asset).unwrap();
        fs::write(&asset, "custom resource").unwrap();
        assert_eq!(
            run(root.path(), &["agent", "remove", "claude"])
                .status
                .code(),
            Some(2)
        );
        assert!(installed.join("SKILL.md").exists());
        assert!(installed.join("references/compact-ui.md").exists());
        assert_eq!(fs::read_to_string(&asset).unwrap(), "custom resource");
        fs::write(&asset, original).unwrap();
    }
    for resource in ["assets/custom.svg", "assets/containerization/custom.txt"] {
        fs::write(installed.join(resource), "user asset").unwrap();
    }
    ok(root.path(), &["agent", "remove", "claude"]);
    assert!(!installed.join("SKILL.md").exists());
    assert!(!installed.join("assets/compact-workspace.svg").exists());
    assert!(
        !installed
            .join("assets/containerization/Dockerfile")
            .exists()
    );
    for resource in ["assets/custom.svg", "assets/containerization/custom.txt"] {
        assert_eq!(
            fs::read_to_string(installed.join(resource)).unwrap(),
            "user asset"
        );
    }
}

#[test]
fn previous_guidance_rule_evidence_requires_a_new_review() {
    for (id, version) in [("UX-002", 5), ("DATA-001", 2), ("PERF-001", 2)] {
        let root = fixture();
        configure(root.path(), |c| {
            c.project.stage = Stage::Beta;
            c.data.database = true;
        });
        write(
            root.path(),
            ".oracle/reports/review.txt",
            "Synthetic review fixture.",
        );
        ok(
            root.path(),
            &[
                "evidence",
                "record",
                id,
                "--result",
                "pass",
                "--summary",
                "Synthetic review for testing evidence version changes",
                "--artifact",
                ".oracle/reports/review.txt",
            ],
        );
        assert_eq!(finding(&assess(root.path()), id)["status"], "pass");
        let path = fs::read_dir(root.path().join(format!(".oracle/evidence/{id}")))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path();
        let mut record: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(record["rule_version"], version);
        record["rule_version"] = (version - 1).into();
        fs::write(path, serde_json::to_vec(&record).unwrap()).unwrap();
        assert_eq!(finding(&assess(root.path()), id)["status"], "stale");
    }
}

#[test]
fn modified_skill_cannot_be_removed_silently() {
    let root = tempfile::tempdir().unwrap();
    ok(root.path(), &["agent", "install", "claude"]);
    write(
        root.path(),
        ".claude/skills/oracle/SKILL.md",
        "user-edited skill",
    );
    assert_eq!(
        run(root.path(), &["agent", "remove", "claude"])
            .status
            .code(),
        Some(2)
    );
    assert!(
        root.path()
            .join(".claude/skills/oracle/references/workflow.md")
            .exists()
    );
}

#[test]
fn non_git_projects_get_honest_tracking_limits_and_no_script_execution() {
    let root = tempfile::tempdir().unwrap();
    ok(root.path(), &["init"]);
    write(
        root.path(),
        "package.json",
        r#"{"scripts":{"postinstall":"touch forbidden-side-effect"}}"#,
    );
    let report = assess(root.path());
    assert_eq!(report["snapshot"]["git"], false);
    assert_eq!(finding(&report, "REPO-002")["status"], "unknown");
    assert!(!root.path().join("forbidden-side-effect").exists());
}

#[cfg(unix)]
#[test]
fn symlinks_cannot_redirect_installs_or_evidence_writes() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), root.path().join(".agents")).unwrap();
    assert_eq!(
        run(root.path(), &["agent", "install", "codex"])
            .status
            .code(),
        Some(2)
    );
    assert!(!outside.path().join("skills").exists());
    symlink(outside.path(), root.path().join(".oracle")).unwrap();
    assert_eq!(run(root.path(), &["init"]).status.code(), Some(2));
    assert!(!root.path().join("oracle.toml").exists());
}

#[test]
fn ignored_project_configuration_still_invalidates_evidence() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    write(root.path(), ".gitignore", "oracle.toml\n.oracle/\n");
    ok(root.path(), &["init", "--stage", "beta"]);
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "Synthetic build and test report.",
    );
    proof(root.path(), "pass");
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "pass");
    configure(root.path(), |c| {
        c.stack.rationale = "Changed context ignored by Git".into()
    });
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "stale");
}

#[test]
fn an_incomplete_snapshot_cannot_record_a_fresh_pass() {
    let root = fixture();
    fs::File::create(root.path().join("large-data.bin"))
        .unwrap()
        .set_len(16 * 1024 * 1024 + 1)
        .unwrap();
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "Synthetic build and test report.",
    );
    assert_eq!(assess(root.path())["snapshot"]["complete"], false);
    let out = run(
        root.path(),
        &[
            "evidence",
            "record",
            "TEST-001",
            "--result",
            "pass",
            "--summary",
            "A snapshot with unread contents cannot be current",
            "--artifact",
            ".oracle/reports/test.txt",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(!root.path().join(".oracle/evidence").exists());
}

#[test]
fn malformed_newest_evidence_is_an_error_not_an_old_pass() {
    let root = fixture();
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "Synthetic build and test report.",
    );
    proof(root.path(), "pass");
    write(
        root.path(),
        ".oracle/evidence/TEST-001/zz-invalid.json",
        "{ invalid",
    );
    assert_eq!(
        run(root.path(), &["check", "--format", "json"])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn subdirectory_inspection_does_not_include_sibling_projects() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    write(root.path(), "sibling/private.txt", "a sibling project");
    fs::create_dir(root.path().join("app")).unwrap();
    ok(&root.path().join("app"), &["init"]);
    write(root.path(), "app/main.rs", "fn main() {}\n");
    git(root.path(), &["add", "."]);
    let report = assess(&root.path().join("app"));
    let files = report["snapshot"]["files"].as_array().unwrap();
    assert!(
        files
            .iter()
            .all(|f| !f["path"].as_str().unwrap().contains("sibling"))
    );
    assert!(files.iter().any(|f| f["path"] == "main.rs"));
}

#[test]
fn cli_projects_do_not_inherit_hosted_service_operations_gates() {
    let root = fixture();
    configure(root.path(), |c| {
        c.project.kind = oracle_guide::config::Kind::Cli;
        c.project.stage = Stage::Production;
    });
    let report = assess(root.path());
    for id in ["OPS-001", "OPS-002", "PERF-001", "UX-002"] {
        assert_eq!(finding(&report, id)["status"], "not_applicable");
    }
    assert_eq!(finding(&report, "TEST-001")["status"], "unknown");
}

#[cfg(unix)]
#[test]
fn attached_artifacts_cannot_follow_external_symlinks() {
    use std::os::unix::fs::symlink;
    let root = fixture();
    let outside = tempfile::tempdir().unwrap();
    write(outside.path(), "report.txt", "An external private report");
    fs::create_dir_all(root.path().join(".oracle/reports")).unwrap();
    symlink(
        outside.path().join("report.txt"),
        root.path().join(".oracle/reports/link.txt"),
    )
    .unwrap();
    let out = run(
        root.path(),
        &[
            "evidence",
            "record",
            "TEST-001",
            "--result",
            "pass",
            "--summary",
            "Reject a report path escaping the project",
            "--artifact",
            ".oracle/reports/link.txt",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(!root.path().join(".oracle/evidence").exists());
}

#[test]
fn review_tracked_local_state_is_not_hidden_from_repository_checks() {
    let root = fixture();
    configure(root.path(), |c| c.repository.max_file_mib = 1);
    write(
        root.path(),
        ".oracle/.env",
        "SYNTHETIC_PRIVATE_VALUE=not-a-real-secret",
    );
    fs::create_dir_all(root.path().join(".oracle/reports")).unwrap();
    fs::File::create(root.path().join(".oracle/reports/large.bin"))
        .unwrap()
        .set_len(1_048_577)
        .unwrap();
    git(
        root.path(),
        &["add", "-f", ".oracle/.env", ".oracle/reports/large.bin"],
    );
    let report = assess(root.path());
    for id in ["REPO-001", "REPO-002", "SEC-001"] {
        assert_eq!(
            finding(&report, id)["status"],
            "fail",
            "{id} must include tracked Oracle state"
        );
    }
    assert!(!report.to_string().contains("SYNTHETIC_PRIVATE_VALUE"));
}

#[cfg(unix)]
#[test]
fn review_executable_permission_changes_invalidate_evidence() {
    use std::os::unix::fs::PermissionsExt;
    let root = fixture();
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    write(root.path(), "launch.sh", "#!/bin/sh\nexit 0\n");
    fs::set_permissions(
        root.path().join("launch.sh"),
        fs::Permissions::from_mode(0o755),
    )
    .unwrap();
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "Launcher executed successfully in this synthetic test.",
    );
    proof(root.path(), "pass");
    fs::set_permissions(
        root.path().join("launch.sh"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    assert_eq!(finding(&assess(root.path()), "TEST-001")["status"], "stale");
}

#[test]
fn review_stored_evidence_cannot_read_forbidden_artifacts() {
    let root = fixture();
    configure(root.path(), |c| c.project.stage = Stage::Beta);
    write(root.path(), ".gitignore", "target/\n.oracle/\n.env\n");
    write(
        root.path(),
        ".env",
        "SYNTHETIC_PRIVATE_VALUE=not-a-real-secret",
    );
    write(
        root.path(),
        ".oracle/reports/test.txt",
        "Synthetic build and test report.",
    );
    proof(root.path(), "pass");
    let record_path = fs::read_dir(root.path().join(".oracle/evidence/TEST-001"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let original: Value = serde_json::from_slice(&fs::read(&record_path).unwrap()).unwrap();
    for name in [".env", ".git/config"] {
        let mut record = original.clone();
        record["artifacts"] = serde_json::json!([{"path":name,"sha256":oracle_guide::project::digest(&fs::read(root.path().join(name)).unwrap())}]);
        fs::write(&record_path, serde_json::to_vec(&record).unwrap()).unwrap();
        assert_eq!(
            finding(&assess(root.path()), "TEST-001")["status"],
            "stale",
            "must not accept {name}"
        );
    }
}

#[test]
fn review_recording_rejects_nested_git_internals() {
    let root = fixture();
    write(
        root.path(),
        ".oracle/reports/nested/.git/config",
        "synthetic internal config",
    );
    let out = run(
        root.path(),
        &[
            "evidence",
            "record",
            "TEST-001",
            "--result",
            "pass",
            "--summary",
            "Git internals are not appropriate evidence artifacts",
            "--artifact",
            ".oracle/reports/nested/.git/config",
        ],
    );
    assert_eq!(out.status.code(), Some(2));
}
