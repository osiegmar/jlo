mod adoptium;
mod conf;
mod download;
mod extract;

use std::{env};
use std::fs::File;
use std::path::{PathBuf};
use std::process::exit;
use tempfile::{tempdir};

fn main() {
    // TODO support Windows when possible
    if !cfg!(unix) {
        eprintln!("Unsupported OS: {}", env::consts::OS);
        exit(1);
    }

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
    eprintln!("Usage: jlo.sh [ init | env ]");
    exit(1);
}

fn init() {
    conf::init_config();
}

fn env() {
    let java_version = conf::load().unwrap_or_else(|e| {
        eprintln!("Error: Could not load configuration: {}", e);
        exit(1);
    });

    eprintln!("Setup environment for Java {}", java_version);

    let java_home = jlo_home_dir().join("jdks").join(java_version);

    if !java_home.exists() {
        install_jdk(&java_home);
    }

    println!("export JAVA_HOME=\"{}\"", java_home.to_str().unwrap());

    let java_bin_path: String = java_home.join("bin").to_str().unwrap().into();
    update_path(&java_bin_path).map(|updated_path| {
        println!("export PATH=\"{}\"", updated_path);
    });
}

fn install_jdk(java_home: &PathBuf) {
    // Find JDK metadata
    let jdk_metadata = adoptium::fetch_metadata();

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

    adoptium::install_jdk(jdk_metadata, &temp_dir.path(), java_home);

    temp_dir.close().unwrap_or_else(|err| {
        eprintln!("Warning: Could not delete temporary directory: {}", err);
    })
}

fn jlo_home_dir() -> PathBuf {
    match env::var_os("JLO_HOME") {
        Some(jlo_home) => PathBuf::from(jlo_home),
        None => match env::home_dir() {
            Some(home) => home.join(".jlo.sh"),
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
