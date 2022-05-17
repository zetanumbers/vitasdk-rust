pub mod toolchain;

use std::{
    borrow::Cow,
    env,
    path::{Path, PathBuf},
    process::Stdio,
};

use eyre::{eyre, WrapErr};
use once_cell::sync::Lazy;
use tokio::io::{self, AsyncBufReadExt};
use tracing::Instrument;

const TARGET_SPEC_NAME: &str = "arm-vita-eabi.json";

const VITASDK: Lazy<PathBuf> = Lazy::new({
    #[tracing::instrument(parent = None)]
    fn get_vitasdk_root() -> PathBuf {
        let path =
            PathBuf::from(env::var_os("VITASDK").expect(
                "VITASDK environment variable isn't set, vitasdk isn't properly installed.",
            ));
        tracing::info!(?path);
        path
    }

    get_vitasdk_root
});

const CARGO: Lazy<Cow<'static, Path>> = Lazy::new({
    #[tracing::instrument(parent = None)]
    fn get_cargo_bin() -> Cow<'static, Path> {
        let path = env::var_os("CARGO").map_or(Cow::Borrowed(Path::new("cargo")), |bin| {
            Cow::Owned(PathBuf::from(bin))
        });
        tracing::info!(?path);
        path
    }

    get_cargo_bin
});

#[tracing::instrument]
pub async fn build<'e>(args: &[String]) -> eyre::Result<()> {
    Lazy::force(&VITASDK);

    let manifest_path =
        get_arg(&args, "--manifest-path").wrap_err("Getting value of `--manifest-path`")?;
    let target_spec_path = target_spec_path(manifest_path.as_deref())
        .await
        .wrap_err_with(|| format!("Searching for `{TARGET_SPEC_NAME}` target spec"))?;

    let mut build = tokio::process::Command::new("xargo");
    build
        .args(&[
            "build",
            "--message-format=json-render-diagnostics",
            "--target",
        ])
        .arg(target_spec_path)
        .args(args)
        .stdout(Stdio::piped());
    let mut build = build
        .spawn()
        .wrap_err_with(|| format!("Running `xargo build`: {build:?}"))?;

    let mut lines = io::BufReader::new(build.stdout.take().unwrap()).lines();
    let mut tasks = Vec::new();

    while let Some(line) = lines.next_line().await.wrap_err("Parsing stdout")? {
        let message: cargo_metadata::Message =
            serde_json::from_str(&line).wrap_err("Parsing `xargo build`'s stdout")?;
        match message {
            cargo_metadata::Message::CompilerArtifact(cargo_metadata::Artifact {
                executable: Some(executable),
                ..
            }) => tasks.push(tokio::spawn(postprocess_elf(executable.into()))),
            _ => (),
        }
    }

    for task in tasks {
        task.await
            .unwrap_or_else(|e| Err(e.into()))
            .wrap_err("Joining on postprocessing tasks")?
    }

    Ok(())
}

#[tracing::instrument]
pub async fn postprocess_elf<'e>(elf: PathBuf) -> eyre::Result<()> {
    let velf = elf.with_extension("velf");
    let sfo = elf.with_extension("sfo");
    let eboot_bin = elf.with_extension("eboot-bin");
    let vpk = elf.with_extension("vpk");

    // TODO: title
    let title = elf
        .file_stem()
        .ok_or_else(|| eyre!("`elf` path doesn't have a file stem"))?
        .to_owned();

    // TODO: other arguments via cargo metadata

    tokio::try_join!(
        async {
            toolchain::VitaMksfoex::new(&title, &velf).run().await?;
            eyre::Result::<()>::Ok(())
        },
        async {
            toolchain::VitaElfCreate::new(&elf, &velf).run().await?;
            toolchain::VitaMakeFself::new(&velf, &eboot_bin)
                .run()
                .await?;
            Ok(())
        }
    )?;

    toolchain::VitaPackVpk::new(&sfo, &eboot_bin, &vpk)
        .run()
        .await?;

    Ok(())
}

fn get_arg<'a>(args: &'a [String], name: &str) -> eyre::Result<Option<&'a str>> {
    args.windows(2)
        .find_map(|args| match args[0].strip_prefix(name) {
            Some("") => Some(
                args.get(1)
                    .map(String::as_str)
                    .ok_or_else(|| eyre!("Flag has no value")),
            ),
            Some(a) => a.strip_prefix('=').map(Ok),
            None => None,
        })
        .transpose()
}

#[tracing::instrument]
async fn target_spec_path(manifest_path: Option<&str>) -> eyre::Result<PathBuf> {
    let mut command = tokio::process::Command::new(&**CARGO);
    command
        .args(&["locate-project", "--offline", "--message-format=plain"])
        .stderr(Stdio::inherit())
        .stdin(Stdio::null())
        .stdout(Stdio::piped());

    if let Some(manifest_path) = manifest_path {
        command.args(&["--manifest_path", &manifest_path]);
    }

    // Spawn in parallel to minimize latency
    let pkg = tracing::info_span!("cargo-locate-project", workspace = false).in_scope(
        || -> eyre::Result<_> {
            tracing::debug!(?command);
            let child = command.spawn()?;
            Ok(target_spec_path_extract(child).in_current_span())
        },
    )?;
    command.arg("--workspace");

    let ws = tracing::info_span!("cargo-locate-project", workspace = true).in_scope(
        || -> eyre::Result<_> {
            tracing::debug!(?command);
            let child = command.spawn()?;
            Ok(target_spec_path_extract(child).in_current_span())
        },
    )?;

    let mut path = pkg.await?;
    if path.exists() {
        return Ok(path);
    }

    path = ws.await?;
    if path.exists() {
        return Ok(path);
    }

    eyre::bail!("No `{TARGET_SPEC_NAME}` found in package's root nor in the workspace root. Perhaps this is not a vitasdk-rust project (create a new one using `cargo vitasdk new`).");
}

async fn target_spec_path_extract(child: tokio::process::Child) -> eyre::Result<PathBuf> {
    let output = child
        .wait_with_output()
        .await
        .wrap_err("Waiting until finish")?;
    handle_exit_status(output.status)?;

    let manifest =
        Path::new(std::str::from_utf8(&output.stdout).wrap_err("Converting stdout into utf-8")?);

    let target_spec = manifest.parent().unwrap().join(TARGET_SPEC_NAME);
    Ok(target_spec)
}

fn handle_exit_status(exit_status: std::process::ExitStatus) -> eyre::Result<()> {
    if !exit_status.success() {
        tracing::error!(?exit_status, "Command failed");
        eyre::bail!("Command failed");
    }
    Ok(())
}
