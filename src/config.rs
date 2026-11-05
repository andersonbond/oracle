use crate::{Result, project};
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ValueEnum, Default,
)]
#[serde(rename_all = "kebab-case")]
pub enum Stage {
    #[default]
    Prototype,
    Beta,
    Production,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    #[default]
    WebApp,
    Service,
    Cli,
    Library,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Audience {
    #[default]
    Unspecified,
    Enterprise,
    Consumer,
    Developer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ValueEnum, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Density {
    #[default]
    Unspecified,
    Compact,
    Comfortable,
    Adaptive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub project: Project,
    #[serde(default)]
    pub stack: Stack,
    #[serde(default)]
    pub ux: Ux,
    #[serde(default)]
    pub data: Data,
    #[serde(default)]
    pub repository: Repository,
    #[serde(default)]
    pub policy: Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub kind: Kind,
    #[serde(default)]
    pub stage: Stage,
    #[serde(default)]
    pub audience: Audience,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Stack {
    pub prefer_rust: bool,
    pub language: String,
    pub rationale: String,
}
impl Default for Stack {
    fn default() -> Self {
        Self {
            prefer_rust: true,
            language: String::new(),
            rationale: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Ux {
    pub density: Density,
    pub primary_workflow: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct Data {
    pub personal_data: bool,
    pub database: bool,
    pub private_user_records: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Repository {
    pub max_tracked_mib: u64,
    pub max_file_mib: u64,
}
impl Default for Repository {
    fn default() -> Self {
        Self {
            max_tracked_mib: 100,
            max_file_mib: 50,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Policy {
    pub required: Vec<String>,
    pub evidence_max_age_days: u64,
}
impl Default for Policy {
    fn default() -> Self {
        Self {
            required: vec![],
            evidence_max_age_days: 30,
        }
    }
}

impl Config {
    pub fn new(name: String, kind: Kind, stage: Stage, audience: Audience) -> Self {
        Self {
            schema_version: 1,
            project: Project {
                name,
                kind,
                stage,
                audience,
            },
            stack: Stack::default(),
            ux: Ux::default(),
            data: Data::default(),
            repository: Repository::default(),
            policy: Policy::default(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1 {
            return Err("unsupported schema_version; Oracle v1 expects 1".into());
        }
        if self.project.name.trim().is_empty()
            || self.project.name.len() > 128
            || self.project.name.chars().any(char::is_control)
        {
            return Err("project.name must be 1–128 bytes without control characters".into());
        }
        if self.repository.max_file_mib == 0
            || self.repository.max_file_mib > 100
            || self.repository.max_tracked_mib == 0
            || self.repository.max_tracked_mib > 1_000_000
        {
            return Err("repository budgets must be positive; max_file_mib <= 100 and max_tracked_mib <= 1000000".into());
        }
        if !(1..=3650).contains(&self.policy.evidence_max_age_days) {
            return Err("policy.evidence_max_age_days must be between 1 and 3650".into());
        }
        for id in &self.policy.required {
            crate::rules::rule(id)?;
        }
        Ok(())
    }
}

pub fn load(root: &Path) -> Result<Config> {
    let path = project::safe_path(root, Path::new("oracle.toml"))?;
    if !path.exists() {
        return Err("oracle.toml is missing; run `oracle init` in this project first".into());
    }
    let raw = project::read_limited(&path, 64 * 1024)?;
    let config: Config = toml::from_str(std::str::from_utf8(&raw)?)
        .map_err(|e| format!("invalid oracle.toml: {}", e.message()))?;
    config.validate()?;
    Ok(config)
}

pub fn init(root: &Path, config: &Config, dry_run: bool) -> Result<Vec<String>> {
    config.validate()?;
    let path = project::safe_path(root, Path::new("oracle.toml"))?;
    let ignore = project::safe_path(root, Path::new(".oracle/.gitignore"))?;
    let existing = path.exists();
    if existing {
        load(root)?;
    }
    if ignore.exists() && !ignore.is_file() {
        return Err(".oracle/.gitignore is not a file".into());
    }
    let changes = vec![
        if existing {
            "Kept existing oracle.toml; edit it to change project choices".into()
        } else {
            "Create oracle.toml".into()
        },
        if ignore.exists() {
            "Preserved .oracle/.gitignore; existing ignore rules and Git tracking remain in effect"
                .into()
        } else {
            "Create .oracle/.gitignore for local evidence (does not untrack existing files)".into()
        },
    ];
    if !dry_run {
        if !existing {
            project::write_new(
                &path,
                format!(
                    "# Oracle v1 project specification. Empty decisions remain unknown.\n{}",
                    toml::to_string_pretty(config)?
                )
                .as_bytes(),
            )?;
        }
        fs::create_dir_all(ignore.parent().ok_or("invalid evidence directory")?)?;
        if !ignore.exists() {
            project::write_new(&ignore, b"*\n")?;
        }
    }
    Ok(changes)
}
