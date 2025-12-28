// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

/// Evaluates to a `Command` which executes the Windows version of the Linux binary being compiled,
/// using `{bin}.exe` in release and `cargo run` in debug.
///
/// # Examples
///
/// ```ignore
/// let mut cmd = exe_command!();
/// cmd.arg("--foo");
/// exe_exec!(cmd);
/// ```
#[macro_export]
#[cfg(unix)]
macro_rules! exe_command {
    () => {
        if cfg!(debug_assertions) {
            let mut cmd = std::process::Command::new("cargo");
            cmd.arg("run")
                .arg("--target=x86_64-pc-windows-gnu")
                .arg(concat!("--bin=", env!("CARGO_BIN_NAME")))
                .arg("--");
            cmd
        } else {
            let mut exe = std::env::current_exe().unwrap();
            exe.add_extension("exe");
            std::process::Command::new(exe)
        }
    };
}

/// Runs the command and exits, propagating its exit code. If the command fails to execute or is
/// killed, echoes an appropriate error message to stderr and exits with a code the shell would use.
#[macro_export]
#[cfg(unix)]
macro_rules! exe_exec {
    ($cmd:ident) => {
        use std::{io::ErrorKind, os::unix::process::ExitStatusExt};

        let name = env!("CARGO_BIN_NAME");

        std::process::exit(match $cmd.status() {
            Err(err) => {
                if err.kind() == ErrorKind::NotFound {
                    eprintln!("{name}: could not find '{name}.exe'");
                    127
                } else {
                    eprintln!("{name}: failed to start '{name}.exe': {err}");
                    126
                }
            }
            Ok(status) => {
                if let Some(code) = status.code() {
                    code
                } else {
                    eprintln!("{name}: '{name}.exe' exited with {status}");
                    status.signal().unwrap_or_default().saturating_add(128)
                }
            }
        });
    };
}
