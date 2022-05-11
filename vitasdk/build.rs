use camino::{Utf8Path, Utf8PathBuf};
use std::{
    convert::identity,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

fn main() {
    println!("cargo:rerun-if-env-changed=VITASDK");

    let sysroot = Utf8PathBuf::from(env::var("VITASDK").expect(
        "Vitasdk isn't installed or VITASDK environment variable isn't set to a valid unicode",
    ))
    .join("arm-vita-eabi");

    assert!(sysroot.exists(), "VITASDK's sysroot does not exist");

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

    // create_wrapper(out_dir, include);

    let bindings = bindgen::Builder::default()
        .header(include.join("vitasdk.h"))
        .clang_args(&["-I", include.as_str(), "-target", "armv7a-none-eabihf"])
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Generating vitasdk bindings")
        .to_string();

    let file = syn::parse_file(&identity(bindings)).expect("Parsing generated rust bindings");
}

struct Link {}

impl Link {
    fn load() -> Self {}
}

fn create_wrapper(out_dir: PathBuf, include: Utf8PathBuf) {
    let wrapper =
        fs::File::create(out_dir.join("wrapper.h")).expect("Creating a `wrapper.h` header");
    try_visit_recursive(include.as_std_path(), &mut |entry| {
        if entry.file_type()?.is_file() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "h") {
                let include_path = Utf8Path::from_path(path.strip_prefix(&include).unwrap())
                    .expect(
                        "Every header's path relative to their `include` dir should be unicode",
                    );
                writeln!(&wrapper, "#include <{include_path}>")
                    .expect("Writing a `wrapper.h` header");
            }
        };
        Ok(())
    })
    .expect("Querying all of the header files");
    drop(wrapper);
}

/// Visit order isn't specified
fn try_visit_recursive<V>(directory: &Path, visitor: &mut V) -> io::Result<()>
where
    V: FnMut(fs::DirEntry) -> io::Result<()>,
{
    let entries = fs::read_dir(directory)?;
    for entry in entries {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            try_visit_recursive(&entry.path(), visitor)?;
        }
        visitor(entry)?;
    }

    Ok(())
}
