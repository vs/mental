use super::*;

/// A `ReadOnly` over this crate's directory that holds one file, `held.py`,
/// as text that differs from anything on disk.
fn system() -> (ReadOnly, SystemPathBuf) {
    let dir = SystemPathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let held = dir.join("held.py");
    let files = BTreeMap::from([(held, "held = 1\n".to_owned())]);
    (ReadOnly::new(&dir, files), dir)
}

#[test]
fn a_held_file_reads_as_held_and_others_from_disk() {
    let (system, dir) = system();
    assert_eq!(
        system.read_to_string(&dir.join("held.py")).unwrap(),
        "held = 1\n"
    );
    assert!(
        system
            .read_to_string(&dir.join("Cargo.toml"))
            .unwrap()
            .contains("mental-extract")
    );
}

#[test]
fn paths_are_looked_up_on_disk() {
    let (system, dir) = system();
    let manifest = dir.join("Cargo.toml");
    assert!(system.path_exists(&manifest));
    assert!(!system.path_exists(&dir.join("no-such-file")));
    assert!(system.is_same_file(&manifest, &manifest).unwrap());
    assert!(
        !system
            .is_same_file(&manifest, &dir.join("src/lib.rs"))
            .unwrap()
    );
    assert!(system.path_metadata(&manifest).is_ok());
    assert_eq!(system.current_directory(), &*dir);
}

#[test]
fn nothing_outside_the_files_is_seen_or_written() {
    let (system, _) = system();
    assert!(system.which("uv").is_err());
    assert_eq!(system.user_config_directory(), None);
    assert_eq!(system.cache_dir(), None);
    assert!(system.as_writable().is_none());
    assert!(system.env_var("PATH").is_err());
    let virtual_path = SystemVirtualPath::new("untitled:1");
    assert!(system.read_virtual_path_to_string(virtual_path).is_err());
}

#[test]
fn notebooks_read_from_disk_and_the_system_knows_its_type() {
    let (mut system, dir) = system();
    assert!(system.read_to_notebook(&dir.join("no-such.ipynb")).is_err());
    let virtual_path = SystemVirtualPath::new("untitled:1.ipynb");
    assert!(system.read_virtual_path_to_notebook(virtual_path).is_err());
    assert!(system.as_any().downcast_ref::<ReadOnly>().is_some());
    assert!(system.as_any_mut().downcast_mut::<ReadOnly>().is_some());
    assert!(
        system
            .dyn_clone()
            .as_any()
            .downcast_ref::<ReadOnly>()
            .is_some()
    );
}
