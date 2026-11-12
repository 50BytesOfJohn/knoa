//! Atomic replacement of the dev build's plain-text credentials.

use std::{collections::HashMap, io::Write as _, path::Path};

use anyhow::Context as _;

use super::queue::Operation;

pub(super) fn read(path: &Path) -> anyhow::Result<HashMap<String, String>> {
    match std::fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text).context("dev credentials file is invalid"),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(err) => Err(err.into()),
    }
}

pub(super) fn apply(
    path: &Path,
    url: &str,
    operation: Operation,
) -> anyhow::Result<Option<String>> {
    let mut all = read(path)?;
    match operation {
        Operation::Write(secret) => {
            all.insert(url.to_owned(), secret);
        }
        Operation::Delete => {
            all.remove(url);
        }
        Operation::Read => return Ok(all.remove(url)),
    }
    write(path, &all)?;
    Ok(None)
}

fn write(path: &Path, all: &HashMap<String, String>) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt as _;

    let parent = path.parent().context("credentials path has no parent")?;
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    // The replacement must remain private even if the existing file had broader access.
    temporary
        .as_file()
        .set_permissions(std::fs::Permissions::from_mode(0o600))?;
    temporary.write_all(&serde_json::to_vec_pretty(all)?)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::Read as _, os::unix::fs::PermissionsExt as _};

    use super::*;

    #[test]
    fn replacement_is_atomic_and_private() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("credentials.json");
        let old = HashMap::from([("cloud".into(), "old-refresh".into())]);
        write(&path, &old).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        let mut previous = File::open(&path).unwrap();
        let new = HashMap::from([("cloud".into(), "new-refresh".into())]);

        write(&path, &new).unwrap();

        let mut text = String::new();
        previous.read_to_string(&mut text).unwrap();
        assert_eq!(
            serde_json::from_str::<HashMap<String, String>>(&text).unwrap(),
            old
        );
        assert_eq!(read(&path).unwrap(), new);
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
    }
}
