use std::process::Command;

fn main() {
    // Get the current year at build time using chrono
    let year = chrono::Local::now().format("%Y").to_string();
    println!("cargo:rustc-env=BUILD_YEAR={}", year);

    // Get the build timestamp in ISO 8601 format yyyy-MM-ddTHH:mm:ss
    let build_date = chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    // Get the short git commit hash
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| {
            if output.status.success() {
                String::from_utf8(output.stdout).ok()
            } else {
                None
            }
        })
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
