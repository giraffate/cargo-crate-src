use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

use futures::{stream, StreamExt};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    let root_path = env::current_dir().unwrap();
    let toml_path = root_path.join("Cargo.toml");

    if !toml_path.exists() {
        println!("crate-src: Failed to find Cargo.toml");
        return;
    }

    let out = env::var("CRATESRC_CLONE_ROOT")
        .unwrap()
        .parse::<PathBuf>()
        .unwrap();

    let mut buf = String::new();
    File::open(toml_path)
        .unwrap()
        .read_to_string(&mut buf)
        .unwrap();
    let config: Config = toml::from_str(&buf).unwrap();

    let client = Client::builder()
        .user_agent("cargo-crate-src")
        .build()
        .unwrap();

    let bodies = stream::iter(config.dependencies.keys())
        .map(|name| {
            let client = &client;
            println!("crate-src: Fetching {} from crates.io ...", name);
            async move {
                let resp = client
                    .get(format!("https://crates.io/api/v1/crates/{}", name))
                    .send()
                    .await?;
                match resp.error_for_status() {
                    Ok(resp) => resp.json::<ApiResponse>().await,
                    Err(err) => Err(err),
                }
            }
        })
        .buffer_unordered(2);

    bodies
        .for_each(|resp| async {
            match resp {
                Ok(resp) => {
                    let repo_url = resp.krate.repository;
                    let repo = repo_url.split('/').last().unwrap();
                    if !is_github_url(&repo_url) {
                        println!(
                            "crate-src: Skipped to clone {} because repository isn't in github.com",
                            repo
                        );
                        return;
                    }
                    if out.join(repo).exists() {
                        println!(
                            "crate-src: Skipped to clone {} because it's already existed",
                            repo
                        );
                        return;
                    }

                    println!("crate-src: Cloning from {} ...", &repo_url);
                    Command::new("git")
                        .args(&["clone", &format!("{}.git", &repo_url)])
                        .current_dir(&out)
                        .output()
                        .unwrap();
                }
                Err(e) => eprintln!("crate-src: Got an error: {}", e),
            }
        })
        .await;
}

fn is_github_url(url: &str) -> bool {
    let re = Regex::new(r"^https://github\.com/.+/.+").unwrap();
    re.is_match(url)
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    dependencies: HashMap<String, toml::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    #[serde(rename = "crate")]
    krate: Krate,
}

#[derive(Debug, Serialize, Deserialize)]
struct Krate {
    repository: String,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_github_url() {
        assert!(is_github_url(
            "https://github.com/giraffate/cargo-crate-src"
        ));

        assert!(!is_github_url(
            "https://gitlab.giraffate.com/utils/rust-gitlab"
        ))
    }
}
