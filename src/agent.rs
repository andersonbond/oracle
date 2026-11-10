use crate::{Result, project};
use clap::ValueEnum;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Client {
    Codex,
    Claude,
    Generic,
}

const FILES: &[(&str, &[u8])] = &[
    ("SKILL.md", include_bytes!("../skills/oracle/SKILL.md")),
    (
        "references/ui-design.md",
        include_bytes!("../skills/oracle/references/ui-design.md"),
    ),
    (
        "references/emil-design-engineering.md",
        include_bytes!("../skills/oracle/references/emil-design-engineering.md"),
    ),
    (
        "references/emil-ui-cheatsheet.md",
        include_bytes!("../skills/oracle/references/emil-ui-cheatsheet.md"),
    ),
    (
        "references/emil-LICENSE.txt",
        include_bytes!("../skills/oracle/references/emil-LICENSE.txt"),
    ),
    (
        "references/workflow.md",
        include_bytes!("../skills/oracle/references/workflow.md"),
    ),
    (
        "references/compact-ui.md",
        include_bytes!("../skills/oracle/references/compact-ui.md"),
    ),
    (
        "references/simple-projects.md",
        include_bytes!("../skills/oracle/references/simple-projects.md"),
    ),
    (
        "references/realtime.md",
        include_bytes!("../skills/oracle/references/realtime.md"),
    ),
    (
        "references/database.md",
        include_bytes!("../skills/oracle/references/database.md"),
    ),
    (
        "references/image-uploads.md",
        include_bytes!("../skills/oracle/references/image-uploads.md"),
    ),
    (
        "assets/simple-projects.svg",
        include_bytes!("../skills/oracle/assets/simple-projects.svg"),
    ),
    (
        "assets/compact-workspace.svg",
        include_bytes!("../skills/oracle/assets/compact-workspace.svg"),
    ),
    (
        "assets/adaptive-workspace.svg",
        include_bytes!("../skills/oracle/assets/adaptive-workspace.svg"),
    ),
];

fn directory(client: Client) -> PathBuf {
    PathBuf::from(match client {
        Client::Codex | Client::Generic => ".agents/skills/oracle",
        Client::Claude => ".claude/skills/oracle",
    })
}

pub fn install(root: &Path, client: Client, remove: bool, dry_run: bool) -> Result<Vec<String>> {
    let dir = directory(client);
    let mut changes = vec![];
    let mut pending = vec![];
    // Preflight the entire package before making any changes. Existing content
    // must match byte-for-byte; no AGENTS.md or client settings are touched.
    for (name, content) in FILES {
        let relative = dir.join(name);
        let path = project::safe_path(root, &relative)?;
        if path.exists() {
            if project::read_limited(&path, 256 * 1024)? != *content {
                return Err(format!("preserving modified/conflicting skill file {}; merge it manually before install/remove", relative.display()).into());
            }
            if remove {
                pending.push((path, *content));
                changes.push(format!("Remove {}", relative.display()));
            } else {
                changes.push(format!("Already current: {}", relative.display()));
            }
        } else if !remove {
            pending.push((path, *content));
            changes.push(format!("Create {}", relative.display()));
        }
    }
    if !dry_run {
        for (path, content) in pending {
            if remove {
                fs::remove_file(path)?;
            } else {
                project::write_new(&path, content)?;
            }
        }
        if remove {
            let parent = project::safe_path(root, &dir)?;
            // Only remove empty package-owned directories, never client roots.
            for path in [parent.join("references"), parent.join("assets"), parent] {
                let _ = fs::remove_dir(path);
            }
        }
    }
    if changes.is_empty() {
        changes.push("No installed Oracle files to remove".into());
    }
    Ok(changes)
}
