use crate::cli::{print_error, print_info, print_success};
use anyhow::{Context, Result};
use colored::Colorize;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use crate::config::get_repos;

#[derive(Debug, Clone, Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub url: String,
    pub os: String,
    pub arch: String,
    pub deps: Vec<String>,
    pub author: String,
    pub license: String,
    
    #[serde(skip)]
    pub base_url: String, 
}

#[derive(Debug, Deserialize)]
pub struct _Meta {
    pub name: String,
    pub version: String,
    pub arch: String,
    pub os: String,
    pub deps: Vec<String>,
    pub author: String,
    pub license: String,
    pub description: Option<String>,
}

const REPOS_CONFIG: &str = "repos.list";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct RepoConfig {
    pub url: String,
    pub gpg_key: Option<String>,
}

fn verify_repo_index(repo_url: &str) -> Result<()> {
    let index = download_file(&format!("{}/index.json", repo_url))?;
    let signature = download_file(&format!("{}/index.json.asc", repo_url))?;

    let temp_dir = tempfile::tempdir()?;
    let index_path = temp_dir.path().join("index.json");
    let sig_path = temp_dir.path().join("index.sig");

    std::fs::write(&index_path, index)?;
    std::fs::write(&sig_path, signature)?;

    let status = Command::new("gpg")
    .args(&["--verify", sig_path.to_str().unwrap(), index_path.to_str().unwrap()])
    .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("❌ Repository signature verification failed!"));
    }

    Ok(())
}

pub fn fetch_repository(repo_url: &str) -> Result<HashMap<String, PackageInfo>> {
    let url = format!("{}/index.json", repo_url.trim_end_matches('/'));
    print_info(&format!("Fetching repository: {}", url));

    let response = reqwest::blocking::get(&url)
    .with_context(|| format!("Failed to fetch repository: {}", url))?;

    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Repository returned status: {}",
            response.status()
        ));
    }

    let index: Value = response
    .json()
    .with_context(|| "Failed to parse repository index")?;

    let mut packages = HashMap::new();
    let clean_repo_url = repo_url.trim_end_matches('/');
    if let Some(pkgs) = index.get("packages").and_then(Value::as_object) {
        for (name, info) in pkgs {
            packages.insert(
                name.clone(),
                            PackageInfo {
                                name: name.clone(),
                            version: info["version"].as_str().unwrap_or("unknown").to_string(),
                            description: info["description"].as_str().unwrap_or("No description").to_string(),
                            url: info["url"].as_str().unwrap_or("").to_string(),
                            os: info["os"].as_str().unwrap_or("all").to_string(),
                            arch: info["arch"].as_str().unwrap_or("any").to_string(),
                            deps: info["deps"].as_array()
                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                            .unwrap_or_default(),
                            author: info["author"].as_str().unwrap_or("unknown").to_string(),
                            license: info["license"].as_str().unwrap_or("unknown").to_string(),
                            },
            );
        }
    }

    print_success(&format!("Found {} packages", packages.len()));
    Ok(packages)
}

pub fn search(query: &str) -> Result<()> {
    let repos = crate::config::get_repos()?;
    let mut results = Vec::new();

    for repo in repos {
        match fetch_repository(&repo) {
            Ok(packages) => {
                for (_, pkg) in packages {
                    if pkg.name.contains(query) || pkg.description.contains(query) {
                        results.push(pkg);
                    }
                }
            }
            Err(e) => print_error(&format!("Error fetching repo {}: {}", repo_name, e)),
        }
    }

    if results.is_empty() {
        println!("No packages found matching '{}'", query);
        return Ok(());
    }

    println!("{:<20} {:<10} {:<10} {:<20}", "Package", "Version", "OS", "Description");
    println!("{:-<20} {:-<10} {:-<10} {:-<20}", "", "", "", "");

    for pkg in results {
        println!(
            "{:<20} {:<10} {:<10} {}",
            pkg.name.green().bold(),
                 pkg.version,
                 pkg.os,
                 pkg.description
        );
    }

    Ok(())
}

pub fn find_package(pkg_name: &str) -> Result<PackageInfo> {
    let repos = get_repos()?;

    for (_name, repo_config) in repos.iter() {
        verify_repository(&repo_config.url)?;
        let packages = fetch_repository(&repo_config.url)?;
        if let Some(pkg) = packages.get(pkg_name) {
            return Ok(pkg.clone());
        }
    }

    Err(anyhow::anyhow!("Package '{}' not found in any repository", pkg_name))
}

pub fn repo_add(url: &str, name: Option<&str>, key_url: Option<&str>) -> Result<()> {
    let repo_name = name.unwrap_or_else(|| {
        url.split('/').nth(2).unwrap_or("unknown")
    });

    let mut config = load_repos_config()?;
    if config.contains_key(repo_name) {
        return Err(anyhow::anyhow!("Repository '{}' already exists", repo_name));
    }

    verify_repository(url)?;

    config.insert(repo_name.to_string(), RepoConfig {
        url: url.to_string(),
                  gpg_key: key_url.map(|s| s.to_string()),
    });

    save_repos_config(&config)?;
    print_success(&format!("Added repository '{}'", repo_name));
    Ok(())
}

pub fn repo_remove(name: &str) -> Result<()> {
    let mut config = load_repos_config()?;
    if config.remove(name).is_none() {
        return Err(anyhow::anyhow!("Repository '{}' not found", name));
    }
    save_repos_config(&config)?;
    crate::cli::print_success(&format!("Removed repository '{}'", name));
    Ok(())
}

pub fn repo_list() -> Result<()> {
    let config = load_repos_config()?;
    if config.is_empty() {
        println!("No repositories configured.");
        return Ok(());
    }

    println!("{:<20} {:<40}", "NAME", "URL");
    println!("{:-<20} {:-<40}", "", "");
    for (name, repo) in config {
        println!("{:<20} {:<40}", name.blue().bold(), repo.url);
    }
    Ok(())
}

pub fn load_repos_config() -> Result<HashMap<String, RepoConfig>> {
    let path = get_config_path(REPOS_CONFIG)?;
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content)?)
}

fn save_repos_config(config: &HashMap<String, RepoConfig>) -> Result<()> {
    let path = get_config_path(REPOS_CONFIG)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(config)?)?;
    Ok(())
}

fn get_config_path(filename: &str) -> Result<PathBuf> {
    Ok(dirs::config_dir()
    .ok_or_else(|| anyhow::anyhow!("Config directory not found"))?
    .join("anspm")
    .join(filename))
}

pub fn repo_update_keys() -> Result<()> {
    use std::process::Command;

    let repos = load_repos_config()?;

    if repos.is_empty() {
        print_info("No repositories configured - nothing to update");
        return Ok(());
    }

    for (name, repo) in repos {
        print_info(&format!("Updating keys for repository: {}", name));

        if let Some(key_url) = repo.gpg_key {
            let _key = download_file(&key_url)?;

            let status = Command::new("gpg")
            .arg("--import")
            .stdin(std::process::Stdio::piped())
            .status()?;

            if !status.success() {
                print_error(&format!("Failed to import key for repository {}", name));
                continue;
            }

            print_success(&format!("Successfully updated keys for {}", name));
        } else {
            print_info(&format!("Repository {} has no GPG key configured", name));
        }
    }

    print_success("All repository keys updated successfully");
    Ok(())
}

pub fn repo_verify(name: &str) -> Result<()> {
    let config = load_repos_config()?;
    let repo = config.get(name)
    .ok_or_else(|| anyhow::anyhow!("Repository '{}' not found", name))?;

    verify_repo_index(&repo.url)?;
    print_success(&format!("Repository '{}' verified successfully", name));
    Ok(())
}

fn download_file(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!("Failed to download file: {}", url));
    }
    Ok(response.bytes()?.to_vec())
}

fn verify_repository(url: &str) -> Result<()> {
    verify_repo_index(url)
}
