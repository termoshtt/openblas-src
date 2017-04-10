use std::env;
use std::path::PathBuf;
use std::process::Command;

macro_rules! binary(() => (if cfg!(target_pointer_width = "32") { "32" } else { "64" }));
macro_rules! feature(($name:expr) => (env::var(concat!("CARGO_FEATURE_", $name)).is_ok()));
macro_rules! switch(($condition:expr) => (if $condition { "YES" } else { "NO" }));
macro_rules! variable(($name:expr) => (env::var($name).unwrap()));

fn main() {
    let kind = if feature!("STATIC") { "static" } else { "dylib" };
    if !feature!("SYSTEM") {
        let cblas = feature!("CBLAS");
        let lapacke = feature!("LAPACKE");
        let source = PathBuf::from("source");
        let output = PathBuf::from(variable!("OUT_DIR").replace(r"\", "/"));
        env::remove_var("TARGET");
        run(Command::new("make")
                    .args(&["libs", "netlib", "shared"])
                    .arg(format!("BINARY={}", binary!()))
                    .arg(format!("{}_CBLAS=1", switch!(cblas)))
                    .arg(format!("{}_LAPACKE=1", switch!(lapacke)))
                    .arg(format!("-j{}", variable!("NUM_JOBS")))
                    .current_dir(&source));
        run(Command::new("make")
                    .arg("install")
                    .arg(format!("DESTDIR={}", output.display()))
                    .current_dir(&source));
        println!("cargo:rustc-link-search={}", output.join("opt/OpenBLAS/lib").display());
    }
    println!("cargo:rustc-link-lib={}=openblas", kind);
    println!("cargo:rustc-link-lib=dylib=irc");
    println!("cargo:rustc-link-lib=dylib=ifcore");
    println!("cargo:rustc-link-lib=dylib=svml");
}

fn run(command: &mut Command) {
    println!("Running: `{:?}`", command);
    match command.status() {
        Ok(status) => if !status.success() {
            panic!("Failed: `{:?}` ({})", command, status);
        },
        Err(error) => {
            panic!("Failed: `{:?}` ({})", command, error);
        },
    }
}
