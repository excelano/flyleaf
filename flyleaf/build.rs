//! Embed the Windows application manifest, and do nothing else ever.
//!
//! Copied from slipcase-desktop's build script, whose comment records why
//! this route and not a resource compiler: `CLAUDE.md` says nothing compiles
//! C and a build needs a Rust toolchain and nothing else, and this prints two
//! linker arguments so that the linker already linking the binary embeds
//! `packaging/windows/flyleaf.manifest`. The manifest declares DPI awareness
//! before any of the program's code runs, and the Windows App Certification
//! Kit reads the manifest rather than the process.
//!
//! **A second use for this file is a decision, not a precedent.**
//!
//! Author: David M. Anderson
//! Built with AI assistance (Claude, Anthropic)

#![forbid(unsafe_code)]

use std::path::Path;

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("packaging")
        .join("windows")
        .join("flyleaf.manifest");

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", manifest.display());

    // Read from the environment rather than from `cfg!`, because a build script
    // is compiled for the host and `cfg!(windows)` in here answers about the
    // machine doing the building; this repository cross-checks the Windows
    // target from Linux.
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os != "windows" || env != "msvc" {
        return;
    }

    // `/MANIFEST:EMBED` is MSVC's, which is why the guard tests the environment
    // and not only the operating system.
    for arg in [
        "/MANIFEST:EMBED",
        &format!("/MANIFESTINPUT:{}", manifest.display()),
    ] {
        println!("cargo:rustc-link-arg-bin=flyleaf={arg}");
    }
}
