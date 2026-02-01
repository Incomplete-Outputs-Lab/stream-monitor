use std::process::Command;

fn main() {
    // Git commit hashを取得
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if output.status.success() {
            let commit_hash = String::from_utf8_lossy(&output.stdout);
            let commit_hash = commit_hash.trim();
            println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_hash);
        }
    }

    // ビルド日時を取得
    let build_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("cargo:rustc-env=BUILD_DATE={}", build_date);

    tauri_build::build()
}
