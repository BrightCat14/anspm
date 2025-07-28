use crate::cli::{print_error, print_info, print_success};
use crate::config::{read_tracking_file, write_tracking_file};
use crate::repo;
use anyhow::{Result};
use colored::Colorize;
use serde_json::json;
use std::fs;
use std::path::{Path};
use std::process::Command;
use crate::repo::PackageInfo;
use crate::config::get_cache_dir;

fn update_package_db(pkg_name: &str, pkg: &PackageInfo, files: &[String]) -> Result<()> {
    let mut db = read_tracking_file()?;
    db[pkg_name] = json!({
        "version": pkg.version,
        "files": files,
        "meta": {
            "name": pkg.name,
            "version": pkg.version,
            "arch": pkg.arch,
            "os": pkg.os,
            "deps": pkg.deps,
            "author": pkg.author,
            "license": pkg.license,
            "description": pkg.description
        }
    });
    write_tracking_file(&db)
}

fn download_pkg_with_cache(pkg_name: &str, url: &str) -> Result<Vec<u8>> {
    let cache_dir = get_cache_dir()?;
    let cached_path = cache_dir.join(format!(
        "{}-{}",
        pkg_name,
        url.split('/').last().unwrap_or("pkg.pkg")
    ));

    if cached_path.exists() {
        print_info(&format!("Using cached package: {}", cached_path.display()));
        return Ok(fs::read(&cached_path)?);
    }

    print_info(&format!("Downloading package: {}", url));
    let pkg_data = reqwest::blocking::get(url)?
    .bytes()?
    .to_vec();

    fs::write(&cached_path, &pkg_data)?;
    print_info(&format!("Cached package at: {}", cached_path.display()));

    Ok(pkg_data)
}

pub fn install(pkg_name: &str, check: bool) -> Result<()> {
    print_info(&format!("Installing package: {}", pkg_name));

    let pkg = repo::find_package(pkg_name)?;
    if check {
        if let Ok(db) = read_tracking_file() {
            if let Some(installed_pkg) = db.get(pkg_name) {
                let installed_ver = installed_pkg["version"].as_str().unwrap_or("");

                if installed_ver == pkg.version {
                    print_info(&format!(
                        "Package {} v{} is already installed. Use anspm reinstall {} to reinstall.",
                        pkg_name, pkg.version, pkg_name
                    ));
                    return Ok(());
                } else if *installed_ver > *pkg.version {
                    print_info(&format!(
                        "Newer version ({}) is already installed. Downgrading to {} requires anspm reinstall {}.",
                                        installed_ver, pkg.version, pkg_name
                    ));
                    return Ok(());
                } else {
                    print_info(&format!(
                        "Package {} is installed ({}). New version {} available.\nRun `anspm update {}` to update.",
                                        pkg_name, installed_ver, pkg.version, pkg_name
                    ));
                    return Ok(());
                }
            }
        }
    }

    if pkg.os != "all" && pkg.os != std::env::consts::OS {
        return Err(anyhow::anyhow!(
            "Package '{}' is for {} (your OS is {})",
                                   pkg_name,
                                   pkg.os,
                                   std::env::consts::OS
        ));
    }

    print_info(&format!("Starting use package from: {}", pkg.url));
    let pkg_data = download_pkg_with_cache(pkg_name, &pkg.url)?;

    let temp_dir = tempfile::tempdir()?;
    let pkg_path = temp_dir.path().join(format!("{}.pkg", pkg_name));
    fs::write(&pkg_path, &pkg_data)?;

    print_info("Installing files to system...");
    let status = Command::new("sudo")
    .args(&["tar", "-xzf", pkg_path.to_str().unwrap(), "-C", "/"])
    .status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("Failed to install package files"));
    }

    let output = Command::new("tar")
    .args(&["-tzf", pkg_path.to_str().unwrap()])
    .output()?;

    let installed_files: Vec<String> = String::from_utf8(output.stdout)?
    .lines()
    .map(|s| format!("/{}", s.trim()))
    .collect();

    if installed_files.iter().all(|f| !Path::new(f).exists()) {
        return Err(anyhow::anyhow!("No package files were installed"));
    }

    update_package_db(pkg_name, &pkg, &installed_files)?;

    print_success(&format!(
        "Package {} v{} installed successfully!",
        pkg_name, pkg.version
    ));
    Ok(())
}

pub fn clean_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;
    if cache_dir.exists() {
        fs::remove_dir_all(&cache_dir)?;
        fs::create_dir(&cache_dir)?;
    }
    print_success("Cache cleared successfully!");
    Ok(())
}

pub fn remove(pkg_name: &str) -> Result<()> {
    print_info(&format!("Removing package: {}", pkg_name));

    let mut db = read_tracking_file()?;
    if !db.as_object().unwrap().contains_key(pkg_name) {
        print_error(&format!("Package {} not found!", pkg_name));
        return Ok(());
    }

    if let Some(files) = db[pkg_name]["files"].as_array() {
        let mut paths: Vec<&str> = files.iter().filter_map(|v| v.as_str()).collect();
        paths.sort_by(|a, b| b.cmp(a));

        for path in paths {
            let path = Path::new(path);
            if path.exists() {
                if path.is_file() {
                    fs::remove_file(path).ok();
                } else if path.is_dir() {
                    fs::remove_dir(path).ok();
                }
            }
        }
    }

    db.as_object_mut().unwrap().remove(pkg_name);
    write_tracking_file(&db)?;

    print_success(&format!("Package {} removed successfully!", pkg_name));
    Ok(())
}

pub fn update(only: Option<&str>) -> Result<()> {
    let db = read_tracking_file()?;
    let _repos = repo::load_repos_config()?;

    if let Some(packages) = db.as_object() {
        for (pkg_name, pkg_info) in packages {
            if let Some(only_pkg) = only {
                if pkg_name != only_pkg {
                    continue;
                }
            }

            if let Some(installed_version) = pkg_info["version"].as_str() {
                if let Ok(latest_pkg) = repo::find_package(pkg_name) {
                    if *latest_pkg.version > *installed_version {
                        println!(
                            "Update available for {}: {} -> {}.",
                            pkg_name.green().bold(),
                                installed_version,
                                latest_pkg.version
                        );
                        println!("Updating {} to {}...", pkg_name, latest_pkg.version);
                        install(pkg_name, false)?;
                    }
                }
            }
        }
    }

    Ok(())
}

pub fn reinstall(pkg_name: &str) -> Result<()> {
    remove(pkg_name)?;
    install(pkg_name, false)
}

pub fn list() -> Result<()> {
    print_info("Listing installed packages:");

    let db = read_tracking_file()?;
    if db.as_object().unwrap().is_empty() {
        println!("No packages installed.");
        return Ok(());
    }

    println!("{:<20} {:<10} {:<10} {:<20}", "Package", "Version", "OS", "Description");
    println!("{:-<20} {:-<10} {:-<10} {:-<20}", "", "", "", "");

    for (name, info) in db.as_object().unwrap() {
        let meta = &info["meta"];
        println!(
            "{:<20} {:<10} {:<10} {}",
            name.green().bold(),
                 meta["version"].as_str().unwrap_or("unknown"),
                 meta["os"].as_str().unwrap_or("unknown"),
                 meta["description"].as_str().unwrap_or("No description")
        );
    }

    Ok(())
}

pub fn _verify(pkg_name: &str) -> Result<()> {
    let db = read_tracking_file()?;
    if let Some(pkg) = db.get(pkg_name) {
        let files = pkg["files"].as_array().unwrap();
        let mut missing = Vec::new();

        for file in files {
            let path = file.as_str().unwrap();
            if !Path::new(path).exists() {
                missing.push(path);
            }
        }

        if missing.is_empty() {
            print_success(&format!("Package {} is properly installed", pkg_name));
        } else {
            print_error(&format!("Missing files: {:?}", missing));
        }
    } else {
        print_error(&format!("Package {} not found", pkg_name));
    }
    Ok(())
}
