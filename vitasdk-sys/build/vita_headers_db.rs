use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct VitaImports {
    pub version: i32,
    pub firmware: String,
    pub modules: HashMap<String, VitaImportsModule>,
}

pub fn postfix(firmware: &str) -> VitaImportsPostfix<'_> {
    VitaImportsPostfix { firmware }
}

pub struct VitaImportsPostfix<'a> {
    pub firmware: &'a str,
}

impl<'a> std::fmt::Display for VitaImportsPostfix<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.firmware != "3.60" {
            f.write_str("_")?;
            self.firmware.split('.').try_for_each(|s| f.write_str(s))?;
        }
        Ok(())
    }
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
    #[serde(rename = "functions", default = "HashMap::new")]
    pub function_nids: HashMap<String, u32>,
    #[serde(rename = "variables", default = "HashMap::new")]
    pub variable_nids: HashMap<String, u32>,
}
