use std::env;
use std::path::PathBuf;

fn main() {
    // This build script's purpose is to find the compiled binary for `reev-agent`
    // and provide its path to the `reev-runner` crate as a compile-time environment variable.
    // This avoids hardcoding the path and allows `reev-runner` to execute the agent
    // directly, preventing the recompilation issues seen with `cargo run -p reev-agent`.

    // Get the build profile ("debug", "release", etc.) from Cargo.
    let profile = env::var("PROFILE")
        .expect("The PROFILE environment variable is not set. This should be provided by Cargo.");

    // The `CARGO_MANIFEST_DIR` variable gives the path to the directory containing Cargo.toml for this crate.
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect(
        "The CARGO_MANIFEST_DIR environment variable is not set. This should be provided by Cargo.",
    );

    // Construct the path to the workspace root. Since this script is in `crates/reev-runner/`,
    // we navigate up two parent directories.
    let workspace_root = PathBuf::from(manifest_dir)
        .parent()
        .expect("Failed to get parent of manifest dir")
        .parent()
        .expect("Failed to get grandparent of manifest dir (workspace root)")
        .to_path_buf();

    // Construct the full path to the `reev-agent` executable within the workspace's target directory.
    let agent_bin_path = workspace_root
        .join("target")
        .join(profile)
        .join("reev-agent");

    // Pass the constructed path to the `reev-runner` crate code.
    // The `env!` macro can then be used in `src/lib.rs` to get this path.
    // Example: `let agent_path = env!("REEV_AGENT_PATH");`
    println!(
        "cargo:rustc-env=REEV_AGENT_PATH={}",
        agent_bin_path
            .to_str()
            .expect("Path to agent binary is not valid UTF-8")
    );

    // Tell Cargo to re-run this build script if it changes.
    println!("cargo:rerun-if-changed=build.rs");
}
