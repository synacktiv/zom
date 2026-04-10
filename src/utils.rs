use std::path::PathBuf;
use std::{io, path};

use futures_util::StreamExt;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub async fn list_dir<P: AsRef<path::Path>>(path: P) -> std::io::Result<Vec<tokio::fs::DirEntry>> {
    let path = path.as_ref();
    let mut entries = vec![];

    let mut stream = tokio::fs::read_dir(path).await?;
    while let Some(item) = stream.next_entry().await? {
        entries.push(item);
    }

    Ok(entries)
}

pub async fn list_dir_sorted_by_semver<P: AsRef<path::Path>>(
    path: P,
) -> std::io::Result<Vec<tokio::fs::DirEntry>> {
    let mut list = list_dir(path).await?;
    // sort by semver and put invalid entries at the end of the list
    list.sort_by_key(|entry| {
        std::cmp::Reverse(
            entry
                .file_name()
                .to_str()
                .and_then(|f| semver::Version::parse(f).ok()),
        )
    });
    Ok(list)
}

pub async fn download_to_file<P: AsRef<path::Path>>(
    req: reqwest::RequestBuilder,
    path: P,
) -> anyhow::Result<()> {
    let mut file = fs::File::create_new(path.as_ref()).await?;

    let res = req.send().await?.error_for_status()?;

    let mut stream = res.bytes_stream();
    while let Some(item) = stream.next().await {
        file.write_all(&item?).await?;
    }

    Ok(())
}

pub trait SafeJoin {
    /// [`std::path::Path::join`] but checks that the result path is a children
    /// of self. This function does not resolves symlinks.
    ///
    /// ## Error
    ///
    /// - If computed path cannot be turned into an absolute path
    /// - If computed path is not a children of self
    fn safe_join<P: AsRef<path::Path>>(&self, path: P) -> std::io::Result<path::PathBuf>;
}

impl SafeJoin for path::Path {
    fn safe_join<P: AsRef<Self>>(&self, path: P) -> std::io::Result<path::PathBuf> {
        let unchecked_path = self.join(path.as_ref());

        let mut unchecked_abs = PathBuf::new();
        for component in path::absolute(&unchecked_path)?.components() {
            match component {
                path::Component::Prefix(prefix_component) => {
                    unchecked_abs.push(prefix_component.as_os_str());
                }
                path::Component::RootDir => {
                    unchecked_abs = PathBuf::new();

                    // we do want to create a new root pathbuf
                    #[expect(clippy::path_buf_push_overwrite)]
                    unchecked_abs.push("/");
                }
                path::Component::CurDir => {}
                path::Component::ParentDir => {
                    unchecked_abs.pop();
                }
                path::Component::Normal(os_str) => unchecked_abs.push(os_str),
            }
        }

        let self_abs = path::absolute(self)?;

        if !unchecked_abs.starts_with(&self_abs) {
            return Err(io::Error::other(format!(
                "file {} goes beyond {}",
                unchecked_abs.display(),
                self_abs.display(),
            )));
        }

        Ok(unchecked_path)
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use crate::utils::SafeJoin;

    #[test]
    pub fn path_traversal_tests() {
        let root = PathBuf::from_str("/tmp/dir/").expect("cannot build base path");

        assert!(
            root.safe_join("/foo").is_err(),
            "absolute path are not handled properly..."
        );

        assert!(
            root.safe_join("../../a").is_err(),
            "parent dir evasion is not handled properly..."
        );

        assert!(root.safe_join("./a1").is_ok());

        assert!(
            root.safe_join("/").is_err(),
            "parent dir evasion is not handled properly..."
        );

        assert!(root.safe_join("../dir").is_ok());
    }
}
