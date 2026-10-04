pub mod github;
pub mod template;
use crate::infra::error::{Result, user};
use std::{
    path::Path,
    process::{Command, Output},
};

pub fn output(cwd: &Path, args: &[&str]) -> Result<Output> {
    let output = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_HTTP_LOW_SPEED_LIMIT", "1000")
        .env("GIT_HTTP_LOW_SPEED_TIME", "30")
        .output()?;
    if !output.status.success() {
        return Err(user("git", String::from_utf8_lossy(&output.stderr).trim()));
    }
    Ok(output)
}

pub fn run(cwd: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8_lossy(&output(cwd, args)?.stdout)
        .trim()
        .into())
}

pub fn identity(cwd: &Path) -> Result<(String, String)> {
    let value = run(cwd, &["var", "GIT_AUTHOR_IDENT"])
        .map_err(|_| user("identity", cwd.display().to_string()))?;
    let (name, rest) = value
        .rsplit_once(" <")
        .ok_or_else(|| user("identity", &value))?;
    let email = rest.split('>').next().unwrap_or_default();
    if name.is_empty() || email.is_empty() {
        return Err(user("identity", &value));
    }
    Ok((name.into(), email.into()))
}

pub fn initialize(root: &Path, fallback: &(String, String)) -> Result<()> {
    run(root, &["init", "--initial-branch=main"])?;
    let identity = identity(root).unwrap_or_else(|_| fallback.clone());
    run(root, &["add", "--all"])?;
    let output = Command::new("git")
        .current_dir(root)
        .args([
            "-c",
            &format!("user.name={}", identity.0),
            "-c",
            &format!("user.email={}", identity.1),
            "commit",
            "-m",
            "chore(site): Initialize Nayuta blog",
        ])
        .env("GIT_AUTHOR_NAME", &identity.0)
        .env("GIT_AUTHOR_EMAIL", &identity.1)
        .env("GIT_COMMITTER_NAME", &identity.0)
        .env("GIT_COMMITTER_EMAIL", &identity.1)
        .output()?;
    if !output.status.success() {
        return Err(user("git", String::from_utf8_lossy(&output.stderr).trim()));
    }
    if run(root, &["rev-list", "--all", "--count"])? != "1" {
        return Err(user("git", "initial commit count"));
    }
    Ok(())
}
