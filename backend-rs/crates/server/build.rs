use std::process::Command;

fn main() {
    let now = chrono::Utc::now().to_rfc3339();
    println!("cargo:rustc-env=GE_BUILD_TIME={now}");

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

    println!("cargo:rerun-if-changed=build.rs");
}
