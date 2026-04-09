use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const TRAINEDDATA_ENV_KEYS: [&str; 4] = [
    "OCR_TRAINEDDATA_PATH",
    "TESSDATA_PATH",
    "OCR_TRAINEDDATA_FILE",
    "TESSDATA_FILE",
];
const DEFAULT_TRAINEDDATA_PATH: &str = "tessdata/eng.traineddata";

fn main() {
    configure_embedded_traineddata();
    tauri_build::build()
}

fn configure_embedded_traineddata() {
    for key in TRAINEDDATA_ENV_KEYS {
        println!("cargo:rerun-if-env-changed={key}");
    }
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=../.env");
    println!("cargo:rerun-if-changed=tessdata");

    let configured = configured_traineddata_path();
    let source_path = resolve_traineddata_source_path(&configured);

    if !source_path.is_file() {
        panic!(
            "Configured traineddata file does not exist: {}",
            source_path.display()
        );
    }

    let normalized_path = source_path.canonicalize().unwrap_or(source_path);
    let file_name = normalized_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("eng.traineddata");

    println!(
        "cargo:rustc-env=OCR_EMBEDDED_TRAINEDDATA_PATH={}",
        normalized_path.display()
    );
    println!("cargo:rustc-env=OCR_EMBEDDED_TRAINEDDATA_FILENAME={file_name}");
}

fn configured_traineddata_path() -> String {
    first_non_empty_env_value().unwrap_or_else(|| {
        first_non_empty_dotenv_value().unwrap_or_else(|| DEFAULT_TRAINEDDATA_PATH.to_string())
    })
}

fn first_non_empty_env_value() -> Option<String> {
    TRAINEDDATA_ENV_KEYS
        .iter()
        .find_map(|key| env::var(key).ok().filter(|value| !value.trim().is_empty()))
}

fn first_non_empty_dotenv_value() -> Option<String> {
    dotenv_candidates().into_iter().find_map(|path| {
        let contents = fs::read_to_string(path).ok()?;
        parse_dotenv_value(&contents)
    })
}

fn dotenv_candidates() -> [PathBuf; 2] {
    [PathBuf::from(".env"), PathBuf::from("..").join(".env")]
}

fn parse_dotenv_value(contents: &str) -> Option<String> {
    for line in contents.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let line = line.strip_prefix("export ").unwrap_or(line);
        let Some((key, raw_value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        if !TRAINEDDATA_ENV_KEYS.contains(&key) {
            continue;
        }

        let value = raw_value.trim();
        let value = value
            .strip_prefix('"')
            .and_then(|v| v.strip_suffix('"'))
            .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
            .unwrap_or(value)
            .trim();

        if !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

fn resolve_traineddata_source_path(configured: &str) -> PathBuf {
    let normalized = configured.trim().replace('\\', "/");
    let candidate = PathBuf::from(&normalized);

    if candidate.is_absolute() {
        return candidate;
    }

    if normalized.contains('/') {
        if candidate.exists() {
            return candidate;
        }

        let workspace_relative = Path::new("..").join(&candidate);
        if workspace_relative.exists() {
            return workspace_relative;
        }

        candidate
    } else {
        let local = Path::new("tessdata").join(&normalized);
        if local.exists() {
            return local;
        }

        let workspace_relative = Path::new("..").join(&local);
        if workspace_relative.exists() {
            return workspace_relative;
        }

        local
    }
}
