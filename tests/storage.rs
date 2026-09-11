use system_designer::{model::*, storage};
#[test]
fn project_file_save_and_open() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("design.json");
    let p = Project::blank();
    let stamp = storage::save_project(&path, &p, None).expect("save");
    let (q, current) = storage::read_project(&path).expect("open");
    assert_eq!(p, q);
    assert_eq!(stamp, current);
}
#[test]
fn save_keeps_prior_valid_backup() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("design.json");
    let mut p = Project::blank();
    let old = p.clone();
    let stamp = storage::save_project(&path, &p, None).expect("save");
    p.name = "Next".into();
    storage::save_project(&path, &p, Some(&stamp)).expect("save again");
    assert_eq!(
        storage::read_project(&storage::backup_path(&path))
            .expect("backup")
            .0,
        old
    );
    assert_eq!(storage::read_project(&path).expect("current").0, p);
}
#[test]
fn external_edit_is_not_overwritten() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("design.json");
    let p = Project::blank();
    let stamp = storage::save_project(&path, &p, None).expect("save");
    let mut external = p.clone();
    external.name = "Someone else's edit".into();
    let text = serde_json::to_string(&external).expect("json");
    std::fs::write(&path, text.as_bytes()).expect("external write");
    assert!(storage::save_project(&path, &p, Some(&stamp)).is_err());
    assert_eq!(std::fs::read_to_string(&path).expect("read"), text);
}
#[test]
fn missing_expected_file_is_not_silently_recreated() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("design.json");
    let p = Project::blank();
    assert!(storage::save_project(&path, &p, Some("old fingerprint")).is_err());
    assert!(!path.exists());
}
#[test]
fn invalid_target_is_not_destroyed() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("design.json");
    std::fs::write(&path, "damaged project").expect("write");
    let stamp = storage::stamp(&path).expect("stamp");
    assert!(storage::save_project(&path, &Project::blank(), stamp.as_deref()).is_err());
    assert_eq!(
        std::fs::read_to_string(&path).expect("read"),
        "damaged project"
    );
}
#[test]
fn invalid_candidate_never_replaces_disk() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("design.json");
    let mut p = Project::blank();
    let old = p.clone();
    let stamp = storage::save_project(&path, &p, None).expect("save");
    p.root = "bad".into();
    assert!(storage::save_project(&path, &p, Some(&stamp)).is_err());
    assert_eq!(storage::read_project(&path).expect("read").0, old);
}
#[test]
fn recovery_round_trip() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("recovery.json");
    let p = Project::blank();
    storage::write_recovery(&path, &p, None).expect("recovery");
    let r = storage::read_recovery(&path).expect("read");
    assert_eq!(r.project, p);
    assert!(r.original_path.is_none());
}
#[test]
fn invalid_recovery_preserves_bytes() {
    let dir = tempfile::tempdir().expect("dir");
    let path = dir.path().join("recovery.json");
    std::fs::write(&path, "invalid").expect("write");
    assert!(storage::read_recovery(&path).is_err());
    assert_eq!(std::fs::read_to_string(&path).expect("read"), "invalid");
}
#[test]
fn io_failure_is_visible() {
    let dir = tempfile::tempdir().expect("dir");
    assert!(storage::atomic_write(&dir.path().join("missing/file.json"), b"x").is_err());
}
