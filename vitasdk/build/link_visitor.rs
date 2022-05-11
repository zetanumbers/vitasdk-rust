use std::{
    collections::{hash_map, HashMap},
    fs,
    io::BufReader,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::vita_headers_db::{self, VitaImports};
use camino::Utf8Path;
use color_eyre::eyre;
use syn::{spanned::Spanned, visit_mut::VisitMut};

pub struct Link {
    /// link.function[function_name] = stub_library_name
    function: HashMap<String, Rc<str>>,
    /// link.variable[variable_name] = stub_library_name
    variable: HashMap<String, Rc<str>>,
    source_file: PathBuf,
    link_path_quote: syn::Path,
}

impl Link {
    pub fn load(db: &Utf8Path, source_file: PathBuf) -> eyre::Result<Self> {
        let mut link = Link {
            function: HashMap::new(),
            variable: HashMap::new(),
            link_path_quote: syn::parse_quote!(link),
            source_file,
        };

        for version_dir in db.read_dir()? {
            for yml in version_dir?.path().read_dir()? {
                let yml = yml?.path();
                log::debug!("Loading: {}", yml.display());
                let file = fs::File::open(yml)?;
                let reader = BufReader::new(file);
                let imports: VitaImports = serde_yaml::from_reader(reader)?;
                let firmware = imports.firmware;

                for (mod_name, mod_data) in imports.modules {
                    let postfix = vita_headers_db::postfix(&firmware);
                    let stub_lib_name = Rc::from(format!("{mod_name}{postfix}_stub"));

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
                            log::debug!("Found undefined extern function, assuming it's from libc: {symbol}");
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
                            log::debug!("Found undefined extern variable, assuming it's from libc: {symbol}");
                            return;
                        }
                        Some(c) => c,
                    };
                    set_stub_lib_once(&mut stub_lib_name, candidate, &self.source_file, i);
                }
                syn::ForeignItem::Verbatim(ts) => panic!(
                    "Unexpected syn's verbatim foreign item encountered at \"{}:{}\"",
                    self.source_file.display(),
                    ts.span().start().line,
                ),
                _ => (),
            }
        }

        if let Some(stub_lib_name) = stub_lib_name {
            i.attrs.iter().find(|a| a.path == self.link_path_quote);
            i.attrs.push(syn::parse_quote! {
                #[link(name = #stub_lib_name, kind = "static")]
            });
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
