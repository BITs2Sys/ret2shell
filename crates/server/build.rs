use std::{env, fs, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("constants.rs");
    let version = format!(
        "{}-{}-{}",
        env!("CARGO_PKG_VERSION"),
        git_version::git_version!(
            args = ["--abbrev=8", "--always", "--dirty=*"],
            fallback = "unknown"
        )
        .to_uppercase(),
        rustc_version::version().unwrap()
    );
    fs::write(
        &dest_path,
        format!("pub const RET2SHELL_FULL_VERSION: &'static str = \"{version}\";\n"),
    )
    .unwrap();
}
