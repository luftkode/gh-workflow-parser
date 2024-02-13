use std::error::Error;
use std::fs::File;
use std::io::{self, Read};
use std::{env, fs, path::Path};

// Convenience macro for printing warnings during the build process
//
// Currently the only way to print "info" messages during the build process (see: https://github.com/rust-lang/cargo/issues/985)
macro_rules! print_warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

const GH_CLI_PATH: &str = "gh_cli/gh";

/// Name of the file that contains the byte array for the GitHub CLI file
const INCLUDE_GH_CLI_FILE: &str = "include_gh_cli.rs";

/// Get the length (size in bytes) of a file
///
/// Reads the whole file into memory and returns the length of the vector.
/// This is more reliable than using `std::fs::metadata` because it doesn't rely on the file system.
pub fn file_len(fpath: &Path) -> io::Result<usize> {
    const PRE_ALLOC: usize = 1024 * 1024 * 20; // 20 MiB
    let mut file = File::open(fpath)?;
    let mut raw_mtgogetter = Vec::with_capacity(PRE_ALLOC);
    file.read_to_end(raw_mtgogetter.as_mut())?;
    Ok(raw_mtgogetter.len())
}

fn main() {
    // Re-run this build script if the build script itself changes or if the gh_cli/gh file changes
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=gh_cli/gh");

    let env_out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set");
    let out_dir_path = Path::new(&env_out_dir);
    let gh_cli_path = Path::new(GH_CLI_PATH);

    if !gh_cli_path.exists() {
        print_warn!("{GH_CLI_PATH} does not exist");
        std::process::exit(1);
    }

    include_gh_cli(out_dir_path, gh_cli_path).unwrap();
}

fn include_gh_cli(out_dir: &Path, gh_cli_path: &Path) -> Result<(), Box<dyn Error>> {
    let gh_cli_bytes = fs::read(gh_cli_path).expect("Failed to read gh_cli/gh");
    let github_cli_size = gh_cli_bytes.len();

    let gh_cli_path = out_dir.join("gh_cli");
    fs::write(&gh_cli_path, gh_cli_bytes).expect("Failed to write gh_cli_bz2");

    let include_gh_cli_rs_contents = format_include_gh_cli_rs(github_cli_size, &gh_cli_path);

    let include_gh_cli_rs_path = out_dir.join(INCLUDE_GH_CLI_FILE);
    fs::write(include_gh_cli_rs_path, include_gh_cli_rs_contents)
        .expect("Failed to write include_gh_cli.rs");

    Ok(())
}

/// Format the contents of the `include_gh_cli.rs` file
fn format_include_gh_cli_rs(gh_cli_size: usize, gh_cli_path: &Path) -> String {
    format!(
        r#"
        pub const GH_CLI_BYTES: &[u8; {gh_cli_size}] = include_bytes!({gh_cli_path:?});
        "#
    )
}
