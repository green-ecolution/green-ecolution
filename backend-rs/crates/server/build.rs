use std::process::Command;

fn main() {
    let build_time =
        std::env::var("GE_BUILD_TIME").unwrap_or_else(|_| chrono::Utc::now().to_rfc3339());
    println!("cargo:rustc-env=GE_BUILD_TIME={build_time}");

    let rustc = Command::new("rustc")
        .arg("-vV")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| {
            s.lines()
                .find_map(|l| l.strip_prefix("release: ").map(str::to_string))
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GE_RUSTC_VERSION={rustc}");

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
    println!("cargo:rerun-if-changed=../../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../../.git/refs");
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
