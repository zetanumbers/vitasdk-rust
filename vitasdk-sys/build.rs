#![feature(exit_status_error)]

#[path = "build/link_visitor.rs"]
mod link_visitor;
#[path = "build/vita_headers_db.rs"]
mod vita_headers_db;

use std::{borrow::Cow, env, fs, path::PathBuf, process};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::{self, Context};
use quote::ToTokens;
use syn::visit_mut::VisitMut;

use crate::link_visitor::Link;

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    env_logger::init();

    println!("cargo:rerun-if-env-changed=VITASDK");
    let vitasdk = Utf8PathBuf::from(env::var("VITASDK").expect(
        "Vitasdk isn't installed or VITASDK environment variable isn't set to a valid unicode",
    ));
    let sysroot = vitasdk.join("arm-vita-eabi");

    assert!(
        sysroot.exists(),
        "VITASDK's sysroot does not exist, please install or update vitasdk first"
    );

    let lib = sysroot.join("lib");
    assert!(lib.exists(), "VITASDK's `lib` directory does not exist");
    println!("cargo:rustc-link-search=native={lib}");

    let include = sysroot.join("include");
    assert!(
        include.exists(),
        "VITASDK's `include` directory does not exist"
    );
    println!("cargo:rerun-if-changed={include}");

    let out_dir =
        PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR environment variable isn't set"));

    log::info!("Generating preprocessed bindings");
    let bindings = generate_preprocessed_bindings(&include)?;
    let bindings_output = out_dir.join("preprocessed_bindings.rs");

    log::info!("Parsing preprocessed bindings");
    let mut bindings = syn::parse_file(&bindings)?;

    let db = vitasdk.join("share/vita-headers/db");

    log::info!("Loading vita-headers metadata yaml files from \"{db}\"");
    let mut link = Link::load(&db, bindings_output)?;
    link.visit_file_mut(&mut bindings);

    if !link.undefined_functions.is_empty() {
        log::warn!(
            "Found undefined functions, assuming they're from libc: {:?}",
            link.undefined_functions
        );
    }
    if !link.undefined_variables.is_empty() {
        log::warn!(
            "Found undefined variables, assuming they're from libc: {:?}",
            link.undefined_variables
        );
    }

    let bindings = bindings.into_token_stream();

    let bindings_output_path = out_dir.join("bindings.rs");
    let mut bindings_output = fs::File::create(&bindings_output_path)?;

    log::info!(
        "Writing postprocessed bindings into {}",
        bindings_output_path.display()
    );
    use std::io::Write;
    write!(bindings_output, "{bindings}")?;

    let mut fmt_cmd = process::Command::new(
        env::var_os("CARGO").map_or_else(|| Cow::Borrowed("cargo".as_ref()), Cow::Owned),
    );
    fmt_cmd.args(["fmt", "--"]);
    fmt_cmd.arg(bindings_output_path);

    log::info!("Running formatting command: {fmt_cmd:?}");
    let exit_status = fmt_cmd.status()?;

    log::info!("Formatting command finished");
    exit_status.exit_ok()?;

    Ok(())
}

fn generate_preprocessed_bindings(include: &Utf8Path) -> eyre::Result<String> {
    Ok(bindgen::Builder::default()
        .header(include.join("vitasdk.h"))
        .clang_args(&["-I", include.as_str(), "-target", "armv7a-none-eabihf"])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .use_core()
        .ctypes_prefix("crate::ctypes")
        .generate_comments(false)
        .prepend_enum_name(false)
        .detect_include_paths(false)
        .generate()
        .wrap_err("Bindgen failed")?
        .to_string())
}
