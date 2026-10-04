use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{key}: {detail}")]
    User { key: &'static str, detail: String },
    #[error("{path}: {source}")]
    File {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn user(key: &'static str, detail: impl Into<String>) -> Error {
    Error::User {
        key,
        detail: detail.into(),
    }
}

pub fn read(path: &std::path::Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| Error::File {
        path: path.into(),
        source,
    })
}
