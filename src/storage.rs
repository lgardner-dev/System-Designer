//! Local files, conflict checks, same-directory replacement, and isolated recovery snapshots.
//! No service, shell execution, embedded browser, or implicit project code execution.
use crate::{exchange::digest, model::*};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
pub fn read_project(path: &Path) -> Result<(Project, String)> {
    let bytes = fs::read(path).map_err(ModelError::one)?;
    let text = std::str::from_utf8(&bytes).map_err(ModelError::one)?;
    Ok((parse(text)?, digest(&bytes)))
}
pub fn stamp(path: &Path) -> Result<Option<String>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(digest(&bytes))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(ModelError::one(e)),
    }
}
fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}
pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<()> {
    let dir = parent(path);
    let mut temp = tempfile::NamedTempFile::new_in(dir).map_err(ModelError::one)?;
    temp.write_all(bytes).map_err(ModelError::one)?;
    temp.as_file().sync_all().map_err(ModelError::one)?;
    temp.persist(path).map_err(ModelError::one)?;
    #[cfg(unix)]
    fs::File::open(dir)
        .and_then(|f| f.sync_all())
        .map_err(ModelError::one)?;
    Ok(())
}
pub fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_os_string();
    name.push(".bak");
    PathBuf::from(name)
}
/// Checks the original fingerprint, preserves a validated previous file, then replaces it.
/// This detects ordinary external edits; it is not a distributed lock or race-free CAS.
pub fn save_project(path: &Path, p: &Project, expected: Option<&str>) -> Result<String> {
    validate(p)?;
    if stamp(path)?.as_deref() != expected {
        return Err(ModelError::one(
            "file changed outside this app; reopen/reconcile or Save As a new file",
        ));
    }
    if path.exists() {
        let (old, old_stamp) = read_project(path)?;
        // Do not destroy a damaged/unrecognized file.
        let _ = old;
        if Some(old_stamp.as_str()) != expected {
            return Err(ModelError::one("file changed while preparing save"));
        }
        let previous = fs::read(path).map_err(ModelError::one)?;
        if Some(digest(&previous).as_str()) != expected {
            return Err(ModelError::one("file changed while preparing backup"));
        }
        atomic_write(&backup_path(path), &previous)?;
    }
    let bytes = serde_json::to_vec_pretty(p).map_err(ModelError::one)?;
    // Recheck after backup I/O. Another process may still race after this check.
    if stamp(path)?.as_deref() != expected {
        return Err(ModelError::one("file changed before replacement"));
    }
    atomic_write(path, &bytes)?;
    Ok(digest(&bytes))
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Recovery {
    pub format: String,
    pub version: u32,
    pub original_path: Option<PathBuf>,
    pub project: Project,
}
pub fn recovery_dir() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("org", "SystemDesigner", "SystemDesigner")
        .ok_or_else(|| ModelError::one("no writable user-data directory"))?;
    let dir = dirs.data_local_dir().join("recovery");
    fs::create_dir_all(&dir).map_err(ModelError::one)?;
    Ok(dir)
}
pub fn new_recovery_path() -> Result<PathBuf> {
    Ok(recovery_dir()?.join(format!("{}.sd-recovery.json", uuid::Uuid::new_v4())))
}
pub fn write_recovery(path: &Path, project: &Project, original: Option<&Path>) -> Result<()> {
    validate(project)?;
    let record = Recovery {
        format: "system-designer-recovery".into(),
        version: 1,
        original_path: original.map(Path::to_owned),
        project: project.clone(),
    };
    atomic_write(
        path,
        &serde_json::to_vec_pretty(&record).map_err(ModelError::one)?,
    )
}
pub fn read_recovery(path: &Path) -> Result<Recovery> {
    let bytes = fs::read(path).map_err(ModelError::one)?;
    let r: Recovery = serde_json::from_slice(&bytes).map_err(ModelError::one)?;
    if r.format != "system-designer-recovery" || r.version != 1 {
        return Err(ModelError::one("unsupported recovery format"));
    }
    validate(&r.project)?;
    Ok(r)
}
pub fn recovery_files() -> Result<Vec<PathBuf>> {
    let mut paths = vec![];
    for e in fs::read_dir(recovery_dir()?).map_err(ModelError::one)? {
        let p = e.map_err(ModelError::one)?.path();
        if p.file_name()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.ends_with(".sd-recovery.json"))
        {
            paths.push(p);
        }
    }
    paths.sort_by_key(|p| std::cmp::Reverse(fs::metadata(p).and_then(|m| m.modified()).ok()));
    Ok(paths)
}
