extern crate embed_resource;
use std::{env, process::Command};
use regex::Regex;

fn main() {
    embed_resource::compile("app.rc", embed_resource::NONE);
    slint_build::compile("ui/confirmdialog.slint").unwrap();
    slint_build::compile("ui/messagebox.slint").unwrap();
    slint_build::compile("ui/settingswindow.slint").unwrap();
    slint_build::compile("ui/kioskwindow.slint").unwrap();
    slint_build::compile("ui/main.slint").unwrap();

    let semver_regex = Regex::new(r"^v\d+(\.\d+)*$").unwrap();

    // Reading the GITHUB_REF_NAME environment variable set by GitHub Actions
    let version = match env::var("GITHUB_REF_NAME") {
        Ok(ref_name) if semver_regex.is_match(&ref_name) => ref_name,
        _ => {
            // Execute the Git command to determine the latest tag
            let command_output = Command::new("git")
                .args(&["describe", "--tags", "--abbrev=0"])
                .output();

            match command_output {
                Ok(output) if output.status.success() => String::from_utf8(output.stdout).unwrap().trim().to_string(),
                _ => "0.0.0".to_string()
            }
        }
    };
    println!("cargo:rustc-env=APP_VERSION={}", version);
}