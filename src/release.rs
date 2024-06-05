use std::{error::Error, fs::File, io::Write, path::Path};
use semver::Version;
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Release {
    tag_name: String,
    pub assets: Vec<Asset>,
}

#[derive(Deserialize)]
pub struct Asset {
    pub browser_download_url: String,
}

pub fn get_latest_release(repo: &str) -> Result<Release, Box<dyn Error>> {
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let client = Client::new();
    let response = client.get(url)
        .header("User-Agent", "request")
        .send()?
        .json::<Release>()?;
    
    Ok(response)
}

pub fn download_latest_release(url: &str, output_path: &Path) -> Result<(), Box<dyn Error>> {
    let response = reqwest::blocking::get(url)?;
    let mut file = File::create(output_path)?;
    let content = response.bytes()?;
    file.write_all(&content)?;
    Ok(())
}

fn is_version_newer(version: &str) -> Result<bool, Box<dyn Error>> {
    let trimmed_version = adjust_version(version);
    let compare_version = Version::parse(&trimmed_version)?;

    let current_trimmed_version = adjust_version(env!("APP_VERSION"));
    let current_version = Version::parse(&current_trimmed_version)?;

    Ok(compare_version > current_version)
}

pub fn is_update_available(release: &Release) -> Result<bool, Box<dyn Error>> {
    let is_newer = is_version_newer(&release.tag_name);
    is_newer
}

fn adjust_version(version: &str) -> String {
    let trimmed_version = version.trim_start_matches('v');
    let dot_count = trimmed_version.matches('.').count();

    match dot_count {
        1 => format!("{}.0", trimmed_version),
        0 => format!("{}.0.0", trimmed_version),
        _ => trimmed_version.to_string(),
    }
}