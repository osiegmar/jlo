use std::fs::OpenOptions;
use std::io::Write;
use std::process::exit;

pub fn load() -> Result<String, String> {
    let java_version = match std::fs::read_to_string(".jlorc") {
        Ok(content) => content.trim().to_string(),
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                return Err("To initialize a new config file, run: `jlo init` first.".to_string())
            }
            return Err(format!("Could not read '.jlorc' file: {}", e));
        }
    };

    if java_version.is_empty() {
        return Err("File '.jlorc' is empty. Please specify a Java version.".to_string())
    }

    if !is_valid_version(&java_version) {
        return Err(format!("Unsupported Java version specified in '.jlorc': '{}'.", java_version));
    }

    Ok(java_version.to_string())
}

pub fn init_config() {
    match OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(".jlorc")
    {
        Ok(mut file) => {
            match write!(file, "25\n") {
                Err(e) => {
                    eprintln!("Error: Could not write to file '.jlorc': {}", e);
                    exit(1);
                }
                Ok(..) => {
                    println!("Created config file '.jlorc' with default Java version 25");
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            eprintln!("Error: File '.jlorc' already exists!");
            exit(1);
        }
        Err(e) => {
            eprintln!("Error: Could not create file '.jlorc': {}", e);
            exit(1);
        }
    }
}

pub fn is_valid_version(version: &str) -> bool {
    // TODO call/cache https://api.adoptium.net/v3/info/available_releases ?

    if let Ok(ver) = version.parse::<u32>() {
        return ver >= 8
    }

    false
}
