use std::fs::OpenOptions;
use std::io::Write;

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

pub fn init_config(latest_release: String) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(".jlorc")
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                "File '.jlorc' already exists!".to_string()
            } else {
                e.to_string()
            }
        })?;

    write!(file, "{}\n", latest_release)
        .map_err(|e| e.to_string())?;

    println!("Created config file '.jlorc' with Java {}", latest_release);
    Ok(())
}

pub fn is_valid_version(version: &str) -> bool {
    if let Ok(ver) = version.parse::<u32>() {
        ver >= 8
    } else {
        false
    }
}
