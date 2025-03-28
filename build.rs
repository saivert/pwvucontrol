use std::env;
use std::fs::copy;
use std::path::{Path, PathBuf};

fn main() {
    // This works around issue with rust-analyzer not picking up OUT_DIR environment variable
    // even if specified via LSP settings (BUG).
    // Having a build.rs file present activates the OUT_DIR environment variable but we
    // also need to copy the generated config.rs file there.
    println!("cargo::rerun-if-changed=src/config.rs.in");
    let out_dir = env::var("OUT_DIR").unwrap();
    let codegen_dir = env::var("CODEGEN_DIR").unwrap_or("builddir/src".to_owned());
    let sourcepath: PathBuf = [&codegen_dir, "config.rs"].iter().collect();
    if !Path::exists(&sourcepath) {
        panic!(
            r#"
            Please configure the project with meson once to generate config.rs!
            If it is not generated in builddir/src then you need to specify the path with the CODEGEN_DIR environment variable.
            "#
        );
    }
    let destpath: PathBuf = [&out_dir, "config.rs"].iter().collect();
    copy(sourcepath, destpath).expect("Copying config.rs success");
}
