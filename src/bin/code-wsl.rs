// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![windows_subsystem = "windows"]

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
    max_term_width = 80,
)]
struct Args {
    /// File or directory path.
    path: String,
}

#[cfg(windows)]
fn main() {
    use anyhow::{Context, Result};
    use std::process::Command;
    use windows::{
        Win32::UI::WindowsAndMessaging::{MB_OK, MessageBoxW},
        core::PCWSTR,
    };
    use wsl_tools::wslpath;

    let result = (|| -> Result<()> {
        let args = Args::try_parse()?;

        let vscode_exe = get_vscode_exe().context("Failed to locate Code.exe")?;
        let distro_name = get_distro_name().context("Failed to get distro name")?;
        let wsl_path = wslpath::to_wsl(&args.path).context("Failed to get WSL path")?;

        let mut cmd = Command::new(vscode_exe);
        cmd.arg("--remote");
        cmd.arg(format!("wsl+{distro_name}")); // The distro name can be left empty, but then it appears as a different "recent folder" if you also do `code .`
        cmd.arg(wsl_path);

        cmd.spawn().context("Failed to start VS Code")?;

        Ok(())
    })();

    if let Err(err) = result {
        unsafe {
            let message = format!("{err}");
            let mut message_u16 = message.encode_utf16().chain(Some(0)).collect::<Vec<_>>();

            MessageBoxW(
                None,
                PCWSTR::from_raw(message_u16.as_mut_ptr()),
                PCWSTR::null(),
                MB_OK,
            );
        }

        std::process::exit(1);
    }
}

#[cfg(unix)]
fn main() {
    unimplemented!();
}

#[cfg(windows)]
fn get_vscode_exe() -> anyhow::Result<String> {
    use anyhow::anyhow;
    use windows::{
        Win32::{
            Foundation::{ERROR_MORE_DATA, ERROR_SUCCESS},
            System::Registry::{
                HKEY_CLASSES_ROOT, RRF_RT_REG_EXPAND_SZ, RRF_RT_REG_SZ, RegGetValueW,
            },
        },
        core::{Error, PCWSTR, w},
    };

    let mut buffer_size: u32 = 512;
    let mut buffer: Vec<u16> = Vec::with_capacity(buffer_size as usize / 2);
    loop {
        match unsafe {
            RegGetValueW(
                HKEY_CLASSES_ROOT,
                w!(r"Applications\Code.exe\shell\open\command"),
                PCWSTR::null(),
                RRF_RT_REG_EXPAND_SZ | RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr().cast()),
                Some(&mut buffer_size),
            )
        } {
            ERROR_SUCCESS => {
                unsafe {
                    buffer.set_len(buffer_size as usize);
                }
                let length = buffer.iter().position(|&x| x == 0).unwrap_or(buffer.len());
                let value = String::from_utf16(&buffer[..length])?;
                return match value.split('"').find(|x| !x.is_empty()) {
                    Some(x) => Ok(x.to_string()),
                    None => Err(anyhow!("Unexpected value in registry.")),
                };
            }
            ERROR_MORE_DATA => {
                buffer.reserve((buffer_size as usize / 2) - buffer.len());
            }
            err => {
                return Err(Error::from_hresult(err.to_hresult()).into());
            }
        }
    }
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
