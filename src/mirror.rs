use std::fs;
use std::path::{self, PathBuf};

use tokio::io;

use crate::utils::{SafeJoin, list_dir_sorted_by_semver};

pub struct MirrorDirectory {
    root: path::PathBuf,
    releases: path::PathBuf,
    extensions: path::PathBuf,
    static_files: path::PathBuf,
}

fn check_dir_exists(p: &path::Path) -> anyhow::Result<()> {
    if !p.exists() {
        anyhow::bail!("directory '{}' does not exists", p.display());
    }
    if !p.is_dir() {
        anyhow::bail!("'{}' is not a directory", p.display());
    }
    Ok(())
}

impl MirrorDirectory {
    pub fn new<P: Into<PathBuf>>(root: P) -> Self {
        let root = root.into();
        let releases = root.join("releases");
        let extensions = root.join("extensions");
        let static_files = root.join("static_files");

        Self {
            root,
            releases,
            extensions,
            static_files,
        }
    }

    fn inner_subdirectories(&self) -> Vec<&path::Path> {
        vec![
            self.releases_dir(),
            self.extensions_dir(),
            self.static_files_dir(),
        ]
    }

    pub fn releases_dir(&self) -> &path::Path {
        &self.releases
    }

    pub fn extensions_dir(&self) -> &path::Path {
        &self.extensions
    }

    pub fn static_files_dir(&self) -> &path::Path {
        &self.static_files
    }

    pub fn check_valid(&self) -> anyhow::Result<()> {
        check_dir_exists(&self.root)?;

        for sub_directory in self.inner_subdirectories() {
            check_dir_exists(sub_directory)?;
        }
        Ok(())
    }

    pub fn create_dir(&self) -> io::Result<()> {
        for sub_directory in self.inner_subdirectories() {
            fs::create_dir_all(sub_directory)?;
        }
        Ok(())
    }

    pub async fn version_dir(&self, version: &str) -> anyhow::Result<path::PathBuf> {
        if version == "latest" {
            return Ok(list_dir_sorted_by_semver(self.releases_dir())
                .await?
                .first()
                .ok_or_else(|| anyhow::anyhow!("No versions available"))?
                .path());
        }
        self.releases_dir()
            .safe_join(version)
            .map_err(|e| anyhow::anyhow!("No version {version}: {e}"))
    }
}
