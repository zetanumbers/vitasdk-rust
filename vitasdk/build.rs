#[path = "build/link_visitor.rs"]
mod link_visitor;
#[path = "build/vita_headers_db.rs"]
mod vita_headers_db;

use std::{env, fs, path::PathBuf};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::{self, eyre};
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

    log::info!(
        "Writing preprocessed bindings into {}",
        bindings_output.display()
    );
    fs::write(&bindings_output, &bindings)?;

    log::info!("Parsing preprocessed bindings");
    let mut bindings = syn::parse_file(&bindings)?;

    let db = vitasdk.join("share/vita-headers/db");
    log::info!("Loading vita-headers metadata yaml files from \"{db}\"");
    let mut link = Link::load(&db, bindings_output)?;
    link.visit_file_mut(&mut bindings);
    let bindings = bindings.into_token_stream();

    let bindings_output_path = out_dir.join("bindings.rs");
    let mut bindings_output = fs::File::create(&bindings_output_path)?;

    use std::io::Write;
    log::info!(
        "Writing postprocessed bindings into {}",
        bindings_output_path.display()
    );
    write!(bindings_output, "{bindings}")?;

    // TODO: cargo fmt -- bindings.rs
    // TODO: swap types from `::str::os::raw`
    // TODO: include file into the actuall crate

    Ok(())
}

fn generate_preprocessed_bindings(include: &Utf8Path) -> eyre::Result<String> {
    Ok(bindgen::Builder::default()
        .header(include.join("vitasdk.h"))
        .clang_args(&["-I", include.as_str(), "-target", "armv7a-none-eabihf"])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .map_err(|()| eyre!("Bindgen failed"))?
        .to_string())
}
