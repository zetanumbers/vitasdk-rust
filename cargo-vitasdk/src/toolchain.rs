use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use camino::Utf8Path;
use eyre::WrapErr;
use once_cell::sync::Lazy;

use crate::{handle_exit_status, VITASDK};

#[tracing::instrument(parent = None)]
fn get_vitasdk_subpath(subpath: &Path) -> PathBuf {
    let path = VITASDK.join(subpath);
    tracing::debug!(?path);
    path
}

static VITA_ELF_CREATE: Lazy<PathBuf> =
    Lazy::new(|| get_vitasdk_subpath("bin/vita-elf-create".as_ref()));
static VITA_MKSFOEX: Lazy<PathBuf> = Lazy::new(|| get_vitasdk_subpath("bin/vita-mksfoex".as_ref()));
static VITA_MAKE_FSELF: Lazy<PathBuf> =
    Lazy::new(|| get_vitasdk_subpath("bin/vita-make-fself".as_ref()));
static VITA_PACK_VPK: Lazy<PathBuf> =
    Lazy::new(|| get_vitasdk_subpath("bin/vita-pack-vpk".as_ref()));

/// logging verbosity (more v is more verbose)
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LoggingVerbosity {
    V = 1,
    VV,
    VVV,
}

impl LoggingVerbosity {
    pub fn as_str(self) -> &'static str {
        &"-vvv"[..1 + self as usize]
    }
}

impl std::fmt::Display for LoggingVerbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug)]
pub struct VitaElfCreate<'a> {
    pub verbosity: Option<LoggingVerbosity>,
    pub allow_empty_imports: bool,
    /// path to yaml file
    pub config_options: Option<&'a Path>,
    /// input ARM ET_EXEC type ELF
    pub input_elf: &'a Path,
    /// output ET_SCE_RELEXEC type ELF
    pub output_velf: &'a Path,
}

impl<'a> VitaElfCreate<'a> {
    pub fn new(input_elf: &'a Path, output_velf: &'a Path) -> Self {
        VitaElfCreate {
            verbosity: None,
            allow_empty_imports: false,
            config_options: None,
            input_elf,
            output_velf,
        }
    }

    #[tracing::instrument]
    pub async fn run(&self) -> eyre::Result<()> {
        let mut command = tokio::process::Command::new(&*VITA_ELF_CREATE);

        if let Some(v) = self.verbosity {
            command.arg(v.as_str());
        }
        if self.allow_empty_imports {
            command.arg("-n");
        }
        if let Some(v) = self.config_options {
            command.arg("-e").arg(v);
        }
        command.args([self.input_elf, self.output_velf]);

        tracing::debug!(?command, "Running");
        let status = command.status().await?;

        handle_exit_status(status)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct VitaMksfoex<'a> {
    /// Add new DWORD values
    pub dword_values: &'a [(&'a str, u32)],
    /// Add new string values
    pub string_values: &'a [(&'a str, &'a str)],
    pub title: &'a OsStr,
    pub output_sfo: &'a Path,
}

impl<'a> VitaMksfoex<'a> {
    pub fn new(title: &'a OsStr, output_sfo: &'a Path) -> Self {
        VitaMksfoex {
            dword_values: &[],
            string_values: &[],
            title,
            output_sfo,
        }
    }

    #[tracing::instrument]
    pub async fn run(&self) -> eyre::Result<()> {
        use std::fmt::Write;

        let mut command = tokio::process::Command::new(&*VITA_MKSFOEX);

        let mut buffer = String::new();

        for (name, value) in self.dword_values {
            buffer.clear();
            write!(buffer, "{name}={value}").wrap_err("Writing dword values")?;
            command.arg("-d").arg(&buffer);
        }

        for (name, str) in self.string_values {
            buffer.clear();
            write!(buffer, "{name}={str}").wrap_err("Writing string values")?;
            command.arg("-s").arg(&buffer);
        }

        command.arg(self.title).arg(self.output_sfo);

        tracing::debug!(?command, "Running");
        let status = command.status().await?;

        handle_exit_status(status)?;

        Ok(())
    }
}

// TODO: figure out which authids to use (SceShell or vitasdk)

pub const AUTHID_DEFAULT: u64 = 0x2F00000000000001;
pub const AUTHID_SAFE: u64 = 0x2F00000000000002;
pub const AUTHID_SECRET_SAFE: u64 = 0x2F00000000000003;

#[derive(Clone, Debug)]
pub struct VitaMakeFself<'a> {
    /// Authid for permissions (see AUTHID_* constants)
    pub authid: u64,
    pub enable_compression: bool,
    /// Memory budget for the application in kilobytes. (Normal app: 0, System mode app: 0x1000 - 0x12800)
    pub memory_budget: Option<u32>,
    /// Physically contiguous memory budget for the application in kilobytes. (Note: The budget will be subtracted from standard memory budget)
    pub physically_contiguous_memory_budget: Option<u32>,
    /// ATTRIBUTE word in Control Info section 6.
    pub attrinbute_cinfo: Option<u32>,
    pub disable_aslr: bool,
    pub input_velf: &'a Path,
    pub output_eboot_bin: &'a Path,
}

impl<'a> VitaMakeFself<'a> {
    pub fn new(input_velf: &'a Path, output_eboot_bin: &'a Path) -> Self {
        Self {
            authid: AUTHID_DEFAULT,
            enable_compression: false,
            memory_budget: None,
            physically_contiguous_memory_budget: None,
            attrinbute_cinfo: None,
            disable_aslr: false,
            input_velf,
            output_eboot_bin,
        }
    }

    #[tracing::instrument]
    pub async fn run(&self) -> eyre::Result<()> {
        use std::fmt::Write;

        let mut command = tokio::process::Command::new(&*VITA_MAKE_FSELF);

        let mut buffer = String::new();

        write!(buffer, "{}", self.authid).wrap_err("Writing authid")?;
        command.arg("-a").arg(&buffer);

        if self.enable_compression {
            command.arg("-c");
        }

        if let Some(memory_budget) = self.memory_budget {
            buffer.clear();
            write!(buffer, "{memory_budget}").wrap_err("Writing memory budget")?;
            command.arg("-m").arg(&buffer);
        }

        if let Some(memory_budget) = self.physically_contiguous_memory_budget {
            buffer.clear();
            write!(buffer, "{memory_budget}")
                .wrap_err("Writing physically contiguous memory budget")?;
            command.arg("-pm").arg(&buffer);
        }

        if let Some(attribute_cinfo) = self.attrinbute_cinfo {
            buffer.clear();
            write!(buffer, "{attribute_cinfo}").wrap_err("Writing attribute cinfo")?;
            command.arg("-at").arg(&buffer);
        }

        if self.disable_aslr {
            command.arg("-na");
        }

        command.arg(self.input_velf).arg(self.output_eboot_bin);

        tracing::debug!(?command, "Running");
        let status = command.status().await?;

        handle_exit_status(status)?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct VitaPackVpk<'a> {
    /// sets the param.sfo file
    param_sfo: &'a Path,
    /// sets the eboot.bin file
    eboot_bin: &'a Path,
    /// adds the file or directory src to the vpk as dst
    additional_data: &'a [(&'a Utf8Path, &'a Utf8Path)],
    output_vpk: &'a Path,
}

impl<'a> VitaPackVpk<'a> {
    pub fn new(param_sfo: &'a Path, eboot_bin: &'a Path, output_vpk: &'a Path) -> Self {
        VitaPackVpk {
            param_sfo,
            eboot_bin,
            additional_data: &[],
            output_vpk,
        }
    }

    pub async fn run(&self) -> eyre::Result<()> {
        use std::fmt::Write;

        let mut command = tokio::process::Command::new(&*VITA_PACK_VPK);

        command
            .arg("--sfo")
            .arg(self.param_sfo)
            .arg("--eboot")
            .arg(self.eboot_bin);

        let mut buffer = String::new();
        for (src, dst) in self.additional_data {
            buffer.clear();
            write!(buffer, "{src}={dst}").wrap_err("Writing additional file paths")?;
            command.arg("--add").arg(&buffer);
        }

        command.arg(self.output_vpk);

        tracing::debug!(?command, "Running");
        let status = command.status().await?;

        handle_exit_status(status)?;

        Ok(())
    }
}
