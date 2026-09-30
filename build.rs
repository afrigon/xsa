use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const SHADER_DIRECTORY: &str = "shaders";

fn main() {
    println!("cargo::rerun-if-changed={SHADER_DIRECTORY}");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("cargo sets OUT_DIR"));

    for entry in fs::read_dir(SHADER_DIRECTORY).expect("reading the shader directory") {
        let source = entry.expect("reading a shader directory entry").path();
        if source.extension().is_none_or(|extension| extension != "slang") {
            continue;
        }
        let file_name = source.with_extension("spv");
        let output = out_dir.join(file_name.file_name().expect("shader files have a name"));

        let status = Command::new("slangc")
            .arg(&source)
            .args(["-target", "spirv", "-matrix-layout-column-major", "-o"])
            .arg(&output)
            .status()
            .expect("running slangc (is the mise environment active?)");
        assert!(status.success(), "slangc failed to compile {}", source.display());
    }
}
