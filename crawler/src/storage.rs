// storage.rs

use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::path::{Path, PathBuf};
use url::Url;
use sha2::{Sha256, Digest};

pub async fn save_content(url: &str, content: &str) -> Result<(), std::io::Error> {
    let file_name = url_to_file_name(url);
    let path = Path::new("crawled_pages").join(file_name);

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut file = File::create(&path).await?;
    file.write_all(content.as_bytes()).await?;

    // Write metadata file
    let metadata_path = path.with_extension("meta");
    let mut metadata_file = File::create(metadata_path).await?;
    let metadata = format!("URL: {}\nTimestamp: {}", url, chrono::Utc::now());
    metadata_file.write_all(metadata.as_bytes()).await?;

    Ok(())
}

fn url_to_file_name(url: &str) -> PathBuf {
    let parsed_url = Url::parse(url).unwrap_or_else(|_| Url::parse("http://invalid.url").unwrap());

    let host = parsed_url.host_str().unwrap_or("unknown_host");
    let path = parsed_url.path().trim_start_matches('/').replace('/', "_");
    let query = parsed_url.query().unwrap_or("").replace('&', "_");

    let mut hasher = Sha256::new();
    hasher.update(url.as_bytes());
    let hash = hasher.finalize();
    let hash_hex = hex::encode(&hash);  // Convert the hash to a hexadecimal string

    let file_name = format!(
        "{}__{}__{}__{}",
        host,
        path.chars().take(50).collect::<String>(),
        query.chars().take(50).collect::<String>(),
        &hash_hex[..16]  // Use the first 16 characters of the hash
    );

    PathBuf::from(file_name).with_extension("html")
}