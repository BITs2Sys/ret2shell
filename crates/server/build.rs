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
    let full_version = format!(
        "{version}-{}-{}-{}",
        build_target::target_arch().unwrap(),
        build_target::target_os().unwrap(),
        build_target::target_env().unwrap(),
    );
    fs::write(
        dest_path,
        format!("pub const R2S_VERSION: &'static str = \"{version}\";\npub const R2S_FULL_VERSION: &'static str = \"{full_version}\";\n"),
    )

    .unwrap();
}
