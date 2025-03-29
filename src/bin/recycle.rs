// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "recycle",
    bin_name = "recycle",
    about = "\
Sends the given files/directories to the Recycle Bin.

The default behavior (without --rm) is to let the shell display the normal progress and \
confirmation dialogs and add to Explorer's undo history, the same as if the user had deleted the \
files in Explorer. This is due to Windows API limitations: it is not possible to recycle files \
without any dialogs without also risking the shell permanently deleting files. Consequently, this \
command MUST NOT be used without --rm in scripts where the user is not expecting it.\
",
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

    /// Ignore nonexistent files.
    #[arg(short, long)]
    force: bool,

    /// Hide all dialogs and let the shell permanently delete anything it can't recycle. Warnings:
    ///
    /// • This may result in files that could have been recycled being nuked instead; see comment in
    ///   `recycle_bin.rs` for details.
    ///
    /// • Directories will be deleted recursively.
    ///
    /// • Files in the WSL filesystem that would require sudo will silently fail to delete (same
    ///   happens in Explorer).
    #[arg(long)]
    rm: bool,

    /// Show recycle progress in the terminal.
    #[arg(short, long)]
    verbose: bool,
}

#[cfg(windows)]
fn main() {
    use wsl_tools::recycle_bin::{
        self, RECYCLE_DANGEROUSLY_IN_BACKGROUND, RECYCLE_IGNORE_NOT_FOUND, RecycleOptions,
    };

    let args = Args::parse();

    let mut options: RecycleOptions = 0;

    if args.force {
        options |= RECYCLE_IGNORE_NOT_FOUND;
    }

    if args.rm {
        options |= RECYCLE_DANGEROUSLY_IN_BACKGROUND;
    }

    let result = if args.verbose {
        recycle_bin::recycle_with_callback(&args.paths, options, |item, err| match err {
            // There's no way to know for sure if the item was actually recycled or deleted
            // permanently (`dwflags` can lie), so our verbage here should reflect that.
            None => println!("recycle: removed \"{item}\""),
            Some(e) => eprintln!("recycle: failed to recycle \"{item}\": {e}"),
        })
    } else {
        recycle_bin::recycle(&args.paths, options)
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

    if args.force {
        cmd.arg("--force");
    }

    if args.rm {
        cmd.arg("--rm");
    }

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
