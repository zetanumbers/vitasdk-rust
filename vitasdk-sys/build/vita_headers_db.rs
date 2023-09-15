use std::{collections::HashMap, fs, io, path::Path};

use serde::Deserialize;

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
}

#[derive(Deserialize)]
pub struct VitaImports {
    pub version: i32,
    pub firmware: String,
    pub modules: HashMap<String, VitaImportsModule>,
}

// TODO: weak
pub fn stub_lib_name<'a>(mod_name: &'a str, firmware: &'a str) -> impl std::fmt::Display + 'a {
    struct StubLibName<'a> {
        mod_name: &'a str,
        firmware: &'a str,
    }

    impl std::fmt::Display for StubLibName<'_> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
