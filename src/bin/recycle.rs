// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use clap::Parser;
use std::fs;

#[derive(Parser)]
#[command(
    name = "recycle",
    bin_name = "recycle",
    about = "\
Sends the given files/directories to the Recycle Bin.

The default behavior (without --rm) is to let the shell display the normal progress and \
confirmation dialogs and add to Explorer's undo history, the same as if the user had deleted the \
files from Explorer. This is due to Windows API limitations: it is not possible to recycle without \
any dialogs without also risking the shell permanently deleting files. Consequently, this command \
MUST NOT be used without --rm in scripts where the user is not expecting it.\
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
    next_line_help = true,
)]
#[allow(clippy::struct_excessive_bools)]
#[rustfmt::skip]
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

    #[arg(short, long, help = "Ignore nonexistent files.")]
    force: bool,

    #[arg(long, help = if cfg!(unix) {
        "Hide all dialogs and let the shell permanently delete anything it can't recycle. \
        Directories will produce an error unless --recursive. Files in the WSL filesystem will be \
        deleted Linux-side.\n\nWarning: this may result in files that could have been recycled \
         beingnuked instead; see comment in `recycle_bin.rs` for details."
    } else {
        "Hide all dialogs and let the shell permanently delete anything it can't recycle. \
        Directories will produce an error unless --recursive. Note that files in the WSL \
        filesystem which require sudo cannot be deleted with `recycle.exe` (Explorer won't do \
        it).\n\nWarning: this may result in files that could have been recycled being nuked \
        instead; see comment in `recycle_bin.rs` for details."
    })]
    rm: bool,

    // Clap can't show aliases on the same line: https://github.com/clap-rs/clap/issues/5459
    #[arg(short, short_alias = 'R', long, help = "\
        Allow recycling and deleting directories when --rm is used. No effect without --rm (the \
        shell will display a dialog instead).")]
    recursive: bool,

    #[arg(short, long, help = "Show recycle progress in the terminal.")]
    verbose: bool,
}

#[cfg(windows)]
fn main() {
    use wsl_tools::recycle_bin::{
        self, RECYCLE_DANGEROUSLY_IN_BACKGROUND, RECYCLE_IGNORE_NOT_FOUND, RecycleError,
        RecycleOptions,
    };

    let args = Args::parse();

    let mut options: RecycleOptions = 0;

    if args.force {
        options |= RECYCLE_IGNORE_NOT_FOUND;
    }

    if args.rm {
        options |= RECYCLE_DANGEROUSLY_IN_BACKGROUND;

        // Make sure we won't be deleting any directories if not --recursive
        if !args.recursive {
            for path in &args.paths {
                // Don't follow symlinks
                if fs::symlink_metadata(path).is_ok_and(|m| m.is_dir()) {
                    eprintln!("recycle: Cannot remove \"{path}\": Is a directory.");
                    std::process::exit(1);
                }
            }
        }
    }

    let mut errored = false;
    let result = recycle_bin::recycle_with_callback(&args.paths, options, |item, err| match err {
        None => {
            if args.verbose {
                // There's no way to know for sure if the item was actually recycled or deleted
                // permanently (`dwflags` can lie), so our verbage here should reflect that.
                println!("recycle: Removed \"{item}\"");
            }
        }
        Some(e) => {
            errored = true;
            eprintln!("recycle: Failed to recycle \"{item}\": {e}");
        }
    });

    if let Err(err) = result {
        // No need to print "The operation was canceled." if there were specific errors.
        if !matches!(err, RecycleError::Canceled) || !errored {
            eprintln!("recycle: {err}");
        }

        std::process::exit(1);
    }
}

#[cfg(unix)]
fn main() {
    use std::{cell::LazyCell, fs::Metadata, io::ErrorKind, os::linux::fs::MetadataExt};
    use wsl_tools::{exe_command, exe_exec, wslpath};

    let args = Args::parse();

    let mut cmd = exe_command!();

    if args.force {
        cmd.arg("--force");
    }

    if args.rm {
        cmd.arg("--rm");
    }

    if args.recursive {
        cmd.arg("--recursive");
    }

    if args.verbose {
        cmd.arg("--verbose");
    }

    let root_dev_inode: LazyCell<Metadata> = LazyCell::new(|| fs::symlink_metadata("/").unwrap());
    let mut any_to_recycle = false;

    if !args.paths.is_empty() {
        cmd.arg("--");

        // Convert WSL paths to Windows paths. The `symlink_to_windows` function runs wslpath on the
        // dirname and then appends the basename, as wslpath resolves symlinks when converting from
        // WSL to Windows paths (this is hardcoded in the source and unfortunately can't be
        // overridden: https://github.com/microsoft/WSL/blob/2.7.0/src/linux/init/wslpath.cpp#L428).
        for path in args.paths {
            match wslpath::symlink_to_windows(&path) {
                Ok(x) if args.rm && x.starts_with(r"\\wsl.localhost\") => {
                    // For paths in the WSL filesystem, we can unlink them here. If --rm wasn't
                    // given, we'll skip this so that the shell can display a dialog. Note that we
                    // use symlink_metadata (lstat) here instead of metadata (stat) to be consistent
                    // with `rm` and not follow symlinks (even if --recursive).
                    let stat = match fs::symlink_metadata(&path) {
                        Ok(m) => m,
                        Err(err) if err.kind() == ErrorKind::NotFound => {
                            if args.force {
                                continue;
                            }
                            eprintln!(
                                "recycle: Failed to delete \"{path}\": No such file or directory."
                            );
                            std::process::exit(1);
                        }
                        Err(err) => {
                            eprintln!("recycle: Failed to stat \"{path}\": {err}");
                            std::process::exit(1);
                        }
                    };

                    // https://github.com/coreutils/coreutils/blob/master/src/rm.c
                    if stat.st_dev() == root_dev_inode.st_dev()
                        && stat.st_ino() == root_dev_inode.st_ino()
                    {
                        if path == "/" {
                            eprintln!("recycle: Refusing to delete \"/\".");
                        } else {
                            eprintln!("recycle: Refusing to delete \"{path}\" (same as \"/\").");
                        }
                        std::process::exit(1);
                    }

                    let result = if stat.is_dir() {
                        if !args.recursive {
                            eprintln!("recycle: Cannot remove \"{path}\": Is a directory.");
                            std::process::exit(1);
                        }
                        fs::remove_dir_all(&path)
                    } else {
                        fs::remove_file(&path)
                    };

                    match result {
                        Ok(()) => {
                            if args.verbose {
                                println!("recycle: Removed \"{path}\"");
                            }
                        }
                        Err(err) if err.kind() == ErrorKind::NotFound => {
                            if args.force {
                                continue;
                            }
                            eprintln!(
                                "recycle: Failed to delete \"{path}\": No such file or directory."
                            );
                            std::process::exit(1);
                        }
                        Err(err) => {
                            eprintln!("recycle: Failed to delete \"{path}\": {err}");
                            std::process::exit(1);
                        }
                    }
                }
                Ok(x) => {
                    cmd.arg(x);
                    any_to_recycle = true;
                }
                Err(err) => {
                    eprintln!("recycle: Failed to execute wslpath on \"{path}\": {err}");
                    std::process::exit(1);
                }
            }
        }
    }

    if any_to_recycle {
        exe_exec!(cmd);
    }
}
