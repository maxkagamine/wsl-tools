// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg_attr(not(test), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "code-wsl",
    bin_name = "code-wsl",
    about = "Opens the given Windows path with VS Code in WSL.",
    version = concat!(clap::crate_version!(), "
Copyright (c) Max Kagamine
Licensed under the Apache License, Version 2.0

https://github.com/maxkagamine/wsl-tools"),
    help_template = "\
{usage-heading} {usage}
{about-section}
{all-args}",
    term_width = 0,
)]
struct Args {
    /// File or directory path.
    path: String,
}

#[cfg(windows)]
fn main() {
    use anyhow::{Context, Result};
    use std::{path::Path, process::Command};
    use wsl_tools::{message_box, vscode, wslpath};

    std::panic::set_hook(Box::new(|info| {
        message_box::show(info.to_string(), None, None);
        std::process::exit(1);
    }));

    let result = (|| -> Result<()> {
        let args = Args::try_parse()?;

        let vscode_exe = vscode::get_vscode_exe().context("Failed to locate Code.exe")?;
        let distro_name = get_distro_name().context("Failed to get distro name")?;

        let mut cmd = Command::new(vscode_exe);

        let Ok(wsl_path) = wslpath::to_wsl(&args.path) else {
            // If wslpath fails, we'll assume it's because the location isn't mounted in WSL and
            // open it in a local VS Code instead.
            //
            // TODO: Offer to mount the drive / network share automatically (see #3)
            cmd.arg(&args.path);
            cmd.spawn().context("Failed to start VS Code")?;
            return Ok(());
        };

        // This is what the remote extension's wslCode.sh script calls it. The distro name can be
        // left empty, but then it appears as a different "recent folder" if you also do `code .`
        let authority = format!("wsl+{distro_name}");
        let uri = vscode::wsl_path_to_vscode_remote_uri(&authority, &wsl_path);

        cmd.arg("--remote");
        cmd.arg(authority);

        // The --file-uri and --folder-uri options appear to be undocumented (found them by running
        // Procmon to see how the WSL `code` script launched VS Code), but without them we run into
        // an issue where the remote extension can't distinguish between files and directories. If
        // you run `Code.exe --remote wsl+Arch Makefile` for example, it'll try to open it as a
        // directory because it doesn't have a dot in the filename.
        if Path::new(&args.path).is_dir() {
            cmd.arg("--folder-uri");
        } else {
            cmd.arg("--file-uri");
        }

        cmd.arg(uri);

        cmd.spawn().context("Failed to start VS Code")?;

        Ok(())
    })();

    if let Err(err) = result {
        message_box::show(format!("{err:?}"), None, None);
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn main() {
    unimplemented!();
}

#[cfg(windows)]
fn get_distro_name() -> anyhow::Result<String> {
    use std::{os::windows::process::CommandExt, process::Command};
    use windows::Win32::System::Threading::CREATE_NO_WINDOW;

    let output = Command::new("wsl.exe")
        .creation_flags(CREATE_NO_WINDOW.0)
        .arg("echo")
        .arg("$WSL_DISTRO_NAME")
        .output()?;
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}
