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
from Explorer. Right click undo in Explorer is enabled as well for consistency. This is due to \
Windows API limitations: it is not possible to recycle files without any dialogs without also \
risking the shell permanently deleting files. Consequently, this command \x1b[3mmust not\x1b[m be \
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
    //
    #[arg(required(true), help = if cfg!(unix) {
        "Files/directories to recycle, relative to the current directory. Linux paths are \
        automatically converted to Windows paths."
    } else {
        "Files/directories to recycle, relative to the current directory."
    })]
    paths: Vec<String>,

    /// Show recycle progress in the terminal.
    #[arg(short, long)]
    verbose: bool,
}

#[cfg(windows)]
fn main() {
    use wsl_tools::recycle_bin::{self, RECYCLE_NORMAL};

    let args = Args::parse();

    let result = if args.verbose {
        recycle_bin::recycle_with_callback(&args.paths, RECYCLE_NORMAL, |item, err| match err {
            // There's no way to know for sure if the item was actually recycled or deleted
            // permanently (`dwflags` can lie), so our verbage here should reflect that.
            None => println!("recycle: removed \"{item}\""),
            Some(e) => eprintln!("recycle: failed to recycle \"{item}\": {e}"),
        })
    } else {
        recycle_bin::recycle(&args.paths, RECYCLE_NORMAL)
    };

    if let Err(err) = result {
        eprintln!("recycle: {err}");
        std::process::exit(1);
    }
}

#[cfg(unix)]
fn main() {
    use wsl_tools::{exe_command, exe_exec, wslpath};

    let args = Args::parse();

    let mut cmd = exe_command!();

    if args.verbose {
        cmd.arg("--verbose");
    }

    if !args.paths.is_empty() {
        cmd.arg("--");

        // Convert WSL paths to Windows paths
        for path in args.paths {
            match wslpath::to_windows(&path) {
                Ok(x) => {
                    cmd.arg(x);
                }
                Err(err) => {
                    eprintln!("recycle: failed to execute wslpath on \"{path}\": {err}");
                    std::process::exit(1);
                }
            }
        }
    }

    exe_exec!(cmd);
}
