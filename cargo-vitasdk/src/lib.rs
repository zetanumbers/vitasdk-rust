pub mod toolchain;

use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::SystemTime,
};

use eyre::{eyre, WrapErr};
use futures::future;
use once_cell::sync::Lazy;
use tokio::io::{self, AsyncBufReadExt};
use tracing::Instrument;

const TARGET_SPEC_NAME: &str = "armv7-sony-vita-newlibeabihf";

static VITASDK: Lazy<PathBuf> = Lazy::new({
    #[tracing::instrument(parent = None)]
    fn get_vitasdk_root() -> PathBuf {
        let path =
            PathBuf::from(env::var_os("VITASDK").expect(
                "VITASDK environment variable isn't set, vitasdk isn't properly installed.",
            ));
        tracing::debug!(?path);
        path
    }

    get_vitasdk_root
});

#[tracing::instrument]
pub async fn build<'e>(args: &[String]) -> eyre::Result<()> {
    Lazy::force(&VITASDK);

    let cargo = env::var_os("CARGO");
    let cargo = cargo.as_deref().unwrap_or_else(|| "cargo".as_ref());
    let mut build = tokio::process::Command::new(cargo);
    build
        .args([
            "build",
            "--message-format=json-render-diagnostics",
            "-Zbuild-std=panic_abort,std",
            "--target",
        ])
        .arg(TARGET_SPEC_NAME)
        .args(args)
        .stdout(Stdio::piped());

    let mut tasks = Vec::new();
    let parent_span = tracing::Span::current();

    {
        let _entered = tracing::debug_span!("cargo-build", command = ?build).entered();
        let mut build = build.spawn()?;
        tracing::trace!("Spawned");

        let mut lines = io::BufReader::new(build.stdout.take().unwrap()).lines();

        while let Some(line) = lines.next_line().await.wrap_err("Parsing stdout")? {
            let message: cargo_metadata::Message =
                serde_json::from_str(&line).wrap_err("Parsing `cargo build`'s stdout")?;

            if let cargo_metadata::Message::CompilerArtifact(cargo_metadata::Artifact {
                executable: Some(executable),
                ..
            }) = message
            {
                tasks.push(tokio::spawn(
                    postprocess_elf(executable.into()).instrument(parent_span.clone()),
                ))
            }
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

    #[tracing::instrument]
    async fn is_cached(name: &str, inputs: &[&Path], output: &Path) -> eyre::Result<bool> {
        async fn mtime(path: &Path) -> std::io::Result<Option<SystemTime>> {
            tokio::fs::metadata(path)
                .await
                .and_then(|m| m.modified())
                .map(Some)
                .or_else(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => Ok(None),
                    _ => Err(e),
                })
        }

        let mtimes = match future::try_join_all(
            std::iter::once(&output)
                .chain(inputs)
                .map(|path| mtime(path)),
        )
        .await
        {
            // rerun every time on platforms without any mtime functionality
            Err(e) if matches!(e.kind(), std::io::ErrorKind::Unsupported) => Ok(vec![None]),
            other => other,
        }
        .wrap_err("Aqiring file's last modification time")?;

        let input_mtimes = &mtimes[1..];
        let output_mtime = mtimes[0];

        let is_cached = output_mtime
            .map_or(Result::<_, usize>::Ok(false), |ot| {
                input_mtimes
                    .iter()
                    .enumerate()
                    .try_fold(true, |acc, (i, it)| Ok((*it).ok_or(i)? < ot && acc))
            })
            .map_err(|i| eyre!("Input file doesn't exist: {:?}", inputs[i]))?;

        tracing::debug!(is_cached);

        Ok(is_cached)
    }

    // TODO: title
    let title = elf
        .file_stem()
        .ok_or_else(|| eyre!("`elf` path doesn't have a file stem"))?
        .to_owned();

    // TODO: other arguments via cargo metadata

    tokio::try_join!(
        async {
            if !is_cached("vita-mksfoex", &[], &sfo).await? {
                toolchain::VitaMksfoex::new(&title, &sfo).run().await?;
            }
            eyre::Result::<()>::Ok(())
        },
        async {
            if !is_cached("vita-elf-create", &[&elf], &velf).await? {
                toolchain::VitaElfCreate::new(&elf, &velf).run().await?;
            }
            if !is_cached("vita-make-fself", &[&velf], &eboot_bin).await? {
                toolchain::VitaMakeFself::new(&velf, &eboot_bin)
                    .run()
                    .await?;
            }
            Ok(())
        }
    )?;

    if !is_cached("vita-pack-vpk", &[&sfo, &eboot_bin], &vpk).await? {
        toolchain::VitaPackVpk::new(&sfo, &eboot_bin, &vpk)
            .run()
            .await?;
    }

    Ok(())
}

fn handle_exit_status(exit_status: std::process::ExitStatus) -> eyre::Result<()> {
    if !exit_status.success() {
        tracing::error!(?exit_status, "Command failed");
        eyre::bail!("Command failed");
    }
    Ok(())
}
