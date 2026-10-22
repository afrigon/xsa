use std::env;
use std::path::Path;
use std::process::Command;

const RELEASE_TAG_PREFIX: &str = "xsa-v";

fn main() {
    println!("cargo:rerun-if-env-changed=CI");
    watch_git_path("HEAD");
    watch_git_path("packed-refs");
    watch_git_path("refs");

    let kind = if is_official_release() { "release" } else { "local" };
    println!("cargo:rustc-env=XSA_BUILD_KIND={kind}");

    if let Some(branch) = git(&["symbolic-ref", "-q", "--short", "HEAD"]) {
        println!("cargo:rustc-env=XSA_GIT_BRANCH={branch}");
    }

    if let Some(commit) = git(&["rev-parse", "--short", "HEAD"]) {
        println!("cargo:rustc-env=XSA_GIT_COMMIT={commit}");
    }
}

// An official release is a CI build of a commit tagged with this crate's own version; a commit carrying another
// release tag fails the build rather than ship under a version its tag does not name.
fn is_official_release() -> bool {
    if env::var("CI").as_deref() != Ok("true") {
        return false;
    }

    let expected = format!("{RELEASE_TAG_PREFIX}{}", env!("CARGO_PKG_VERSION"));
    let pattern = format!("{RELEASE_TAG_PREFIX}*");
    let tags = git(&["tag", "--points-at", "HEAD", "--list", &pattern]).unwrap_or_default();
    let tags: Vec<&str> = tags.lines().collect();

    if tags.contains(&expected.as_str()) {
        return true;
    }

    if !tags.is_empty() {
        panic!(
            "HEAD is tagged {} but the xsa crate's version is tagged {expected}",
            tags.join(", ")
        );
    }

    false
}

fn watch_git_path(path: &str) {
    let Some(resolved) = git(&["rev-parse", "--git-path", path]) else {
        return;
    };

    if Path::new(&resolved).exists() {
        println!("cargo:rerun-if-changed={resolved}");
    }
}

fn git(arguments: &[&str]) -> Option<String> {
    let output = Command::new("git").args(arguments).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8(output.stdout).ok()?;
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}
