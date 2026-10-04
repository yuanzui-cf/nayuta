use crate::infra::error::{Result, user};
use reqwest::{blocking::Client, header};
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

pub const REPOSITORY: &str = "yuanzui-cf/nayuta";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub prerelease: bool,
    pub draft: bool,
    #[serde(default)]
    pub assets: Vec<Asset>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub name: String,
    pub browser_download_url: String,
}

pub fn cache_dir() -> PathBuf {
    std::env::var_os("NAYUTA_CACHE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            directories::ProjectDirs::from("dev", "nayuta", "nayuta")
                .map(|d| d.cache_dir().to_owned())
                .unwrap_or_else(|| std::env::temp_dir().join("nayuta-cache"))
        })
}

pub fn client() -> Result<Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("nayuta-cli"),
    );
    headers.insert(
        header::ACCEPT,
        header::HeaderValue::from_static("application/vnd.github+json"),
    );
    if let Some(token) = std::env::var("GH_TOKEN")
        .ok()
        .or_else(|| std::env::var("GITHUB_TOKEN").ok())
    {
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| user("network", "token"))?,
        );
    }
    Ok(Client::builder()
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()?)
}

pub fn releases(offline: bool) -> Result<Vec<Release>> {
    let cache = cache_dir().join("releases.json");
    let cached = std::fs::read(&cache)
        .ok()
        .and_then(|data| serde_json::from_slice::<Vec<Release>>(&data).ok());
    if offline {
        return cached.ok_or_else(|| user("not_found", "releases cache"));
    }
    let request = || -> Result<Vec<Release>> {
        let client = client()?;
        let mut all = Vec::new();
        for page in 1..=20 {
            let response = client
                .get(format!(
                    "https://api.github.com/repos/{REPOSITORY}/releases?per_page=100&page={page}"
                ))
                .send()?
                .error_for_status()?;
            let batch: Vec<Release> = response.json()?;
            let count = batch.len();
            all.extend(batch.into_iter().filter(|r| !r.draft));
            if count < 100 {
                break;
            }
        }
        Ok(all)
    };
    match request() {
        Ok(all) => {
            std::fs::create_dir_all(cache.parent().unwrap())?;
            let mut temp = tempfile::NamedTempFile::new_in(cache.parent().unwrap())?;
            use std::io::Write;
            temp.write_all(&serde_json::to_vec(&all)?)?;
            temp.persist(&cache).map_err(|e| e.error)?;
            Ok(all)
        }
        Err(error) => {
            tracing::warn!(error=%error,"GitHub release request failed");
            cached.ok_or_else(|| user("network", "GitHub Releases"))
        }
    }
}

pub fn release(reference: &str) -> Result<Release> {
    let mut url = reqwest::Url::parse(&format!(
        "https://api.github.com/repos/{REPOSITORY}/releases/tags/"
    ))
    .unwrap();
    url.path_segments_mut()
        .unwrap()
        .pop_if_empty()
        .push(reference);
    Ok(client()?.get(url).send()?.error_for_status()?.json()?)
}
