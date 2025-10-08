mod adoptium;
mod conf;
mod download;
mod extract;

use crate::adoptium::{clean_jdks, fetch_metadata, find_installed_jdk, find_suitable_jdk, JdkMetadata};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::exit;
use tempfile::tempdir;

fn main() {
    if env::args().len() < 2 {
        eprintln!("Arguments missing.");
        print_usage_and_exit()
    }

    // Get command
    let command = &env::args().nth(1).unwrap();
    match command.as_str() {
        "env" => {
            env();
        }
        "clean" => {
            clean();
        }
        "init" => {
            init();
        }
        "update" => {
            update();
        }
        "use" => {
            juse();
        }
        "selfupdate" => {
            eprintln!("Self-update is handled by the jlo shell function.");
            exit(1);
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
    eprintln!("Usage: jlo [ env | clean | init | update | use | selfupdate ]");
    exit(1);
}

fn init() {
    conf::init_config().unwrap_or_else(|e| {
        eprintln!("Error: Could not create config file: {}", e);
        exit(1);
    });
}

fn clean() {
    let jdk_base = jlo_home_dir().join("jdks");
    clean_jdks(&jdk_base);
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

fn cmd_line_version() -> Option<String> {
    if env::args().len() <= 2 {
        return None
    }

    let java_version = &env::args().nth(2).unwrap();

    if !conf::is_valid_version(java_version) {
        eprintln!("Unsupported version: '{}'.", java_version);
        exit(1);
    }

    Some(java_version.to_string())
}

fn update() {
    let java_version = cmd_line_version()
        .or_else(|| conf::load().ok())
        .unwrap_or_else(|| {
            eprintln!("Error: Could not load configuration and no version specified.");
            exit(1);
        });

    let jdk_metadata = fetch_metadata(&java_version);
    let jdk_base = jlo_home_dir().join("jdks");

    if let Some(path) = find_installed_jdk(&jdk_metadata, &jdk_base) {
        eprintln!(
            "Most recent version of JDK {} is already installed at: {}",
            java_version,
            path.to_str().unwrap()
        );
    } else {
        install_jdk(&jdk_base, &jdk_metadata);
    }
}

fn env() {
    let java_version = conf::load().unwrap_or_else(|e| {
        eprintln!("Error: Could not load configuration: {}", e);
        exit(1);
    });

    setup(&java_version);
}

fn setup(java_version: &String) {
    let jdk_base = jlo_home_dir().join("jdks");

    let java_home = find_suitable_jdk(&jdk_base, java_version)
        .unwrap_or_else(|| {
            // Find JDK metadata
            let jdk_metadata = fetch_metadata(java_version);
            install_jdk(&jdk_base, &jdk_metadata)
        });

    let mut updates = false;

    let current_java_home = env::var("JAVA_HOME").unwrap_or_default();
    if current_java_home != java_home.to_string_lossy() {
        updates = true;
        println!("export JAVA_HOME=\"{}\"", java_home.to_string_lossy());
    }

    let java_bin_path = java_home.join("bin").to_string_lossy().into_owned();
    if let Some(updated_path) = update_path(&java_bin_path) {
        updates = true;
        println!("export PATH=\"{}\"", updated_path);
    }

    if updates {
        eprintln!("Use Java from {}", java_home.to_string_lossy());
    }
}

fn install_jdk(jdk_base: &Path, jdk_metadata: &JdkMetadata) -> PathBuf {
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
    extract::extract(&temp_file, &temp_dir.path()).unwrap();

    let dest_dir = jdk_base.join(&jdk_metadata.semver);
    adoptium::install_jdk(&jdk_metadata, &temp_dir.path(), dest_dir.as_path());

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
