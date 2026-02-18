use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap_or_default());
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());

    let dest_dir = find_profile_dir(&out_dir, &profile).unwrap_or_else(|| out_dir.clone());
    let dest_path = dest_dir.join("reauth.toml");

    let template_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/reauth.toml.template");

    println!("cargo:rerun-if-changed={}", template_path.display());

    if dest_path.exists() {
        return;
    }

    match fs::read_to_string(&template_path) {
        Ok(contents) => {
            if let Err(err) = fs::write(&dest_path, contents) {
                eprintln!("Failed to write config template: {}", err);
            }
        }
        Err(err) => {
            eprintln!("Failed to read config template: {}", err);
        }
    }
}

fn find_profile_dir(out_dir: &Path, profile: &str) -> Option<PathBuf> {
    let profile_os = OsStr::new(profile);
    for ancestor in out_dir.ancestors() {
        if ancestor.file_name() == Some(profile_os) {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}
