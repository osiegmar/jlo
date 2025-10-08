use semver_rs::compare;
use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;

pub fn clean_jdks(jdk_base: &Path) {
    // collector major versions
    let mut installed_jdks: std::collections::HashMap<i64, Vec<PathBuf>> = std::collections::HashMap::new();
    let entries = match std::fs::read_dir(jdk_base) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("Error reading JDK base directory {:?}: {}", jdk_base, e);
            exit(1);
        }
    };

    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            eprintln!("{:?} is not a directory", path);
            continue;
        }
        let file_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name,
            None => {
                eprintln!("Skipping directory with invalid name: {:?}", path);
                continue
            },
        };
        let semver = match semver_rs::parse(file_name, None) {
            Ok(sv) => sv,
            Err(_) => {
                eprintln!("Skipping non-semver directory: {:?}", path);
                continue
            },
        };
        installed_jdks.entry(semver.major).or_default().push(path);
    }

    for (major, mut paths) in installed_jdks {
        paths.sort_by(|a, b| {
            let a_str = a.file_name().and_then(|name| name.to_str()).unwrap_or("");
            let b_str = b.file_name().and_then(|name| name.to_str()).unwrap_or("");
            compare(b_str, a_str, None).unwrap()
        });

        if paths.len() <= 1 {
            continue;
        }

        let kept = paths[0].file_name().unwrap().to_str().unwrap();
        let removed = paths[1..]
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect::<Vec<_>>()
            .join(", ");

        eprintln!(
            "Keeping {} for JDK {}, but removing: {}",
            kept, major, removed
        );

        for old_jdk in &paths[1..] {
            if let Err(e) = std::fs::remove_dir_all(old_jdk) {
                eprintln!("Error removing old JDK {:?}: {}", old_jdk, e);
            }
        }
    }
}

pub fn find_suitable_jdk(jdk_base: &Path, required_version: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(jdk_base).ok()?;

    let mut matching_versions: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map_or(false, |name| name.starts_with(required_version))
        })
        .collect();

    matching_versions.sort_by(|a, b| {
        let a_str = a.file_name().and_then(|name| name.to_str()).unwrap_or("");
        let b_str = b.file_name().and_then(|name| name.to_str()).unwrap_or("");

        compare(b_str, a_str, None).unwrap()
    });

    matching_versions.first().cloned()
}

pub fn fetch_metadata(java_version: &String) -> JdkMetadata {
    let api_url = format!(
        "https://api.adoptium.net/v3/assets/latest/{java_version}/hotspot?architecture={arch}&image_type=jdk&os={os}&vendor=eclipse",
        java_version = java_version,
        arch = jdk_arch(),
        os = jdk_os()
    );

    let metadata_response = reqwest::blocking::get(&api_url);

    match metadata_response {
        Ok(response) => {
            if !response.status().is_success() {
                eprintln!(
                    "Error: Failed to fetch metadata from API: HTTP {}",
                    response.status()
                );
                exit(1);
            }
            let json: serde_json::Value = match response.json() {
                Ok(j) => j,
                Err(e) => {
                    eprintln!("Error: Failed to parse JSON response: {}", e);
                    exit(1);
                }
            };

            let json_array = json.as_array().unwrap_or_else(|| {
                eprintln!("Error: Unexpected JSON structure received from API.");
                exit(1);
            });

            if json_array.len() == 0 {
                eprintln!(
                    "Error: No matching JDK found for the specified version and system architecture."
                );
                eprintln!("Tried to fetch metadata from: {}", api_url);
                exit(1);
            }

            let root_node = json_array.first().unwrap();

            let semver = root_node["version"]["semver"].as_str().unwrap_or("");
            let release_name = root_node["release_name"].as_str().unwrap_or("");
            let package_name = root_node["binary"]["package"]["name"]
                .as_str()
                .unwrap_or("");
            let download_link = root_node["binary"]["package"]["link"]
                .as_str()
                .unwrap_or("");
            let checksum = root_node["binary"]["package"]["checksum"]
                .as_str()
                .unwrap_or("");
            if semver.is_empty()
                || release_name.is_empty()
                || package_name.is_empty()
                || download_link.is_empty()
                || checksum.is_empty()
            {
                eprintln!("Error: Incomplete metadata received from API.");
                exit(1);
            }
            JdkMetadata {
                semver: semver.to_string(),
                release_name: release_name.to_string(),
                package_name: package_name.to_string(),
                download_link: download_link.to_string(),
                checksum: checksum.to_string(),
            }
        }
        Err(e) => {
            eprintln!("Error: Could not fetch metadata from API: {}", e);
            exit(1);
        }
    }
}

pub fn install_jdk(jdk_metadata: &JdkMetadata, source_dir: &Path, dest_dir: &Path) {
    // Validate extracted path
    let extracted_jdk_path = find_jdk_path(&jdk_metadata, &source_dir).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        exit(1);
    });

    // Move extracted JDK to final location
    eprintln!("Installing JDK to {:?}", dest_dir);
    std::fs::create_dir_all(dest_dir.parent().unwrap()).unwrap();
    std::fs::rename(extracted_jdk_path, dest_dir).unwrap();
}

fn find_jdk_path(jdk_metadata: &JdkMetadata, temp_dest: &Path) -> Result<PathBuf, String> {
    let mut extracted_jdk_path = temp_dest.join(&jdk_metadata.release_name);

    // On macOS, the JDK is inside Contents/Home
    if env::consts::OS == "macos" {
        extracted_jdk_path = extracted_jdk_path.join("Contents").join("Home");
    }

    if env::consts::OS == "windows" {
        let java_bin = extracted_jdk_path.join("bin").join("java.exe");
        if !java_bin.exists() {
            return Err(format!(
                "Error: java executable is missing at: {:?}",
                java_bin
            ));
        }
    } else {
        let java_bin = extracted_jdk_path.join("bin").join("java");
        if !java_bin.exists() {
            return Err(format!(
                "Error: java executable is missing at: {:?}",
                java_bin
            ));
        }
    }

    Ok(extracted_jdk_path)
}

pub fn find_installed_jdk(jdk_metadata: &JdkMetadata, jdk_base_path: &Path) -> Option<PathBuf> {
    let extracted_jdk_path = jdk_base_path.join(&jdk_metadata.semver);
    match extracted_jdk_path.exists() {
        true => Some(extracted_jdk_path),
        false => None,
    }
}

pub struct JdkMetadata {
    pub semver: String,
    pub release_name: String,
    pub package_name: String,
    pub download_link: String,
    pub checksum: String,
}

fn jdk_os() -> &'static str {
    match env::consts::OS {
        "linux" | "windows" | "solaris" | "aix" => env::consts::OS,
        "macos" => "mac",
        _ => panic!("Unknown OS: {}", env::consts::OS),
    }
}

fn jdk_arch() -> &'static str {
    match env::consts::ARCH {
        "x86_64" => "x64",
        "x86" => "x32",
        "powerpc64" => {
            if cfg!(target_endian = "little") {
                "ppc64le"
            } else {
                "ppc64"
            }
        }
        "s390x" | "arm" | "aarch64" => env::consts::ARCH,
        "sparc64" => "sparcv9",
        "riscv64" => "riscv64",
        _ => panic!("Unknown ARCH: {}", env::consts::ARCH),
    }
}

pub fn find_latest_jdk() -> Result<String, String> {
    let response = reqwest::blocking::get("https://api.adoptium.net/v3/info/available_releases");
    
    match response {
        Ok(releases) => {
            let json: serde_json::Value = releases.json().map_err(|e| format!("Failed to parse JSON response: {}", e))?;
            let available_releases = json["available_releases"].as_array().ok_or("Unexpected JSON structure received from API.")?;
            
            let latest = match available_releases.iter()
                .filter_map(|v| v.as_i64())
                .max() {
                    Some(v) => v,
                    None => return Err("No available releases found.".to_string()),
                };
            
            Ok(latest.to_string())
        }
        Err(e) => Err(format!("Could not fetch available releases from API: {}", e)),
    }
}