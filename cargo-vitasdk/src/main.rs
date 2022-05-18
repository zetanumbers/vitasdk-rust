use std::{collections::VecDeque, env};

use color_eyre::eyre::{self, WrapErr};

fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

fn main() -> eyre::Result<()> {
    install_tracing();
    color_eyre::install()?;

    let mut args: VecDeque<String> = std::env::args().collect();
    // arg0
    args.pop_front();

    eyre::ensure!(
        args.pop_front().as_deref() == Some("vitasdk"),
        "first argument should be equal to `vitasdk`"
    );

    let executor = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .wrap_err("Creating tokio executor")?;

    match args.pop_front().as_deref() {
        Some("build" | "b") => executor
            .block_on(cargo_vitasdk::build(args.make_contiguous()))
            .wrap_err("Executing `cargo vitasdk build` subcommand")?,
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
                tracing::error!(flag, "Unknown flag");
                eyre::bail!("Unknown flag");
            }
        }
        Some(unknown) => {
            tracing::error!(subcommand = unknown, "Unknown subcommand");
            tracing::info!("Type `cargo vitasdk help` to print help.");
            eyre::bail!("Unknown subcommand");
        }
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
