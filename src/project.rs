use crate::Result;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    io::{self, Read, Write},
    path::{Component, Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use walkdir::WalkDir;

const MAX_FILES: usize = 20_000;
const MAX_HASH_BYTES: u64 = 128 * 1024 * 1024;
const MAX_GIT_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const GENERATED: &[&str] = &[
    "target",
    "node_modules",
    ".venv",
    "venv",
    "dist",
    "build",
    ".next",
    "coverage",
    "__pycache__",
    ".cache",
    ".oracle",
];

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub bytes: u64,
    pub tracked: bool,
    pub generated: bool,
    pub secret_candidate: bool,
}

#[derive(Debug, Serialize)]
pub struct Snapshot {
    pub git: bool,
    pub files: Vec<FileInfo>,
    pub tracked_bytes: u64,
    pub inspected_bytes: u64,
    pub git_object_bytes: Option<u64>,
    pub fingerprint: String,
    pub complete: bool,
    pub notes: Vec<String>,
}

pub fn safe_path(root: &Path, relative: &Path) -> Result<PathBuf> {
    if relative.as_os_str().is_empty() {
        return Err("empty project-relative path".into());
    }
    let mut target = root.to_path_buf();
    for part in relative.components() {
        let Component::Normal(name) = part else {
            return Err("paths must be relative and contain no '..' or root prefix".into());
        };
        target.push(name);
        match fs::symlink_metadata(&target) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err("refusing a symlink in a project path".into());
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(target)
}

pub fn read_limited(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let meta = fs::symlink_metadata(path)?;
    if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > limit {
        return Err(format!("input must be a regular file no larger than {limit} bytes").into());
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?
        .take(limit + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("input grew beyond the read limit".into());
    }
    Ok(bytes)
}

pub fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

pub fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn file_digest(root: &Path, relative: &Path) -> Result<String> {
    let path = safe_path(root, relative)?;
    Ok(digest(&read_limited(&path, MAX_ARTIFACT_BYTES)?))
}

fn bounded_output(command: &mut Command, limit: usize, timeout: Duration) -> io::Result<Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()?;
    let stdout = child
        .stdout
        .take()
        .expect("stdout was configured as a pipe");
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout
            .take(limit as u64 + 1)
            .read_to_end(&mut bytes)
            .and_then(|_| {
                if bytes.len() > limit {
                    Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Git output exceeds the 8 MiB safety limit",
                    ))
                } else {
                    Ok(bytes)
                }
            });
        let _ = sender.send(result);
    });
    let started = Instant::now();
    let mut output = None;
    let mut status = None;
    let result = loop {
        if output.is_none() {
            match receiver.try_recv() {
                Ok(Ok(bytes)) => output = Some(bytes),
                Ok(Err(error)) => break Err(error),
                Err(mpsc::TryRecvError::Disconnected) => {
                    break Err(io::Error::other("Git output reader disconnected"));
                }
                Err(mpsc::TryRecvError::Empty) => (),
            }
        }
        if status.is_none() {
            match child.try_wait() {
                Ok(result) => status = result,
                Err(error) => break Err(error),
            }
        }
        if let (Some(status), Some(stdout)) = (status, output.as_mut()) {
            break Ok(Output {
                status,
                stdout: std::mem::take(stdout),
                stderr: vec![],
            });
        }
        if started.elapsed() >= timeout {
            break Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "Git inspection timed out",
            ));
        }
        thread::sleep(Duration::from_millis(5));
    };
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

fn git(root: &Path, args: &[&str]) -> Result<Option<Output>> {
    let mut command = Command::new("git");
    command
        .arg("-c")
        .arg("core.fsmonitor=false")
        .arg("-C")
        .arg(root)
        .args(args)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env_remove("GIT_COMMON_DIR");
    match bounded_output(&mut command, MAX_GIT_BYTES, Duration::from_secs(10)) {
        Ok(output) => Ok(output.status.success().then_some(output)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn paths(bytes: &[u8]) -> Result<BTreeSet<String>> {
    if bytes.len() > MAX_GIT_BYTES {
        return Err("Git file inventory exceeds the 8 MiB safety limit".into());
    }
    let mut out = BTreeSet::new();
    for name in bytes.split(|b| *b == 0).filter(|b| !b.is_empty()) {
        out.insert(
            std::str::from_utf8(name)
                .map_err(|_| "non-UTF-8 filename; inventory cannot be represented safely")?
                .to_owned(),
        );
    }
    Ok(out)
}

fn generated(path: &str) -> bool {
    path.split('/').any(|part| GENERATED.contains(&part))
}

pub fn secret_name(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path).to_lowercase();
    let example = [".example", ".sample", ".template"]
        .iter()
        .any(|suffix| name.ends_with(suffix));
    (!example && (name == ".env" || name.starts_with(".env.")))
        || name.ends_with(".pem")
        || name.ends_with(".key")
        || name == "id_rsa"
        || name == "id_ed25519"
}

pub fn scan(root: &Path) -> Result<Snapshot> {
    let mut complete = true;
    let mut notes = vec!["Inspection covers the working tree, not remote GitHub storage, LFS objects, or every historical blob.".into()];
    let tracked_output = git(root, &["ls-files", "-z", "--cached", "--", "."])?;
    let is_git = tracked_output.is_some();
    let tracked = if let Some(o) = tracked_output {
        paths(&o.stdout)?
    } else {
        BTreeSet::new()
    };
    let mut names = tracked.clone();
    if is_git {
        let other = git(
            root,
            &[
                "ls-files",
                "-z",
                "--others",
                "--exclude-standard",
                "--",
                ".",
            ],
        )?
        .ok_or("Git untracked-file discovery failed")?;
        names.extend(paths(&other.stdout)?);
        notes.push("Git-tracked and unignored files are inspected. Local .oracle state and secret file contents are excluded from hashing.".into());
    } else {
        notes.push("No usable Git index: filesystem fallback excludes dependency/build folders; tracked sizes and history are unknown.".into());
        for (visited, entry) in WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                e.depth() == 0
                    || (!e.file_type().is_dir()
                        || (!GENERATED.contains(&e.file_name().to_string_lossy().as_ref())
                            && e.file_name() != ".git"
                            && e.file_name() != ".oracle"))
            })
            .enumerate()
        {
            if visited >= MAX_FILES {
                return Err(
                    "filesystem discovery exceeds 20000 entries; narrow the project root".into(),
                );
            }
            let entry = match entry {
                Ok(e) => e,
                Err(_) => {
                    complete = false;
                    continue;
                }
            };
            if entry.file_type().is_file() || entry.file_type().is_symlink() {
                let relative = entry.path().strip_prefix(root)?;
                let name = relative
                    .to_str()
                    .ok_or("non-UTF-8 filename")?
                    .replace(std::path::MAIN_SEPARATOR, "/");
                names.insert(name);
                if names.len() > MAX_FILES {
                    return Err("project exceeds 20000 files; narrow the project root or ignore generated content".into());
                }
            }
        }
    }
    names.retain(|name| {
        name != ".git"
            && !name.starts_with(".git/")
            && (!name.starts_with(".oracle/") || tracked.contains(name))
    });
    // Configuration is an explicit input even when a user's ignore rules hide it.
    if root.join("oracle.toml").exists() {
        names.insert("oracle.toml".into());
    }
    if names.len() > MAX_FILES {
        return Err(
            "project exceeds 20000 files; narrow the project root or ignore generated content"
                .into(),
        );
    }
    let mut hasher = Sha256::new();
    hasher.update(b"oracle-snapshot-v2\0");
    let mut files = vec![];
    let mut tracked_bytes: u64 = 0;
    let mut inspected_bytes: u64 = 0;
    let mut hashed_bytes: u64 = 0;
    for name in names {
        let local_state = name.starts_with(".oracle/");
        if !local_state {
            hasher.update((name.len() as u64).to_le_bytes());
            hasher.update(name.as_bytes());
        }
        let path = match safe_path(root, Path::new(&name)) {
            Ok(p) => p,
            Err(_) => {
                complete = false;
                hasher.update(b"unsafe-path");
                continue;
            }
        };
        let meta = match fs::metadata(&path) {
            Ok(m) if m.is_file() => m,
            _ => {
                complete = false;
                hasher.update(b"missing-or-non-file");
                continue;
            }
        };
        let size = meta.len();
        let is_tracked = tracked.contains(&name);
        let secret = secret_name(&name);
        if is_tracked {
            tracked_bytes = tracked_bytes.saturating_add(size);
        }
        inspected_bytes = inspected_bytes.saturating_add(size);
        files.push(FileInfo {
            generated: generated(&name),
            secret_candidate: secret,
            tracked: is_tracked,
            path: name,
            bytes: size,
        });
        // Local evidence is not a source input, but force-tracking it must not
        // hide its filenames or size from repository checks.
        if local_state {
            continue;
        }
        hasher.update(size.to_le_bytes());
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            hasher.update((meta.permissions().mode() & 0o111).to_le_bytes());
        }
        if secret || size > MAX_ARTIFACT_BYTES || hashed_bytes.saturating_add(size) > MAX_HASH_BYTES
        {
            complete = false;
            hasher.update(b"content-not-hashed");
            continue;
        }
        match read_limited(&path, MAX_ARTIFACT_BYTES) {
            Ok(content) => {
                hashed_bytes += content.len() as u64;
                hasher.update(Sha256::digest(&content));
            }
            Err(_) => {
                complete = false;
                hasher.update(b"unreadable");
            }
        }
    }
    if !complete {
        notes.push("Snapshot is incomplete (symlink, secret candidate, missing/unreadable file, submodule, or hashing limit). Evidence cannot be marked current. Hash limits: 16 MiB/file, 128 MiB total.".into());
    }
    let git_object_bytes = if root.join(".git").exists() {
        git(root, &["count-objects", "-v"])?
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|raw| {
                raw.lines()
                    .filter_map(|line| line.split_once(": "))
                    .filter(|(key, _)| *key == "size" || *key == "size-pack")
                    .filter_map(|(_, value)| value.parse::<u64>().ok())
                    .fold(0u64, u64::saturating_add)
                    .saturating_mul(1024)
            })
    } else {
        None
    };
    Ok(Snapshot {
        git: is_git,
        files,
        tracked_bytes,
        inspected_bytes,
        git_object_bytes,
        fingerprint: format!("{:x}", hasher.finalize()),
        complete,
        notes,
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn git_output_limit_stops_a_continuous_producer() {
        let start = Instant::now();
        let error = bounded_output(
            Command::new("sh").args(["-c", "while :; do printf 0123456789; done"]),
            32,
            Duration::from_secs(5),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert!(start.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn git_timeout_stops_a_process_that_keeps_stdout_open() {
        let start = Instant::now();
        let error = bounded_output(
            Command::new("sh").args(["-c", "printf partial; exec sleep 30"]),
            1024,
            Duration::from_millis(50),
        )
        .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        assert!(start.elapsed() < Duration::from_secs(5));
    }
}
