// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg_attr(not(test), windows_subsystem = "windows")]

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "run-in-wsl",
    bin_name = "run-in-wsl",
    about = "\
Executes the given Windows file path in WSL, using its containing folder as the current directory.

This will:
1. Run the user's default shell and load its profile (.bashrc etc.), so that PATH and
    other env vars are set the same as when running the script from a terminal
2. Run scripts in a non-interactive, non-login shell (again, same as normal)
3. Respect shebangs (doesn't need to be a bash script)
4. Not choke on spaces, quotes, bangs, dollar signs, emoji, etc. in the filename",
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
    /// File path.
    path: String,
}

#[cfg(windows)]
fn main() {
    use anyhow::{Context, Result, anyhow};
    use shell_escape;
    use std::{os::windows::process::CommandExt, process::Command};
    use wsl_tools::message_box;

    std::panic::set_hook(Box::new(|info| {
        message_box::show(info.to_string(), None::<&str>, None);
        std::process::exit(1);
    }));

    let result = (|| -> Result<()> {
        let args = Args::try_parse()?;
        let path = std::fs::canonicalize(&args.path)?;

        if !path.is_file() {
            return Err(anyhow!("\"{}\" is not a file.", &args.path));
        }

        // These can't panic since we know it's a path to a file. Don't even need to use wslpath;
        // WSL will take care of that for us just by setting the current directory.
        let dir = path.parent().unwrap();
        let file = path
            .file_name()
            .unwrap()
            .to_str()
            .ok_or_else(|| anyhow!("Filename not valid UTF-8."))?;

        // Test file for comparing ways of running scripts:
        //   #!/bin/bash
        //   env; echo
        //   [[ $- == *i* ]] && echo 'Interactive' || echo 'Not interactive'
        //   shopt -q login_shell && echo 'Login shell' || echo 'Not login shell'

        let mut cmd = Command::new("wsl.exe");
        cmd.current_dir(dir);
        cmd.arg("--shell-type");
        cmd.arg("login");
        cmd.arg("--");

        // wsl.exe passes the remainder of the command line string verbatim (remember that Windows
        // arguments aren't arrays; Rust's .arg() is doing Windows-style quoting/escaping):
        // https://github.com/microsoft/WSL/blob/master/src/windows/common/WslClient.cpp
        cmd.raw_arg(shell_escape::unix::escape(format!("./{file}").into()).into_owned());

        cmd.spawn().context("Error running wsl.exe")?;

        Ok(())
    })();

    if let Err(err) = result {
        message_box::show(format!("{err:?}"), None::<&str>, None);
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn main() {
    unimplemented!();
}
