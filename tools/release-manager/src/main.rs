use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use reqwest::header::{AUTHORIZATION, USER_AGENT, CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::{Client, Body};
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tokio_util::codec::{BytesCodec, FramedRead};

mod release_logic;
use release_logic::{ReleaseManager, ReleaseRecord};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the ISO file to upload
    #[arg(long)]
    iso: PathBuf,

    /// GitHub repository (owner/repo)
    #[arg(long)]
    repo: String,

    /// Path to the releases.json file
    #[arg(long, default_value = "releases.json")]
    history: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let token = std::env::var("GITHUB_TOKEN").context("GITHUB_TOKEN environment variable not set")?;

    let client = Client::new();

    // 1. Load History
    let history_records = load_history(&args.history).unwrap_or_else(|_| Vec::new());
    println!("Loaded {} historical releases.", history_records.len());

    let mut manager = ReleaseManager::new(history_records);

    // 2. Prepare New Release
    let now = Utc::now();
    let tag_name = format!("release-{}", now.format("%Y%m%d-%H%M"));
    let start_time = std::time::Instant::now();
    println!("Preparing release: {}", tag_name);

    // 3. Create Release on GitHub
    let release_id = create_github_release(&client, &args.repo, &token, &tag_name, &args.iso).await?;
    println!("Created release on GitHub with ID: {}", release_id);
    println!("Upload finished in {:.2?}", start_time.elapsed());

    let new_record = ReleaseRecord {
        tag_name: tag_name.clone(),
        created_at: now,
        release_id,
    };

    // 4. Calculate Retention Policy via ReleaseManager
    let to_delete = manager.push_release(new_record);
    let kept = manager.get_releases();

    println!("Retention Policy Result:");
    println!("  Keeping: {} releases", kept.len());
    for r in kept {
        println!("    - {} ({})", r.tag_name, r.created_at);
    }
    println!("  Deleting: {} releases", to_delete.len());
    for r in &to_delete {
        println!("    - {} ({})", r.tag_name, r.created_at);
    }

    // 5. Delete Old Releases from GitHub
    for release in to_delete {
        println!("Deleting old release: {}", release.tag_name);
        if let Err(e) = delete_github_release(&client, &args.repo, &token, release.release_id, &release.tag_name).await {
            eprintln!("Failed to delete release {}: {}", release.tag_name, e);
        }
    }

    // 6. Save Updated History
    // We need to convert the slice/vec from manager back to what save_history expects or just pass the slice
    save_history(&args.history, kept)?;
    println!("Updated {} successfully.", args.history.display());

    Ok(())
}

fn load_history(path: &Path) -> Result<Vec<ReleaseRecord>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let file = std::fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let history: Vec<ReleaseRecord> = serde_json::from_reader(reader)?;
    Ok(history)
}

fn save_history(path: &Path, history: &[ReleaseRecord]) -> Result<()> {
    let file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, history)?;
    Ok(())
}

async fn create_github_release(
    client: &Client,
    repo: &str,
    token: &str,
    tag_name: &str,
    iso_path: &Path,
) -> Result<u64> {
    let url = format!("https://api.github.com/repos/{}/releases", repo);

    // 1. Create Release
    let create_body = serde_json::json!({
        "tag_name": tag_name,
        "name": tag_name,
        "body": format!("Automated release for NixOS ISO. Created at {}", Utc::now()),
        "draft": false,
        "prerelease": false
    });

    let resp = client
        .post(&url)
        .header(USER_AGENT, "release-manager")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .json(&create_body)
        .send()
        .await?
        .error_for_status()?;

    let release_json: serde_json::Value = resp.json().await?;
    let release_id = release_json["id"].as_u64().context("Failed to get release ID")?;
    let upload_url_template = release_json["upload_url"]
        .as_str()
        .context("Failed to get upload_url")?;
    let upload_url = upload_url_template.split('{').next().unwrap();

    // 2. Upload Asset (Streaming)
    let file = File::open(iso_path).await.with_context(|| format!("Failed to open ISO file at {:?}", iso_path))?;
    let metadata = file.metadata().await?;
    let file_len = metadata.len();
    let file_name = iso_path.file_name().unwrap().to_string_lossy();
    
    let upload_query_url = format!("{}?name={}", upload_url, file_name);

    println!("Starting stream upload for {} ({} bytes)", file_name, file_len);

    // Create a stream from the file
    let stream = FramedRead::new(file, BytesCodec::new());
    let body = Body::wrap_stream(stream);

    let resp_upload = client
        .post(&upload_query_url)
        .header(USER_AGENT, "release-manager")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(CONTENT_TYPE, "application/octet-stream")
        .header(CONTENT_LENGTH, file_len) // Important for progress/server to know size
        .body(body)
        .send()
        .await?
        .error_for_status()?;

    println!("Asset uploaded: {}", resp_upload.status());

    Ok(release_id)
}

async fn delete_github_release(client: &Client, repo: &str, token: &str, release_id: u64, tag_name: &str) -> Result<()> {
    // 1. Delete Release
    let url = format!("https://api.github.com/repos/{}/releases/{}", repo, release_id);
    client
        .delete(&url)
        .header(USER_AGENT, "release-manager")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?
        .error_for_status()?;

    // 2. Delete Tag
    let tag_url = format!("https://api.github.com/repos/{}/git/refs/tags/{}", repo, tag_name);
    let resp = client
        .delete(&tag_url)
        .header(USER_AGENT, "release-manager")
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .send()
        .await?;
    
    if !resp.status().is_success() {
        eprintln!("Warning: Failed to delete tag ref {}: {}", tag_name, resp.status());
    }

    Ok(())
}
