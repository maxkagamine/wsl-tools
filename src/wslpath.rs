// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use core::str;
use std::{ffi::OsStr, io, process::Command};

fn wslpath<T: AsRef<OsStr>>(path: T, option: &str) -> io::Result<String> {
    let mut cmd = if cfg!(windows) {
        let mut cmd = Command::new("wsl.exe");
        cmd.arg("-e"); // Execute command without using the shell (avoids escaping issues)
        cmd.arg("wslpath");
        cmd
    } else {
        Command::new("wslpath")
    };

    let output = cmd.arg(option).arg("--").arg(path).output()?;

    if !output.status.success() {
        // wslpath prints the help text on stdout and the error message on stderr, which is helpful
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(io::Error::other(stderr));
    }

    let stdout = str::from_utf8(&output.stdout)
        .map_err(|_| io::Error::other("wslpath returned invalid UTF-8"))?
        .trim()
        .to_string();

    Ok(stdout)
}

/// Translates from a WSL path to a Windows path.
///
/// # Errors
///
/// Failed to execute `wslpath` or got invalid UTF-8.
pub fn to_windows<T: AsRef<OsStr>>(path: T) -> io::Result<String> {
    wslpath(path, "-aw")
}

/// Translates from a Windows path to a WSL path.
///
/// # Errors
///
/// Failed to execute `wslpath` or got invalid UTF-8.
pub fn to_wsl<T: AsRef<OsStr>>(path: T) -> io::Result<String> {
    wslpath(path, "-a")
}
