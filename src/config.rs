use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use dirs;
use serde_json::json;

pub fn get_repos() -> Result<Vec<String>> {
    let path = get_config_path("repos.list")?;
    if !path.exists() {
        create_default_repos(&path)?;
    }
    fs::read_to_string(path)
    .context("Failed to read repos.list")?
    .lines()
    .map(|s| Ok(s.trim().to_string()))
    .collect::<Result<Vec<_>>>()
}

pub fn get_tracking_file_path() -> Result<PathBuf> {
    let path = if cfg!(unix) && nix::unistd::Uid::effective().is_root() {
        PathBuf::from("/var/lib/anspm/installed.db")
    } else {
        dirs::config_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
        .join("anspm/installed.db")
    };

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create config directory")?;
    }
    Ok(path)
}

pub fn read_tracking_file() -> Result<serde_json::Value> {
    let path = get_tracking_file_path()?;
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let content = fs::read_to_string(&path).context("Failed to read tracking file")?;
    serde_json::from_str(&content).context("Failed to parse tracking file")
}

pub fn write_tracking_file(data: &serde_json::Value) -> Result<()> {
    let path = get_tracking_file_path()?;
    let content = serde_json::to_string_pretty(data).context("Failed to serialize tracking data")?;
    fs::write(&path, content).context("Failed to write tracking file")
}

fn get_config_path(filename: &str) -> Result<PathBuf> {
    let config_dir = dirs::config_dir()
    .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
    .join("anspm");
    Ok(config_dir.join(filename))
}


fn create_default_repos(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let default_repos = json!({
        "anspm-official": {
            "url": "https://anspm.akaruineko.space",
            "gpg_key": "https://anspm.akaruineko.space/gpg-key.asc"
        }
        // you can add your repositories, just create a pull request!!
    });

    fs::write(
        path,
        serde_json::to_string_pretty(&default_repos)?,
    )?;

    Ok(())
}

pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir()
    .ok_or_else(|| anyhow::anyhow!("Could not find cache directory"))?
    .join("anspm/pkgs");

    if !cache_dir.exists() {
        fs::create_dir_all(&cache_dir)?;
    }

    Ok(cache_dir)
}
