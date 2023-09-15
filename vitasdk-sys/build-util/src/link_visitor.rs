use std::{
    collections::{hash_map, HashMap},
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::vita_headers_db::{stub_lib_name, VitaDb};
use eyre::Context;
use syn::{spanned::Spanned, visit_mut::VisitMut};

pub use syn;

pub struct Link {
    /// link.function[function_name] = stub_library_name
    function: HashMap<String, Rc<str>>,
    /// link.variable[variable_name] = stub_library_name
    variable: HashMap<String, Rc<str>>,
    source_file: PathBuf,
    pub undefined_functions: Vec<String>,
    pub undefined_variables: Vec<String>,
}

impl Link {
    pub fn load(db: &Path, source_file: PathBuf) -> eyre::Result<Self> {
        let mut link = Link {
            function: HashMap::new(),
            variable: HashMap::new(),
            source_file,
            undefined_functions: Vec::new(),
            undefined_variables: Vec::new(),
        };

        let mut db = VitaDb::load(db).wrap_err("Loading vita-header db")?;
        db.remove_conflicting();
        let missing_features = db.missing_features();
        if !missing_features.is_empty() {
            panic!("Missing features in vitasdk-sys `Cargo.toml`. Please run `cargo run -p build-util --bin missing_features` and paste outputed features into vitasdk-sys Cargo.toml")
        }

        for imports in db.files_by_firmware.into_values().flatten() {
            let firmware = imports.firmware;

            for (mod_name, mod_data) in imports.modules {
                let stub_lib_name = Rc::from(stub_lib_name(&mod_name, &firmware).to_string());

                for (_, lib_data) in mod_data.libraries {
                    if lib_data.kernel {
                        continue;
                    }

                    for (function_name, _) in lib_data.function_nids.into_iter() {
                        match link.function.entry(function_name) {
                            hash_map::Entry::Occupied(entry) => {
                                eyre::bail!(
                                    "`{}` extern function links both to `{}` and `{}`",
                                    entry.key(),
                                    entry.get(),
                                    stub_lib_name
                                );
                            }
                            hash_map::Entry::Vacant(entry) => {
                                entry.insert(Rc::clone(&stub_lib_name));
                            }
                        }
                    }

                    for (variable_name, _) in lib_data.variable_nids.into_iter() {
                        match link.variable.entry(variable_name) {
                            hash_map::Entry::Occupied(entry) => {
                                eyre::bail!(
                                    "`{}` extern variable links both to `{}` and `{}`",
                                    entry.key(),
                                    entry.get(),
                                    stub_lib_name
                                );
                            }
                            hash_map::Entry::Vacant(entry) => {
                                entry.insert(Rc::clone(&stub_lib_name));
                            }
                        }
                    }
                }
            }
        }

        Ok(link)
    }
}

impl VisitMut for Link {
    fn visit_item_foreign_mod_mut(&mut self, i: &mut syn::ItemForeignMod) {
        use std::fmt::Write;

        let mut stub_lib_name = None;

        let mut symbol = String::new();
        for item in &i.items {
            match item {
                syn::ForeignItem::Fn(item) => {
                    symbol.clear();
                    write!(symbol, "{}", item.sig.ident).unwrap();
                    let candidate = match self.function.get(&symbol) {
                        None => {
                            self.undefined_functions.push(symbol);
                            return;
                        }
                        Some(c) => c,
                    };
                    set_stub_lib_once(&mut stub_lib_name, candidate, &self.source_file, i);
                }
                syn::ForeignItem::Static(item) => {
                    symbol.clear();
                    write!(symbol, "{}", item.ident).unwrap();

                    let candidate = match self.variable.get(&symbol) {
                        None => {
                            self.undefined_variables.push(symbol);
                            return;
                        }
                        Some(c) => c,
                    };
                    set_stub_lib_once(&mut stub_lib_name, candidate, &self.source_file, i);
                }
                _ => (),
            }
        }

        if let Some(stub_lib_name) = stub_lib_name {
            i.attrs.extend([
                syn::parse_quote!(#[cfg(feature = #stub_lib_name)]),
                syn::parse_quote!(#[cfg_attr(docsrs, doc(cfg(feature = #stub_lib_name)))]),
                syn::parse_quote!(#[link(name = #stub_lib_name, kind = "static")]),
            ]);
        }

        syn::visit_mut::visit_item_foreign_mod_mut(self, i);
    }
}

fn set_stub_lib_once(
    stub_lib_name: &mut Option<Rc<str>>,
    candidate: &Rc<str>,
    source_file: &Path,
    i: &syn::ItemForeignMod,
) {
    match stub_lib_name {
        Some(stub_lib_name) => assert_eq!(
            stub_lib_name,
            candidate,
            "Found extern block, with two incompatible extern items at \"{}:{}\"",
            source_file.display(),
            i.span().start().line,
        ),
        None => *stub_lib_name = Some(Rc::clone(candidate)),
    }
}
