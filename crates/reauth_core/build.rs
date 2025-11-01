use std::process::Command;
use std::path::Path;

fn main() {
    // Only build UI if the "embed-ui" feature is enabled
    if std::env::var("CARGO_FEATURE_EMBED_UI").is_ok() {
        let ui_path = Path::new("../../ui");

        println!("cargo:warning=Running npm ci and npm run build for UI...");

        // Cross-platform npm executable
        let npm = if cfg!(target_os = "windows") { "npm.cmd" } else { "npm" };

        // Always install dependencies
        let status_ci = Command::new(npm)
            .current_dir(&ui_path)
            .args(["ci"])
            .status()
            .expect("Failed to run `npm ci`");

        if !status_ci.success() {
            panic!("❌ npm ci failed. Run it manually in the ui folder.");
        }

        // Always run build
        let status_build = Command::new(npm)
            .current_dir(&ui_path)
            .args(["run", "build"])
            .status()
            .expect("Failed to run `npm run build`");

        if !status_build.success() {
            panic!("❌ npm run build failed. Run it manually in the ui folder.");
        }

        println!("cargo:warning=✅ UI built successfully!");

        // Re-run build.rs if UI source files change (optional, for Cargo caching)
        println!("cargo:rerun-if-changed=../ui/src/");
    }
}
