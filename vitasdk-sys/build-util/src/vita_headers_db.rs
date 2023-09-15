use std::{collections::HashMap, fmt, fs, io, path::Path};

use serde::Deserialize;

/// Blacklisted stub libs containing conflicting symbol definitions.
const CONFLICTING_STUB_LIBS: [&str; 4] = [
    // Defines `__aeabi_uidiv`, which is also defined by compiler_builtins.
    "SceSysclibForDriver_stub",
    // Defines `__aeabi_unwind_cpp_pr0` and probably other symbols that seem
    // to collide with std.
    "SceLibc_stub",
    // This one overrides pthread_getspecific and friends, which makes the app
    // crash when using thread locals...
    "SceLibMonoBridge_stub",
    // Conflicts with compiler_builtins
    "SceRtabi_stub",
];

pub struct VitaDb {
    pub files_by_firmware: HashMap<String, Vec<VitaImports>>,
}

impl VitaDb {
    pub fn load(db: &Path) -> eyre::Result<Self> {
        let mut files_by_firmware = HashMap::<_, Vec<_>>::new();

        for version_dir in db.read_dir()? {
            for yml in version_dir?.path().read_dir()? {
                let yml = yml?.path();
                log::debug!("Loading: {}", yml.display());
                let file = fs::File::open(yml)?;
                let reader = io::BufReader::new(file);
                let imports: VitaImports = serde_yaml::from_reader(reader)?;
                let firmware = imports.firmware.clone();
                files_by_firmware.entry(firmware).or_default().push(imports);
            }
        }

        Ok(VitaDb { files_by_firmware })
    }

    pub fn remove_conflicting(&mut self) {
        self.files_by_firmware
            .values_mut()
            .flatten()
            .for_each(|imports| {
                imports.modules.retain(|name, _| {
                    let stub_lib = stub_lib_name(name, &imports.firmware).to_string();
                    !CONFLICTING_STUB_LIBS.contains(&stub_lib.as_str())
                })
            })
    }

    /// Check missing `vitasdk-sys` features gating link with stub libs.
    pub fn missing_features(&self) -> Vec<String> {
        const VITASDK_SYS_MANIFEST: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../Cargo.toml");

        #[derive(serde::Deserialize)]
        struct CargoManifest {
            #[serde(default)]
            features: HashMap<String, Vec<String>>,
        }

        let manifest = fs::read_to_string(VITASDK_SYS_MANIFEST).unwrap();
        let manifest: CargoManifest = toml::from_str(&manifest).unwrap();

        self.files_by_firmware
            .values()
            .flatten()
            .flat_map(|imports| {
                imports
                    .modules
                    .keys()
                    .map(|name| stub_lib_name(name, &imports.firmware).to_string())
            })
            .filter(|stub_lib| !manifest.features.contains_key(stub_lib))
            .collect()
    }
}

#[derive(Deserialize)]
pub struct VitaImports {
    pub version: i32,
    pub firmware: String,
    pub modules: HashMap<String, VitaImportsModule>,
}

// TODO: weak
pub fn stub_lib_name<'a>(mod_name: &'a str, firmware: &'a str) -> impl fmt::Display + 'a {
    struct StubLibName<'a> {
        mod_name: &'a str,
        firmware: &'a str,
    }

    impl fmt::Display for StubLibName<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.mod_name)?;
            if self.firmware != "3.60" {
                f.write_str("_")?;
                self.firmware.split('.').try_for_each(|s| f.write_str(s))?;
            }
            f.write_str("_stub")
        }
    }

    StubLibName { mod_name, firmware }
}

#[derive(Deserialize)]
pub struct VitaImportsModule {
    pub nid: u32,
    pub libraries: HashMap<String, VitaImportsLib>,
}

#[derive(Deserialize)]
pub struct VitaImportsLib {
    pub kernel: bool,
    pub nid: u32,
    pub version: Option<u32>,
    #[serde(rename = "functions", default)]
    pub function_nids: HashMap<String, u32>,
    #[serde(rename = "variables", default)]
    pub variable_nids: HashMap<String, u32>,
}
