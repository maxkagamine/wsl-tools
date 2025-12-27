// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

/// Characters allowed in a path, per RFC 3986:
/// ```text
/// unreserved    = ALPHA / DIGIT / "-" / "." / "_" / "~"
///
/// sub-delims    = "!" / "$" / "&" / "'" / "(" / ")"
///                     / "*" / "+" / "," / ";" / "="
///
/// path-abempty  = *( "/" segment )
///
/// segment       = *pchar
///
/// pchar         = unreserved / pct-encoded / sub-delims / ":" / "@"
/// ```
const PATH: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'%')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

/// Builds a vscode-remote:// URI from `authority` (`"wsl+{distro_name}"`) and `wsl_path` (absolute).
#[must_use]
pub fn wsl_path_to_vscode_remote_uri(authority: &str, wsl_path: &str) -> String {
    let escaped_path = utf8_percent_encode(wsl_path, PATH).to_string();
    format!("vscode-remote://{authority}{escaped_path}")
}

/// Locates and returns the absolute path to Code.exe
///
/// # Errors
///
/// Failed to read the registry.
#[cfg(windows)]
pub fn get_vscode_exe() -> anyhow::Result<String> {
    use anyhow::Context;
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
                Some(&raw mut buffer_size),
            )
        } {
            ERROR_SUCCESS => {
                unsafe {
                    buffer.set_len(buffer_size as usize / 2);
                }
                let length = buffer
                    .iter()
                    .position(|&x| x == 0)
                    .context("pvData missing null-terminator")?;
                let value = String::from_utf16(&buffer[..length])?;
                return Ok(exe_from_command_string(&value));
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
fn exe_from_command_string(command: &str) -> String {
    let trimmed = command.trim_start();
    if trimmed.starts_with('"') {
        trimmed.chars().skip(1).take_while(|x| *x != '"').collect()
    } else {
        trimmed.chars().take_while(|x| *x != ' ').collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wsl_path_to_vscode_remote_uri_works() {
        assert_eq!(
            wsl_path_to_vscode_remote_uri(
                "wsl+Arch",
                "/home/max/foo/バー/リン%20& ちゃん [@is'cute].txt"
            ),
            // Actual command line arg observed while running `code` in wsl
            "vscode-remote://wsl+Arch/home/max/foo/%E3%83%90%E3%83%BC/%E3%83%AA%E3%83%B3%2520&%20%E3%81%A1%E3%82%83%E3%82%93%20%5B@is'cute%5D.txt"
        );
    }

    #[test]
    #[cfg(windows)]
    fn exe_from_command_string_works() {
        assert_eq!(
            exe_from_command_string(r#"C:\Windows\notepad.exe "%1""#),
            r"C:\Windows\notepad.exe"
        );

        assert_eq!(
            exe_from_command_string(r#" "C:\Program Files\Foo\bar.exe" /x "%1" "#),
            r"C:\Program Files\Foo\bar.exe"
        );
    }
}
