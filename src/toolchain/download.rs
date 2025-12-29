use crate::error::{CargoJamError, Result};
use crate::toolchain::config::ToolchainConfig;
use crate::toolchain::platform::Platform;
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use tar::Archive;

const GITHUB_API_URL: &str = "https://api.github.com/repos/paritytech/polkajam-releases/releases";

#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub published_at: Option<String>,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Fetch available releases from GitHub
pub fn fetch_releases(limit: usize) -> Result<Vec<GitHubRelease>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("cargo-jam")
        .build()
        .map_err(|e| CargoJamError::Git(format!("Failed to create HTTP client: {}", e)))?;

    let url = format!("{}?per_page={}", GITHUB_API_URL, limit);
    let mut request = client.get(&url);

    // Use GITHUB_TOKEN if available (for CI environments with rate limits)
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .map_err(|e| CargoJamError::Git(format!("Failed to fetch releases: {}", e)))?;

    if !response.status().is_success() {
        return Err(CargoJamError::Git(format!(
            "GitHub API returned status: {}",
            response.status()
        )));
    }

    let releases: Vec<GitHubRelease> = response
        .json()
        .map_err(|e| CargoJamError::Git(format!("Failed to parse releases: {}", e)))?;

    Ok(releases)
}

/// Get the latest nightly release
pub fn get_latest_release() -> Result<GitHubRelease> {
    let releases = fetch_releases(10)?;
    releases
        .into_iter()
        .find(|r| r.tag_name.starts_with("nightly"))
        .ok_or_else(|| CargoJamError::Git("No nightly releases found".to_string()))
}

/// Get a specific release by version
pub fn get_release(version: &str) -> Result<GitHubRelease> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("cargo-jam")
        .build()
        .map_err(|e| CargoJamError::Git(format!("Failed to create HTTP client: {}", e)))?;

    let url = format!("{}/tags/{}", GITHUB_API_URL, version);
    let mut request = client.get(&url);

    // Use GITHUB_TOKEN if available (for CI environments with rate limits)
    if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .map_err(|e| CargoJamError::Git(format!("Failed to fetch release {}: {}", version, e)))?;

    if !response.status().is_success() {
        return Err(CargoJamError::Git(format!(
            "Release '{}' not found (status: {})",
            version,
            response.status()
        )));
    }

    let release: GitHubRelease = response
        .json()
        .map_err(|e| CargoJamError::Git(format!("Failed to parse release: {}", e)))?;

    Ok(release)
}

/// Download and install a release
pub fn download_and_install(
    release: &GitHubRelease,
    platform: &Platform,
    force: bool,
) -> Result<PathBuf> {
    let mut config = ToolchainConfig::load()?;

    // Check if already installed
    if !force && config.is_installed() {
        if let Some(ref installed) = config.installed_version {
            if installed == &release.tag_name {
                return Err(CargoJamError::Git(format!(
                    "Version '{}' is already installed. Use --force to reinstall.",
                    release.tag_name
                )));
            }
        }
    }

    // Find the asset for this platform
    let asset = release
        .assets
        .iter()
        .find(|a| a.name.contains(platform.asset_suffix()))
        .ok_or_else(|| {
            CargoJamError::Git(format!(
                "No asset found for platform '{}' in release '{}'. Available assets: {}",
                platform,
                release.tag_name,
                release
                    .assets
                    .iter()
                    .map(|a| a.name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })?;

    // Create toolchain directory
    let toolchain_dir = ToolchainConfig::toolchain_dir()?;
    std::fs::create_dir_all(&toolchain_dir)?;

    // Download the archive
    let download_url = &asset.browser_download_url;
    let archive_path = toolchain_dir.join(&asset.name);

    download_file(download_url, &archive_path)?;

    // Remove old installation if it exists
    let normalized_dir = toolchain_dir.join("polkajam-nightly");
    if normalized_dir.exists() {
        std::fs::remove_dir_all(&normalized_dir)?;
    }

    // Extract the archive
    let extract_dir = toolchain_dir.clone();
    extract_archive(&archive_path, &extract_dir, platform)?;

    // Clean up the archive
    std::fs::remove_file(&archive_path)?;

    // Normalize the extracted directory name to polkajam-nightly
    normalize_extracted_dir(&toolchain_dir)?;

    // Update config
    config.set_installed(&release.tag_name, toolchain_dir.clone());
    config.save()?;

    Ok(toolchain_dir)
}

/// Normalize the extracted directory name to polkajam-nightly
fn normalize_extracted_dir(toolchain_dir: &PathBuf) -> Result<()> {
    let normalized_name = "polkajam-nightly";
    let normalized_path = toolchain_dir.join(normalized_name);

    // Find any directory starting with "polkajam-" that isn't already normalized
    if let Ok(entries) = std::fs::read_dir(toolchain_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().unwrap().to_string_lossy();
                if name.starts_with("polkajam-") && name != normalized_name {
                    // Rename to normalized name
                    std::fs::rename(&path, &normalized_path)?;
                    return Ok(());
                }
            }
        }
    }

    Ok(())
}

/// Download a file with progress indication
fn download_file(url: &str, dest: &PathBuf) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("cargo-jam")
        .build()
        .map_err(|e| CargoJamError::Git(format!("Failed to create HTTP client: {}", e)))?;

    let mut response = client
        .get(url)
        .send()
        .map_err(|e| CargoJamError::Git(format!("Failed to download: {}", e)))?;

    if !response.status().is_success() {
        return Err(CargoJamError::Git(format!(
            "Download failed with status: {}",
            response.status()
        )));
    }

    let mut file = File::create(dest)?;
    io::copy(&mut response, &mut file)?;

    Ok(())
}

/// Extract an archive (tar.gz or zip)
fn extract_archive(archive_path: &PathBuf, dest: &PathBuf, platform: &Platform) -> Result<()> {
    match platform.archive_extension() {
        "tar.gz" => extract_tar_gz(archive_path, dest),
        "zip" => extract_zip(archive_path, dest),
        ext => Err(CargoJamError::Git(format!(
            "Unknown archive extension: {}",
            ext
        ))),
    }
}

fn extract_tar_gz(archive_path: &PathBuf, dest: &PathBuf) -> Result<()> {
    let file = File::open(archive_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive.unpack(dest)?;
    Ok(())
}

fn extract_zip(archive_path: &Path, dest: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)
        .map_err(|e| CargoJamError::Git(format!("Failed to open zip archive: {}", e)))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| CargoJamError::Git(format!("Failed to read zip entry: {}", e)))?;

        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}
