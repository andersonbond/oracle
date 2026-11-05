use crate::{
    Result, VERSION,
    config::Config,
    project::{self, Snapshot},
    rules::{Rule, Status},
};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Component, Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum Method {
    Manual,
    Report,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub schema_version: u32,
    pub oracle_version: String,
    pub rule_id: String,
    pub rule_version: u32,
    pub recorded_unix: u64,
    pub fingerprint: String,
    pub method: Method,
    pub outcome: Outcome,
    pub summary: String,
    pub artifacts: Vec<Artifact>,
}

fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

fn artifact_digest(root: &Path, path: &Path) -> Result<String> {
    let name = path
        .to_str()
        .ok_or("artifact path must be UTF-8")?
        .replace(std::path::MAIN_SEPARATOR, "/");
    let parts: Vec<_> = path
        .components()
        .filter_map(|part| match part {
            Component::Normal(name) => Some(name.to_string_lossy().to_ascii_lowercase()),
            _ => None,
        })
        .collect();
    // Apply the same restrictions on every read, including editable stored
    // records. Compare components to handle case-insensitive filesystems and
    // redundant separators without weakening safe_path's traversal checks.
    if parts.iter().any(|part| part == ".git")
        || parts
            .windows(2)
            .any(|pair| pair[0] == ".oracle" && pair[1] == "evidence")
        || project::secret_name(&name)
    {
        return Err(
            "do not attach Git internals, evidence records, or secret-candidate files".into(),
        );
    }
    project::file_digest(root, path)
}

fn latest(root: &Path, rule: &Rule) -> Result<Option<Evidence>> {
    let dir = project::safe_path(root, &PathBuf::from(".oracle/evidence").join(rule.id))?;
    if !dir.exists() {
        return Ok(None);
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.path().extension().is_some_and(|ext| ext == "json") {
            paths.push(entry.path());
        }
        if paths.len() > 2000 {
            return Err(
                "too many evidence revisions for one rule; archive older local evidence".into(),
            );
        }
    }
    paths.sort();
    let Some(path) = paths.last() else {
        return Ok(None);
    };
    let relative = path.strip_prefix(root)?;
    let bytes = project::read_limited(&project::safe_path(root, relative)?, 128 * 1024)?;
    let evidence: Evidence = serde_json::from_slice(&bytes).map_err(|_| "malformed local evidence JSON; inspect the latest record without trusting its contents")?;
    if evidence.schema_version != 1
        || evidence.rule_id != rule.id
        || evidence.artifacts.is_empty()
        || evidence.artifacts.len() > 8
        || evidence.summary.trim().len() < 10
        || evidence.summary.len() > 2000
    {
        return Err("invalid evidence schema, rule, summary, or artifact count".into());
    }
    Ok(Some(evidence))
}

pub fn record(
    root: &Path,
    rule: &Rule,
    outcome: Outcome,
    method: Method,
    summary: &str,
    artifacts: &[PathBuf],
) -> Result<Evidence> {
    if !rule.accepts_evidence {
        return Err("this rule is evaluated directly; evidence cannot override its result".into());
    }
    if summary.trim().len() < 10 || summary.len() > 2000 {
        return Err(
            "summary must contain 10–2000 bytes describing the scenario and limitations".into(),
        );
    }
    if artifacts.is_empty() || artifacts.len() > 8 {
        return Err("supply 1–8 project-relative --artifact paths".into());
    }
    let config = crate::config::load(root)?;
    config.validate()?;
    let snapshot = project::scan(root)?;
    if !snapshot.complete {
        return Err("cannot record current evidence for an incomplete snapshot; run `oracle assess` for coverage limits".into());
    }
    let mut refs = vec![];
    for path in artifacts {
        let name = path
            .to_str()
            .ok_or("artifact path must be UTF-8")?
            .replace(std::path::MAIN_SEPARATOR, "/");
        let hash = artifact_digest(root, path)?;
        refs.push(Artifact {
            path: name,
            sha256: hash,
        });
    }
    // Recheck after reading artifacts so an edit during collection cannot silently
    // bind proof to a different code snapshot.
    let after = project::scan(root)?;
    if !after.complete || after.fingerprint != snapshot.fingerprint {
        return Err("project changed while recording evidence; retry after changes settle".into());
    }
    let evidence = Evidence {
        schema_version: 1,
        oracle_version: VERSION.into(),
        rule_id: rule.id.into(),
        rule_version: rule.version,
        recorded_unix: now()?,
        fingerprint: snapshot.fingerprint,
        method,
        outcome,
        summary: summary.trim().into(),
        artifacts: refs,
    };
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let path = project::safe_path(
        root,
        &PathBuf::from(".oracle/evidence")
            .join(rule.id)
            .join(format!("{stamp:030}.json")),
    )?;
    project::write_new(&path, &serde_json::to_vec_pretty(&evidence)?)?;
    Ok(evidence)
}

pub fn evaluate(
    root: &Path,
    rule: &Rule,
    config: &Config,
    snapshot: &Snapshot,
) -> Result<(Status, String, Option<Evidence>)> {
    let Some(evidence) = latest(root, rule)? else {
        return Ok((
            Status::Unknown,
            "No evidence recorded. Missing proof is not a confirmed defect.".into(),
            None,
        ));
    };
    let time = now()?;
    let stale_reason =
        if !snapshot.complete {
            Some("current snapshot is incomplete")
        } else if evidence.rule_version != rule.version || evidence.oracle_version != VERSION {
            Some("rule or Oracle version changed")
        } else if evidence.recorded_unix > time {
            Some("record is dated in the future")
        } else if time - evidence.recorded_unix > config.policy.evidence_max_age_days * 86400 {
            Some("evidence exceeded its freshness window")
        } else if evidence.fingerprint != snapshot.fingerprint {
            Some("inspected project contents changed")
        } else if evidence.artifacts.iter().any(|a| {
            artifact_digest(root, Path::new(&a.path)).map_or(true, |hash| hash != a.sha256)
        }) {
            Some("an artifact changed, is missing, or is unsafe to read")
        } else {
            None
        };
    if let Some(reason) = stale_reason {
        return Ok((
            Status::Stale,
            format!(
                "Evidence is stale: {reason}. Re-run/review the relevant scenario and record new evidence."
            ),
            Some(evidence),
        ));
    }
    let status = match evidence.outcome {
        Outcome::Pass => Status::Pass,
        Outcome::Fail => Status::Fail,
    };
    let detail = format!(
        "Current {:?} attestation: {}. Oracle checks provenance/freshness, not the truth of the supplied claim.",
        evidence.method, evidence.summary
    );
    Ok((status, detail, Some(evidence)))
}
