use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../migrations");
    println!("cargo:rerun-if-changed=../../seeds");
    println!("cargo:rerun-if-changed=../../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../../.git/refs");
    for var in [
        "GE_VERSION",
        "GE_GIT_COMMIT",
        "GE_GIT_BRANCH",
        "GE_GIT_REPOSITORY",
        "GE_BUILD_TIME",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let version = env_or("GE_VERSION", || {
        git(&["describe", "--tags", "--always", "--dirty"]).unwrap_or_else(|| "dev".into())
    });
    let commit = env_or("GE_GIT_COMMIT", || {
        git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into())
    });
    let branch = env_or("GE_GIT_BRANCH", || {
        git(&["rev-parse", "--abbrev-ref", "HEAD"]).unwrap_or_else(|| "unknown".into())
    });
    let repository = env_or("GE_GIT_REPOSITORY", || {
        std::env::var("CARGO_PKG_REPOSITORY").unwrap_or_else(|_| "unknown".into())
    });
    // Only emit GE_BUILD_TIME when explicitly set (e.g. CI). Auto-emitting
    // would change every run and invalidate the build cache on every rebuild.
    let build_time = std::env::var("GE_BUILD_TIME").unwrap_or_else(|_| "unknown".into());

    println!("cargo:rustc-env=GE_VERSION={version}");
    println!("cargo:rustc-env=GE_GIT_COMMIT={commit}");
    println!("cargo:rustc-env=GE_GIT_BRANCH={branch}");
    println!("cargo:rustc-env=GE_GIT_REPOSITORY={repository}");
    println!("cargo:rustc-env=GE_BUILD_TIME={build_time}");
}

fn env_or<F: FnOnce() -> String>(key: &str, fallback: F) -> String {
    std::env::var(key)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(fallback)
}

fn git(args: &[&str]) -> Option<String> {
    run("git", args)
}

fn run(cmd: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(cmd).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!s.is_empty()).then_some(s)
}
