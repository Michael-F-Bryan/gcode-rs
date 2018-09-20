extern crate tempfile;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

#[test]
fn the_example_compiles() {
    let (_temp, _exe) = compile_example();
}

#[test]
fn run_the_example() {
    let (_temp, exe) = compile_example();

    let output = Command::new(exe).stdout(Stdio::piped()).output().unwrap();
    assert!(output.status.success());

    let stdout = String::from_utf8(output.stdout).unwrap();

    panic!("{:?}", stdout);
}

fn executable_exists(exe: &str) -> bool {
    Command::new(exe)
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn compile_example() -> (TempDir, PathBuf) {
    // We need to do some annoying path wrangling to get things running both
    // locally and on CI
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let example = crate_root.join("example.c");
    let target_dir = crate_root.parent().unwrap().join("target");
    let mut lib_dir = target_dir.clone();
    if let Ok(target) = env::var("TARGET") {
        lib_dir.push(target);
    }
    lib_dir.push(if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    });

    let temp =
        tempfile::tempdir().expect("Unable to create a temporary directory");
    let mut executable = temp.path().join("main");

    if cfg!(target_os = "windows") {
        executable.set_extension("exe");
    }

    let status = Command::new(get_compiler())
        .arg("-Wall")
        .arg(&example)
        .arg("-o")
        .arg(&executable)
        .arg("-I")
        .arg(&target_dir)
        .arg("-L")
        .arg(&lib_dir)
        .arg("-lgcode_ffi")
        .status()
        .expect("Compilation failed");

    assert!(status.success(), "{:?}", status);

    (temp, executable)
}

fn get_compiler() -> String {
    if let Ok(cc) = env::var("CC") {
        return cc;
    }
    let compilers = &["gcc", "clang"];
    for compiler in compilers {
        if executable_exists(compiler) {
            return compiler.to_string();
        }
    }

    panic!("Unable to determine the compiler");
}
