mod adoptium;
mod conf;
mod download;
mod extract;

use std::{env};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;
use tempfile::{tempdir};

fn main() {
    if env::args().len() < 2 {
        eprintln!("Arguments missing.");
        print_usage_and_exit()
    }

    // Get command
    let command = &env::args().nth(1).unwrap();
    match command.as_str() {
        "init" => {
            init();
        }
        "use" => {
            juse();
        }
        "env" => {
            env();
        }
        "sing" => {
            println!("There are no Easter Eggs in this program. Trust me. 💃");
        }
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage_and_exit()
        }
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!("Usage: jlo [ env | init | use ]");
    exit(1);
}

fn init() {
    conf::init_config();
}

fn juse() {
    if env::args().len() < 3 {
        eprintln!("Missing version argument.");
        print_usage_and_exit()
    }

    let version = &env::args().nth(2).unwrap();
    
    if !conf::is_valid_version(version) {
        eprintln!("Unsupported version: '{}'.", version);
        exit(1);
    }
    
    setup(version);
}

fn env() {
    let java_version = conf::load().unwrap_or_else(|e| {
        eprintln!("Error: Could not load configuration: {}", e);
        exit(1);
    });

    setup(&java_version);
}

fn setup(java_version: &String) {
    eprintln!("Setup environment for Java {}", java_version);

    let jdk_base = jlo_home_dir().join("jdks");

    let java_home = find_suitable_jdk(&jdk_base, &java_version)
        .unwrap_or_else(|| {
            install_jdk(&jdk_base, &java_version)
        });

    eprintln!("Using JAVA_HOME: {}", java_home.to_str().unwrap());
    println!("export JAVA_HOME=\"{}\"", java_home.to_str().unwrap());

    let java_bin_path: String = java_home.join("bin").to_str().unwrap().into();
    update_path(&java_bin_path).map(|updated_path| {
        println!("export PATH=\"{}\"", updated_path);
    });
}

fn find_suitable_jdk(jdk_base: &Path, required_version: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(jdk_base).ok()?;

    let mut matching_versions: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_dir() && path.file_name()
                .and_then(|name| name.to_str())
                .map_or(false, |name| name.starts_with(required_version))
        })
        .collect();

    // Sort by directory name in descending order
    matching_versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    matching_versions.into_iter().next()
}

fn install_jdk(jdk_base: &Path, java_version: &String) -> PathBuf {
    // Find JDK metadata
    let jdk_metadata = adoptium::fetch_metadata(java_version);

    // Download JDK
    let temp_dir = tempdir().unwrap();
    let temp_file = temp_dir.path().join(&jdk_metadata.package_name);
    let file = &mut File::create(&temp_file).unwrap();
    let artifact_description = format!(
        "JDK {} ({})",
        jdk_metadata.semver, jdk_metadata.package_name
    );
    download::download(
        artifact_description.as_str(),
        &jdk_metadata.download_link,
        &jdk_metadata.checksum,
        file,
    )
        .unwrap();

    // Extract JDK to temp dir
    extract::extract(
        &temp_file,
        &temp_dir.path(),
    )
        .unwrap();

    let dest_dir = jdk_base.join(&jdk_metadata.semver);
    adoptium::install_jdk(jdk_metadata, &temp_dir.path(), dest_dir.as_path());

    temp_dir.close().unwrap_or_else(|err| {
        eprintln!("Warning: Could not delete temporary directory: {}", err);
    });

    dest_dir
}

fn jlo_home_dir() -> PathBuf {
    match env::var_os("JLO_HOME") {
        Some(jlo_home) => PathBuf::from(jlo_home),
        None => match env::home_dir() {
            Some(home) => home.join(".jlo"),
            None => {
                eprintln!("Error: Could not determine home directory.");
                exit(1);
            }
        },
    }
}

fn update_path(java_path: &str) -> Option<String> {
    let current_path = env::var("PATH").unwrap_or_default();

    // Remove any existing J'Lo paths to avoid duplicates
    let jlo_base = jlo_home_dir();
    let mut path_vector: Vec<_> = env::split_paths(&current_path)
        .filter(|p| !p.starts_with(&jlo_base))
        .collect();

    // Insert the new path at the beginning
    path_vector.insert(0, java_path.into());

    // Join paths back into a single string
    let new_path = env::join_paths(path_vector)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Only return if the path has changed
    if new_path == current_path {
        None
    } else {
        Some(new_path)
    }
}
