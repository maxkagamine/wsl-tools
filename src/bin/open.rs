// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "open",
    bin_name = "open",
    about = "\
Opens the given files or URLs in their default programs (directories open in Explorer).

For details regarding \x1b[1m--verb\x1b[m, see:
\x1b[4;34mhttps://learn.microsoft.com/en-us/windows/win32/shell/launch#object-verbs\x1b[m
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
        "Files, directories, and/or URLs to open. Linux paths are automatically converted to \
        Windows paths."
    } else {
        "Files, directories, and/or URLs to open."
    })]
    paths: Vec<String>,

    #[arg(short, long, help = "Alias for \x1b[1m--verb edit\x1b[m", conflicts_with_all(["runas", "verb"]))]
    edit: bool,

    #[arg(long, help = "Alias for \x1b[1m--verb runas\x1b[m", conflicts_with_all(["edit", "verb"]))]
    runas: bool,

    /// Verb to execute.
    #[arg(long, conflicts_with_all(["edit", "runas"]))]
    verb: Option<String>,
}

#[cfg(windows)]
fn main() {
    use wsl_tools::process::shell_execute;

    let args = Args::parse();

    let verb: Option<&str> = if args.edit {
        Some("edit")
    } else if args.runas {
        Some("runas")
    } else {
        args.verb.as_deref()
    };

    for path in args.paths {
        if let Err(err) = shell_execute(&path, verb) {
            eprintln!("open: Failed to open \"{path}\": {err:#}");
            std::process::exit(1);
        }
    }
}

#[cfg(unix)]
fn main() {
    use wsl_tools::{exe_command, exe_exec, wslpath};

    let args = Args::parse();

    let mut cmd = exe_command!();

    if let Some(verb) = args.verb {
        cmd.arg("--verb").arg(&verb);
    }

    if args.edit {
        cmd.arg("--edit");
    }

    if args.runas {
        cmd.arg("--runas");
    }

    if !args.paths.is_empty() {
        cmd.arg("--");

        // Convert WSL paths to Windows paths
        for path in args.paths {
            if is_url(&path) {
                cmd.arg(&path);
                continue;
            }

            match wslpath::to_windows(&path) {
                Ok(x) => {
                    cmd.arg(x);
                }
                Err(err) => {
                    eprintln!("open: failed to execute wslpath on \"{path}\": {err}");
                    std::process::exit(1);
                }
            }
        }
    }

    exe_exec!(cmd);
}

#[cfg(unix)]
fn is_url(path: &str) -> bool {
    for c in path.chars() {
        if c == ':' {
            return true;
        }
        if std::path::is_separator(c) {
            return false;
        }
    }
    false
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[test]
    fn is_url_works() {
        assert!(
            is_url("https://youtu.be/hSHxPPV2zKU?list=PLYooEAFUfhDfevWFKLa7gh3BogBUAebYO"),
            "a url is a url"
        );

        assert!(
            is_url("mailto:example@example.com"),
            "protocols don't need the slash-slash"
        );

        assert!(!is_url(r"foo/bar"), "a path is not a url");
        assert!(!is_url("file.txt"), "a filename is not a url");
    }
}
