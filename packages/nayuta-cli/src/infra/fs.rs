use crate::infra::{
    error::{Result, user},
    git::github::cache_dir,
};
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

/// All Nayuta writers lock the same canonical project (or creation destination).
pub struct Lock(File);

impl Lock {
    pub fn acquire(path: &Path) -> Result<Self> {
        let directory = cache_dir().join("locks");
        std::fs::create_dir_all(&directory)?;
        let key = format!("{:x}", Sha256::digest(path.as_os_str().as_encoded_bytes()));
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(directory.join(key))?;
        FileExt::try_lock_exclusive(&file)
            .map_err(|_| user("conflict", path.display().to_string()))?;
        Ok(Self(file))
    }
}
impl Drop for Lock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.0);
    }
}

pub fn digest(bytes: &[u8]) -> Vec<u8> {
    Sha256::digest(bytes).to_vec()
}

/// Replace one file without truncation, only if the source still matches the editor snapshot.
pub fn replace(path: &Path, expected: &[u8], bytes: &[u8]) -> Result<()> {
    let old = std::fs::read(path)?;
    if digest(&old) != expected {
        return Err(user("conflict", path.display().to_string()));
    }
    let mut temp =
        tempfile::NamedTempFile::new_in(path.parent().ok_or_else(|| user("validation", "path"))?)?;
    temp.as_file()
        .set_permissions(std::fs::metadata(path)?.permissions())?;
    temp.write_all(bytes)?;
    temp.as_file().sync_all()?;
    // Recheck after preparing the temporary file; advisory locks coordinate other CLI processes.
    if digest(&std::fs::read(path)?) != expected {
        return Err(user("conflict", path.display().to_string()));
    }
    temp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

pub fn inside(root: &Path, path: &Path) -> Result<PathBuf> {
    let resolved = path.canonicalize()?;
    if !resolved.starts_with(root.canonicalize()?) {
        return Err(user("validation", path.display().to_string()));
    }
    Ok(resolved)
}
