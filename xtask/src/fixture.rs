//! Throwaway repository trees for tests.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A directory under the system temporary directory, removed on drop.
pub struct TempRepo(PathBuf);

impl TempRepo {
    /// Creates an empty tree with a name unique to this process and call.
    pub fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!("xtask-{}-{n}", std::process::id()));
        fs::create_dir_all(&root).expect("create the temporary repository");
        Self(root)
    }

    /// Creates the tree with `files` given as (relative path, contents) pairs.
    pub fn with(files: &[(&str, &str)]) -> Self {
        let repo = Self::new();
        for (path, text) in files {
            repo.write(path, text);
        }
        repo
    }

    /// Writes one file, creating its parent directories.
    pub fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().expect("a file path has a parent"))
            .expect("create parent directories");
        fs::write(&path, text).expect("write the fixture file");
    }

    /// Removes one file.
    pub fn remove(&self, path: &str) {
        fs::remove_file(self.0.join(path)).expect("remove the fixture file");
    }

    /// Returns the root of the tree.
    pub fn root(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRepo {
    fn drop(&mut self) {
        // Best effort: a leftover directory in the temporary directory is harmless.
        let _ = fs::remove_dir_all(&self.0);
    }
}
