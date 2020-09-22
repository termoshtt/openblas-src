use std::{env, process::Command};

fn feature_enabled(feature: &str) -> bool {
    env::var(format!("CARGO_FEATURE_{}", feature.to_uppercase())).is_ok()
}

/// Add path where pacman (on msys2) install OpenBLAS
///
/// - `pacman -S mingw-w64-x86_64-openblas` will install
///   - `libopenbla.dll` into `/mingw64/bin`
///   - `libopenbla.a`   into `/mingw64/lib`
/// - But we have to specify them using `-L` in **Windows manner**
///   - msys2 `/` is `C:\msys64\` in Windows by default install
///   - It can be convert using `cygpath` command
fn windows_gnu_system() {
    let lib_path = String::from_utf8(
        Command::new("cygpath")
            .arg("-w")
            .arg(if feature_enabled("static") {
                "/mingw64/bin"
            } else {
                "/mingw64/lib"
            })
            .output()
            .expect("Failed to exec cygpath")
            .stdout,
    )
    .expect("cygpath output includes non UTF-8 string");
    println!("cargo:rustc-link-search={}", lib_path);
}

/// Use vcpkg for msvc "system" feature
fn windows_msvc_system() {
    if feature_enabled("static") {
        env::set_var("CARGO_CFG_TARGET_FEATURE", "crt-static");
    } else {
        env::set_var("VCPKGRS_DYNAMIC", "1");
    }
    #[cfg(target_env = "msvc")]
    vcpkg::find_package("openblas").unwrap();
    if !cfg!(target_env = "msvc") {
        unreachable!();
    }
}

/// homebrew says
///
/// > openblas is keg-only, which means it was not symlinked into /usr/local,
/// > because macOS provides BLAS in Accelerate.framework.
/// > For compilers to find openblas you may need to set:
///
/// ```text
/// export LDFLAGS="-L/usr/local/opt/openblas/lib"
/// export CPPFLAGS="-I/usr/local/opt/openblas/include"
/// ```
fn macos_system() {
    println!("cargo:rustc-link-search=/usr/local/opt/openblas/lib");
}

fn main() {
    let link_kind = if feature_enabled("static") {
        "static"
    } else {
        "dylib"
    };
    if feature_enabled("system") {
        if cfg!(target_os = "windows") {
            if cfg!(target_env = "gnu") {
                windows_gnu_system();
            } else if cfg!(target_env = "msvc") {
                windows_msvc_system();
            } else {
                panic!(
                    "Unsupported ABI for Windows: {}",
                    env::var("CARGO_CFG_TARGET_ENV").unwrap()
                );
            }
        }
        if cfg!(target_os = "macos") {
            macos_system();
        }
        println!("cargo:rustc-link-lib={}=openblas", link_kind);
    } else {
        if cfg!(target_env = "msvc") {
            panic!(
                "Non-vcpkg builds are not supported on Windows. You must use the 'system' feature."
            )
        }
    }
}
