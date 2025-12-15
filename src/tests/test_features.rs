use std::{
    collections::VecDeque,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
    thread,
};

fn cargo_exe() -> String {
    env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn sarge_manifest_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

fn sarge_target_dir_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("feature-matrix-unit")
}

fn cargo_check_self(no_default: bool, features: &[&str], target_dir: &Path) -> Result<(), String> {
    fs::create_dir_all(target_dir).map_err(|e| format!("create target dir failed: {e}"))?;

    let mut cmd = Command::new(cargo_exe());
    cmd.arg("check")
        .arg("--all-targets")
        .arg("--offline")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(sarge_manifest_path())
        .env("CARGO_TARGET_DIR", target_dir)
        .env("RUSTFLAGS", "-D warnings");

    if no_default {
        cmd.arg("--no-default-features");
    }

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }

    let output = cmd
        .output()
        .map_err(|e| format!("spawn cargo check failed: {e}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!(
            "cargo check failed for no_default={no_default}, features={features:?}\nstdout:\n{stdout}\n\nstderr:\n{stderr}"
        ))
    }
}

#[test]
fn feature_matrix_compiles() {
    #[derive(Clone, Debug)]
    struct Case {
        no_default: bool,
        features: Vec<&'static str>,
    }

    let cases: Vec<Case> = vec![
        // No features at all.
        Case {
            no_default: true,
            features: vec![],
        },
        // Default = help + macros.
        Case {
            no_default: false,
            features: vec![],
        },
        // Single features.
        Case {
            no_default: true,
            features: vec!["help"],
        },
        Case {
            no_default: true,
            features: vec!["macros"],
        },
        // Both features explicitly enabled.
        Case {
            no_default: true,
            features: vec!["help", "macros"],
        },
    ];

    // Concurrency: FEATURE_MATRIX_JOBS overrides, otherwise cap at 4.
    let jobs = env::var("FEATURE_MATRIX_JOBS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| {
            thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(4)
                .min(4)
        });
    let jobs = jobs.clamp(1, cases.len().max(1));

    let queue = Arc::new(Mutex::new(VecDeque::from(cases)));
    let failures: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    thread::scope(|s| {
        for worker_idx in 0..jobs {
            let queue = Arc::clone(&queue);
            let failures = Arc::clone(&failures);
            let worker_target_dir = sarge_target_dir_base().join(format!("worker-{worker_idx}"));

            s.spawn(move || loop {
                let case = { queue.lock().unwrap().pop_front() };
                let Some(case) = case else { break };

                let label = format!(
                    "no_default={}, features={:?}",
                    case.no_default, case.features
                );
                println!("running feature matrix case: {label}");

                if let Err(err) =
                    cargo_check_self(case.no_default, &case.features, &worker_target_dir)
                {
                    failures.lock().unwrap().push(format!("{label}: {err}"));
                }
            });
        }
    });

    let failures = failures.lock().unwrap();
    assert!(
        failures.is_empty(),
        "feature matrix failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time should be after unix epoch")
        .as_nanos();
    dir.push(format!("sarge_{prefix}_{nanos}"));
    dir
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("failed to create parent directories");
    }
    fs::write(path, contents).expect("failed to write file");
}

fn cargo_check_downstream(project_dir: &Path, target_dir: &Path) -> std::process::Output {
    Command::new(cargo_exe())
        .arg("check")
        .arg("--quiet")
        .arg("--offline")
        .env("CARGO_TARGET_DIR", target_dir)
        .current_dir(project_dir)
        .output()
        .expect("failed to run cargo check")
}

fn cargo_run_downstream(project_dir: &Path, target_dir: &Path) -> std::process::Output {
    Command::new(cargo_exe())
        .arg("run")
        .arg("--quiet")
        .arg("--offline")
        .env("CARGO_TARGET_DIR", target_dir)
        .current_dir(project_dir)
        .output()
        .expect("failed to run cargo run")
}

fn cleanup_project_dir(dir: &Path) {
    if env::var_os("SARGE_KEEP_TEST_PROJECTS").is_some() {
        return;
    }
    let _ = fs::remove_dir_all(dir);
}

fn write_cargo_toml(project_dir: &Path, name: &str, sarge_path: &Path) {
    write_file(
        &project_dir.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{name}"
version = "0.0.0"
edition = "2021"

[dependencies]
sarge = {{ path = "{path}" }}
"#,
            name = name,
            path = sarge_path.display()
        ),
    );
}

fn write_cargo_toml_no_help(project_dir: &Path, name: &str, sarge_path: &Path) {
    write_file(
        &project_dir.join("Cargo.toml"),
        &format!(
            r#"[package]
name = "{name}"
version = "0.0.0"
edition = "2021"

[dependencies]
sarge = {{ path = "{path}", default-features = false, features = ["macros"] }}
"#,
            name = name,
            path = sarge_path.display()
        ),
    );
}

#[test]
fn downstream_without_help_feature_can_call_args_help() {
    let project_dir = unique_temp_dir("downstream_help_smoke");
    let target_dir = project_dir.join("target");
    let sarge_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    write_cargo_toml(&project_dir, "sarge_downstream_help_smoke", &sarge_path);

    write_file(
        &project_dir.join("src/main.rs"),
        r"use sarge::prelude::*;

sarge! {
    /// Program docs
    Args,

    /// Print help
    #err 'h' help: bool = true,
}

fn main() {
    let _ = Args::help();
}
",
    );

    let out = cargo_check_downstream(&project_dir, &target_dir);
    assert!(
        out.status.success(),
        "cargo check failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    cleanup_project_dir(&project_dir);
}

#[test]
fn legacy_gt_doc_syntax_is_a_hard_error_in_downstream() {
    let project_dir = unique_temp_dir("downstream_legacy_gt");
    let target_dir = project_dir.join("target");
    let sarge_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    write_cargo_toml(&project_dir, "sarge_downstream_legacy_gt", &sarge_path);

    write_file(
        &project_dir.join("src/main.rs"),
        r#"use sarge::prelude::*;

sarge! {
    > "nope"
    Args,
    flag: bool,
}

fn main() {}
"#,
    );

    let out = cargo_check_downstream(&project_dir, &target_dir);
    assert!(
        !out.status.success(),
        "expected cargo check to fail, but it succeeded:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("legacy `>` doc syntax is not supported"),
        "stderr did not contain expected message:\n{stderr}"
    );

    cleanup_project_dir(&project_dir);
}

#[test]
fn downstream_no_help_feature_returns_placeholder_help_text() {
    let project_dir = unique_temp_dir("downstream_no_help_help_text");
    let target_dir = project_dir.join("target");
    let sarge_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    write_cargo_toml_no_help(
        &project_dir,
        "sarge_downstream_no_help_help_text",
        &sarge_path,
    );

    write_file(
        &project_dir.join("src/main.rs"),
        r#"use sarge::prelude::*;

sarge! {
    Args,
    #err 'h' help: bool = true,
}

fn main() {
    print!("{}", Args::help());
}
"#,
    );

    let out = cargo_run_downstream(&project_dir, &target_dir);
    assert!(
        out.status.success(),
        "cargo run failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("help is disabled"),
        "stdout did not contain expected placeholder text:\n{stdout}"
    );

    cleanup_project_dir(&project_dir);
}
