pub mod toolchain;

use std::{env, path::PathBuf, process::Stdio};

use eyre::{eyre, WrapErr};
use once_cell::sync::Lazy;
use tokio::io::{self, AsyncBufReadExt};

const TARGET_SPEC_NAME: &str = "arm-vita-eabi";

static VITASDK: Lazy<PathBuf> = Lazy::new({
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

#[tracing::instrument]
pub async fn build<'e>(args: &[String]) -> eyre::Result<()> {
    Lazy::force(&VITASDK);

    let mut build = tokio::process::Command::new("xargo");
    build
        .args(&[
            "build",
            "--message-format=json-render-diagnostics",
            "--target",
        ])
        .arg(TARGET_SPEC_NAME)
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

        if let cargo_metadata::Message::CompilerArtifact(cargo_metadata::Artifact {
            executable: Some(executable),
            ..
        }) = message
        {
            tasks.push(tokio::spawn(postprocess_elf(executable.into())))
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

fn handle_exit_status(exit_status: std::process::ExitStatus) -> eyre::Result<()> {
    if !exit_status.success() {
        tracing::error!(?exit_status, "Command failed");
        eyre::bail!("Command failed");
    }
    Ok(())
}
