use std::process::Command;

fn main() {
    let build_time =
        std::env::var("GE_BUILD_TIME").unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());
    println!("cargo:rustc-env=GE_BUILD_TIME={build_time}");

    let rustc_release = rustc_release_line();
    let rustc_version = rustc_release.as_deref().unwrap_or("unknown").to_string();
    println!("cargo:rustc-env=GE_RUSTC_VERSION={rustc_version}");

    let rust_channel = rustc_release
        .as_deref()
        .map(detect_channel)
        .unwrap_or("unknown");
    println!("cargo:rustc-env=GE_RUST_CHANNEL={rust_channel}");

    let edition = rust_edition().unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GE_RUST_EDITION={edition}");

    let git_commit = env_or("GE_GIT_COMMIT", || {
        run_git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into())
    });
    let git_branch = env_or("GE_GIT_BRANCH", || {
        run_git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into())
    });
    println!("cargo:rustc-env=GE_GIT_COMMIT={git_commit}");
    println!("cargo:rustc-env=GE_GIT_BRANCH={git_branch}");

    for var in ["GE_BUILD_TIME", "GE_GIT_COMMIT", "GE_GIT_BRANCH"] {
        println!("cargo:rerun-if-env-changed={var}");
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    println!("cargo:rerun-if-changed=../../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../../.git/refs");
}

fn rust_edition() -> Option<String> {
    let manifest = std::fs::read_to_string("../../Cargo.toml").ok()?;
    manifest.lines().find_map(|line| {
        let trimmed = line.trim();
        let rest = trimmed.strip_prefix("edition")?.trim_start();
        let value = rest.strip_prefix('=')?.trim();
        Some(value.trim_matches('"').to_string())
    })
}

fn rustc_release_line() -> Option<String> {
    let output = Command::new("rustc").arg("-vV").output().ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    stdout
        .lines()
        .find_map(|l| l.strip_prefix("release: ").map(str::to_string))
}

fn detect_channel(release: &str) -> &'static str {
    if release.contains("nightly") {
        "nightly"
    } else if release.contains("beta") {
        "beta"
    } else {
        "stable"
    }
}

fn env_or<F: FnOnce() -> String>(key: &str, fallback: F) -> String {
    std::env::var(key)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(fallback)
}

fn run_git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}
