#![feature(exit_status_error)]

#[path = "build/link_visitor.rs"]
mod link_visitor;
#[path = "build/vita_headers_db.rs"]
mod vita_headers_db;

use std::{borrow::Cow, env, fs, process};

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::{self, Context};
use quote::ToTokens;
use regex::Regex;
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

    let vita_headers_submodule = Utf8Path::new("vita-headers");

    let original_include = vita_headers_submodule.join("include");
    println!("cargo:rerun-if-changed={original_include}");

    let out_dir = Utf8PathBuf::from(env::var("OUT_DIR").unwrap());
    let include = out_dir.join("vita_headers_localized_include");
    fs::remove_dir_all(&include)
        .or_else(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err(e),
        })
        .unwrap();

    localize_bindings(&original_include, &include);

    for entry in Utf8Path::new("src/headers").read_dir_utf8().unwrap() {
        let entry = entry.unwrap();
        fs::copy(entry.path(), include.join(entry.file_name())).unwrap();
    }

    log::info!("Generating preprocessed bindings");
    let bindings = generate_preprocessed_bindings(&include)?;
    let bindings_output = out_dir.join("preprocessed_bindings.rs");

    log::info!("Parsing preprocessed bindings");
    let mut bindings = syn::parse_file(&bindings)?;

    let db = vita_headers_submodule.join("db");

    log::info!("Loading vita-headers metadata yaml files from \"{db}\"");
    let mut link = Link::load(&db, bindings_output.into())?;
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

    log::info!("Writing postprocessed bindings into {bindings_output_path}");
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
        .clang_args(&["-target", "armv7a-none-eabihf"])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .use_core()
        .ctypes_prefix("crate::ctypes")
        .generate_comments(false)
        .prepend_enum_name(false)
        .detect_include_paths(false)
        .formatter(bindgen::Formatter::None)
        .generate()
        .wrap_err("Bindgen failed")?
        .to_string())
}

// Replace `#include <>` with `#include ""`
fn localize_bindings(original_include: &Utf8Path, localized_include: &Utf8Path) {
    struct Localizer<'a> {
        include_regex: Regex,
        local_include_root: &'a Utf8Path,
    }

    impl<'a> Localizer<'a> {
        fn new(local_include_root: &'a Utf8Path) -> Self {
            Localizer {
                include_regex: Regex::new(r"#include <([\w/]+\.h)>").unwrap(),
                local_include_root,
            }
        }

        fn localize_dir(&self, original_include: &Utf8Path, local_include: &Utf8Path) {
            fs::create_dir(local_include).unwrap();
            for entry in original_include.read_dir_utf8().unwrap() {
                let entry = entry.unwrap();
                let local_entry = local_include.join(entry.file_name());
                let original_entry = entry.path();
                let ty = entry.file_type().unwrap();
                if ty.is_dir() {
                    self.localize_dir(original_entry, &local_entry)
                } else if ty.is_file() {
                    self.localize_file(original_entry, &local_entry)
                } else {
                    panic!("{original_entry:?} is bad file type: {ty:?}")
                }
            }
        }

        // TODO: trace span to better find sources of errors
        fn localize_file(&self, original_include: &Utf8Path, local_include: &Utf8Path) {
            let relative_local_root = local_include
                .strip_prefix(self.local_include_root)
                .unwrap()
                .ancestors()
                .skip(2)
                .map(|_| "..")
                .collect::<Utf8PathBuf>();
            let original_include = fs::read_to_string(original_include).unwrap();
            let new_include = self.include_regex.replace_all(
                &original_include,
                |captures: &regex::Captures<'_>| {
                    if let "stddef.h" | "stdint.h" | "stdarg.h" = &captures[1] {
                        return captures[0].to_owned();
                    }
                    let path = relative_local_root.join(&captures[1]);
                    format!("#include \"{path}\"")
                },
            );
            fs::write(local_include, new_include.as_ref()).unwrap();
        }
    }

    Localizer::new(localized_include).localize_dir(original_include, localized_include);
}
