mod target_config;

use std::{
    env,
    path::{Path, PathBuf},
    process::{self, Command, Stdio},
};

use color_eyre::eyre::{self, eyre, WrapErr};

const TARGET_SPEC_NAME: &str = "arm-vita-eabi.json";

fn main() -> eyre::Result<()> {
    color_eyre::install()?;

    let mut args: Vec<String> = std::env::args().collect();
    // arg0
    args.remove(0);

    eyre::ensure!(
        args.remove(0) == "vitasdk",
        "first argument should be equal to `vitasdk`"
    );

    match args.get(0).map(String::as_str) {
        Some("build" | "b") => build().wrap_err("Executing `cargo vitasdk build` subcommand")?,
        Some("check" | "c") => todo!("cargo vitasdk check"),
        Some("docs" | "d") => todo!("cargo vitasdk docs"),
        Some(subcommand @ ("new" | "init")) => todo!("cargo vitasdk {subcommand}"),
        Some("run" | "r") => todo!("cargo vitasdk run"),
        Some("test" | "t") => todo!("cargo vitasdk test"),
        Some("help") | None => print_help(),
        // if flag instead of subcommand
        Some(flag) if flag.starts_with('-') => {
            if args.iter().any(|a| matches!(a.as_str(), "-h" | "--help")) {
                print_help();
            } else if args
                .iter()
                .any(|a| matches!(a.as_str(), "-V" | "--version"))
            {
                eprintln!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            } else {
                panic!("Unknown flag: {}", flag);
            }
        }
        Some(unknown) => panic!(
            "Unknown subcommand: {}. Type `cargo vitasdk help` to print help.",
            unknown
        ),
    }

    Ok(())
}

fn print_help() {
    eprintln!(
        "Cargo wrapper for building psvita apps

USAGE:
    cargo [+toolchain] psvita [OPTIONS] [SUBCOMMAND]

Subcommands:
    build, b    Compile the current package into vpk
    check, c    Analyze the current package and report errors, but don't build object files
    doc, d      Build this package's and its dependencies' documentation
    new         Create a new cargo package
    init        Create a new cargo package in an existing directory
    run, r      Run a binary or example of the local package
    test, t     Run the tests
    help        Print help
"
    );
}

fn build() -> eyre::Result<()> {
    let manifest_path = get_arg(3, "--manifest-path");
    let target_spec_path = target_spec_path(manifest_path.as_deref())
        .wrap_err_with(|| format!("Searching for `{TARGET_SPEC_NAME}` target spec"))?;

    let mut build_cmd = Command::new("xargo");
    build_cmd
        .arg("build")
        .arg("--message-format=json-render-diagnostics")
        .arg("--target")
        .arg(target_spec_path)
        .args(env::args_os().skip(3))
        .stdout(Stdio::piped());

    todo!()
}

fn get_arg(skip: usize, name: &str) -> Option<String> {
    let mut args = std::env::args();
    if skip > 0 {
        args.nth(skip - 1);
    }
    let mut target_next = false;
    for arg in args {
        if target_next {
            return Some(arg);
        }

        match arg.strip_prefix(name) {
            Some("") => target_next = true,
            Some(a) if a.starts_with('=') => return Some(a[1..].to_owned()),
            None | Some(_) => continue,
        }
    }

    None
}

fn target_spec_path(manifest_path: Option<&str>) -> eyre::Result<PathBuf> {
    let mut cmd = process::Command::new("xargo");
    cmd.args(&["locate-project", "--offline", "--message-format=plain"])
        .stderr(Stdio::inherit())
        .stdin(Stdio::null())
        .stdout(Stdio::piped());

    if let Some(manifest_path) = manifest_path {
        cmd.args(&["--manifest_path", &manifest_path]);
    }

    const PKG_SCOPE: &str = "Running `cargo locate-project` without `--workspace`";
    const WS_SCOPE: &str = "Running `cargo locate-project --workspace`";

    // Spawn in parallel to minimize latency
    let pkg_child = cmd.spawn().wrap_err("Spawning").wrap_err(PKG_SCOPE)?;
    cmd.arg("--workspace");
    let ws_child = cmd.spawn().wrap_err("Spawning").wrap_err(WS_SCOPE)?;

    wait_cargo_locate_project(pkg_child)
        .wrap_err(PKG_SCOPE)
        .transpose()
        .or_else(|| {
            wait_cargo_locate_project(ws_child)
                .wrap_err(WS_SCOPE)
                .transpose()
        })
        .unwrap_or_else(|| Err(eyre!("No `{TARGET_SPEC_NAME}` found in package's root nor in the workspace root. Perhaps this is not a vitasdk-rust project (create a new one using `cargo vitasdk new`).")))
}

fn wait_cargo_locate_project(child: process::Child) -> eyre::Result<Option<PathBuf>> {
    let output = child.wait_with_output().wrap_err("Waiting until finish")?;
    eyre::ensure!(output.status.success(), "Process failed");

    let manifest =
        Path::new(std::str::from_utf8(&output.stdout).wrap_err("Converting stdout into utf-8")?);

    for dir_entry in manifest.parent().unwrap().read_dir()? {
        let dir_entry = dir_entry?;
        if dir_entry.file_name() == TARGET_SPEC_NAME {
            return Ok(Some(dir_entry.path()));
        }
    }

    Ok(None)
}
