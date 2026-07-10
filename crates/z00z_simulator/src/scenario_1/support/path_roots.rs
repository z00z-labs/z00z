use std::path::{Component, Path, PathBuf};

pub(crate) fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

pub(crate) fn simulator_root() -> PathBuf {
    normalize_path(&PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

pub(crate) fn workspace_root() -> PathBuf {
    normalize_path(&simulator_root().join("../.."))
}

pub(crate) fn workspace_target_root() -> PathBuf {
    normalize_path(&workspace_root().join("target"))
}

pub(crate) fn resolve_workspace_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if path.is_absolute() {
        normalize_path(path)
    } else {
        normalize_path(&workspace_root().join(path))
    }
}
