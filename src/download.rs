use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use sha2::{Digest, Sha256};

pub fn download(
    name: &str,
    url: &str,
    expected_checksum: &str,
    file: &mut File,
) -> Result<(), Box<dyn Error>> {
    let client = Client::builder().timeout(Duration::from_secs(60)).build()?;

    let response = client.get(url).send()?;

    let total_size = response
        .content_length()
        .ok_or("Failed to get content length")?;

    let mut source = response;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{msg}\n[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );
    pb.set_message(format!("Downloading {} ...", name));

    let mut hasher = Sha256::new();

    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192];
    while let Ok(n) = source.read(&mut buffer) {
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        downloaded += n as u64;
        pb.set_position(downloaded);
        hasher.update(&buffer[..n]);
    }

    pb.finish_and_clear();

    let hash = hex::encode(hasher.finalize());
    if hash != expected_checksum {
        return Err(format!(
            "Checksum mismatch: expected {}, got {}.",
            expected_checksum, hash
        )
            .into());
    }

    eprintln!("✅ Download complete, checksum passed.");

    Ok(())
}
