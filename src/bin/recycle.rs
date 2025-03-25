// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "recycle",
    bin_name = "recycle",
    about = "\
Sends the given files/directories to the Recycle Bin.

This will show a progress dialog and possibly prompts to delete permanently or continue as admin, \
error dialogs such as a file being in use, etc., the same as if the user had deleted the files \
from Explorer. Right click undo in Explorer is enabled as well, for consistency. This is due to \
Windows API limitations: it is not possible to recycle files without any dialogs without also \
risking the shell permanently deleting files. Therefore, this command \x1b[3mmust not\x1b[m be \
used in scripts where the user is not expecting it.",
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
    // IMPORTANT! Any new args added here MUST be replicated in the Linux main() below. (Clap
    // doesn't give us a way to stringify args.)
    #[arg(required(true), help = if cfg!(unix) {
        "Files/directories to recycle, relative to the current directory. Linux paths are \
        automatically converted to Windows paths."
    } else {
        "Files/directories to recycle, relative to the current directory."
    })]
    paths: Vec<String>,
    // TODO: Add the sink back in to enable --verbose logging
    // TODO: Add --rm to make recycle a drop-in replacement for rm
}

#[cfg(windows)]
fn main() {
    use wsl_tools::recycle_bin;

    let args = Args::parse();

    if let Err(err) = recycle_bin::recycle(args.paths) {
        eprintln!("recycle: {err}");
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn main() {
    use wsl_tools::{exe_command, exe_exec};

    let args = Args::parse();

    let mut cmd = exe_command!();

    if !args.paths.is_empty() {
        // TODO: Convert WSL paths to Windows paths
        cmd.arg("--").args(args.paths);
    }

    exe_exec!(cmd);
}
