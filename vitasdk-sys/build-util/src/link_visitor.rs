use std::{
    collections::{hash_map, HashMap, HashSet},
    path::Path,
    rc::Rc,
};

use crate::vita_headers_db::{missing_features_filter, stub_lib_name, VitaDb};
use syn::visit_mut::VisitMut;

const DEFINED_ELSEWHERE_FUNCTIONS: [(&str, &str); 3] = [
    ("vitasdk_get_tls_data", "vitasdk-utils"),
    ("vitasdk_get_pthread_data", "vitasdk-utils"),
    ("vitasdk_delete_thread_reent", "vitasdk-utils"),
];
const DEFINED_ELSEWHERE_VARIABLES: [(&str, &str); 0] = [];

pub use syn;

pub struct Link {
    /// link.function[function_name] = stub_library_name
    function: HashMap<String, Rc<str>>,
    /// link.variable[variable_name] = stub_library_name
    variable: HashMap<String, Rc<str>>,
    stub_libs: HashSet<Rc<str>>,
}

impl Link {
    pub fn load(db: &Path) -> Self {
        let mut link = Link {
            function: DEFINED_ELSEWHERE_FUNCTIONS
                .into_iter()
                .map(|(func, feat)| (func.into(), Rc::from(feat.to_owned())))
                .collect(),
            variable: DEFINED_ELSEWHERE_VARIABLES
                .into_iter()
                .map(|(var, feat)| (var.into(), Rc::from(feat.to_owned())))
                .collect(),
            stub_libs: HashSet::new(),
        };

        let mut db = VitaDb::load(db);

        let mut predicate = missing_features_filter();

        for imports in db.imports_by_firmware.values() {
            for (mod_name, mod_data) in &imports.modules {
                for (lib_name, lib) in &mod_data.libraries {
                    if lib.kernel {
                        continue;
                    }

                    let stub_lib_name = stub_lib_name(
                        mod_name,
                        lib_name,
                        lib.stub_name.as_deref(),
                        lib.kernel,
                        &imports.firmware,
                    )
                    .to_string();
                    let stub_lib_name = link
                        .stub_libs
                        .get(stub_lib_name.as_str())
                        .cloned()
                        .unwrap_or_else(|| Rc::from(stub_lib_name));

                    for function_name in lib.function_nids.keys() {
                        match link.function.entry(function_name.clone()) {
                            hash_map::Entry::Occupied(entry) => {
                                panic!(
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

                    for variable_name in lib.variable_nids.keys() {
                        match link.variable.entry(variable_name.clone()) {
                            hash_map::Entry::Occupied(entry) => {
                                panic!(
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

                    link.stub_libs.insert(stub_lib_name);
                }
            }
        }

        let conflicting_db = db.split_conflicting();
        conflicting_db.stub_lib_names().for_each(|lib_stub| {
            link.stub_libs.remove(lib_stub.as_str());
        });

        if db.stub_lib_names().any(|s| predicate(&s)) {
            panic!("Missing features in vitasdk-sys `Cargo.toml`. \
                Please run `cargo run -p build-util -- stub-libs --as-features` and replace stub lib features in vitasdk-sys Cargo.toml with outputed ones.")
        }

        link
    }
}

impl VisitMut for Link {
    fn visit_foreign_item_fn_mut(&mut self, i: &mut syn::ForeignItemFn) {
        let symbol = i.sig.ident.to_string();

        match self.function.get(&symbol) {
            None => panic!("Undefined foreign fn `{symbol}`"),
            Some(feature) => i.attrs.extend([
                syn::parse_quote!(#[cfg(feature = #feature)]),
                syn::parse_quote!(#[cfg_attr(docsrs, doc(cfg(feature = #feature)))]),
            ]),
        }

        syn::visit_mut::visit_foreign_item_fn_mut(self, i)
    }

    fn visit_foreign_item_static_mut(&mut self, i: &mut syn::ForeignItemStatic) {
        let symbol = i.ident.to_string();

        match self.variable.get(&symbol) {
            None => panic!("Undefined foreign static `{symbol}`"),
            Some(feature) => i.attrs.extend([
                syn::parse_quote!(#[cfg(feature = #feature)]),
                syn::parse_quote!(#[cfg_attr(docsrs, doc(cfg(feature = #feature)))]),
            ]),
        }

        syn::visit_mut::visit_foreign_item_static_mut(self, i)
    }

    fn visit_file_mut(&mut self, i: &mut syn::File) {
        i.items.extend(self.stub_libs.iter().map(|stub_lib_name| {
            syn::parse_quote! {
                #[cfg(feature = #stub_lib_name)]
                #[link(name = #stub_lib_name, kind = "static")]
                extern "C" {}
            }
        }));

        syn::visit_mut::visit_file_mut(self, i)
    }
}
