// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0
//
// Based on xsel 1.2.1 by Conrad Parker

use clap::Parser;
use std::io::IsTerminal;

#[derive(Parser)]
#[command(
    name = "xsel",
    bin_name = "xsel",
    override_help = "\
Usage: xsel [options]

By default the clipboard is output if both standard input and standard output
are terminals (ttys). Otherwise, the clipboard is output if standard output is
not a terminal (tty), and the clipboard is set from standard input if standard
input is not a terminal (tty). If any input or output options are given then the
program behaves only in the requested mode.

If both input and output is required then the previous clipboard is output
before being replaced by the contents of standard input.

\x1b[1;4mInput options\x1b[m
  -a, --append            Append standard input to the clipboard
  -f, --follow            <Not supported>
  -z, --zeroflush         <Not supported>
  -i, --input             Read standard input into the clipboard

\x1b[1;4mOutput options\x1b[m
  -o, --output            Write the clipboard to standard output

  --keep-crlf             <Windows-only addition> By default, CRLF is replaced
                          with LF when pasting. Pass this option to disable.

\x1b[1;4mAction options\x1b[m
  -c, --clear             Clear the clipboard
  -d, --delete            <Not supported>

\x1b[1;4mSelection options\x1b[m
  -p, --primary           PRIMARY and SECONDARY selections have no equivalent
  -s, --secondary         on Windows, but since some Linux clipboard managers
  -b, --clipboard         sync the selection and clipboard buffers, we pretend
                          that's the case and disregard the chosen selection.

  -k, --keep              <No-op>
  -x, --exchange          <No-op>

\x1b[1;4mX options\x1b[m
  --display               <Not supported>
  -m, --name              <Not supported>
  -t, --selectionTimeout  <Not supported>

\x1b[1;4mMiscellaneous options\x1b[m
  --trim                  Remove newline from end of input / output
  -l, --logfile           <Not supported>
  -n, --nodetach          <Ignored>
  -h, --help              Display this help and exit
  -v, --verbose           <Ignored>
  --version               Output version information and exit",
    version = concat!(clap::crate_version!(), "
Copyright (c) Max Kagamine
Licensed under the Apache License, Version 2.0

Based on xsel 1.2.1 by Conrad Parker

https://github.com/maxkagamine/wsl-tools"),
)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    #[arg(short, long)]
    append: bool,
    #[arg(short, long)]
    input: bool,
    #[arg(short, long)]
    output: bool,
    #[arg(long)]
    keep_crlf: bool,
    #[arg(short, long)]
    clear: bool,
    #[arg(short, long)]
    primary: bool,
    #[arg(short, long)]
    secondary: bool,
    #[arg(short = 'b', long)]
    clipboard: bool,
    #[arg(short, long)]
    keep: bool,
    #[arg(short = 'x', long)]
    exchange: bool,
    #[arg(long)]
    trim: bool,
    #[arg(short, long)]
    verbose: bool,

    // When running a Windows exe from WSL, if any pipes are redirected Linux-side, *all* of them
    // will be redirected Windows-side, which means xsel.exe won't be able to tell which are ttys.
    // To solve this, a Linux binary is run first which passes these hidden flags to the exe.
    #[arg(long)]
    stdin_is_tty: Option<bool>,
    #[arg(long)]
    stdout_is_tty: Option<bool>,
}

#[cfg(windows)]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Read;

    let args = Args::parse();

    let stdin_is_tty = args
        .stdin_is_tty
        .unwrap_or_else(|| std::io::stdin().is_terminal());
    let stdout_is_tty = args
        .stdout_is_tty
        .unwrap_or_else(|| std::io::stdout().is_terminal());

    // Determine input/output behavior based on options and pipes
    let do_input = args.append || args.input || (!args.output && !stdin_is_tty);
    let do_output = args.output
        || (!args.append && !args.input && !args.clear && (!stdout_is_tty || stdin_is_tty));

    if args.keep || args.exchange {
        // No-op
        return Ok(());
    }

    let old_sel = if do_output {
        get_clipboard(&args)?.inspect(|x| print!("{x}"))
    } else {
        None
    };

    if args.clear {
        wsl_tools::clipboard::clear()?;
    } else if do_input {
        let mut new_sel = if !args.append {
            String::new()
        } else if do_output {
            old_sel.unwrap_or_default()
        } else {
            get_clipboard(&args)?.unwrap_or_default()
        };

        std::io::stdin().read_to_string(&mut new_sel)?;

        let text = if args.trim {
            new_sel.trim_end_matches(['\r', '\n'])
        } else {
            new_sel.as_ref()
        };

        wsl_tools::clipboard::set_text(text)?;
    }

    Ok(())
}

#[cfg(windows)]
fn get_clipboard(args: &Args) -> Result<Option<String>, windows::core::Error> {
    Ok(wsl_tools::clipboard::get_text()?.map(|text| {
        if args.keep_crlf {
            if args.trim {
                text.trim_end_matches(['\r', '\n']).to_string()
            } else {
                text
            }
        } else if args.trim {
            text.trim_end_matches(['\r', '\n']).replace("\r\n", "\n")
        } else {
            text.replace("\r\n", "\n")
        }
    }))
}

#[cfg(unix)]
fn main() {
    use std::{io::ErrorKind, os::unix::process::ExitStatusExt, process::Command};

    const EXE: &str = "xsel.exe";

    let stdin_is_tty = std::io::stdin().is_terminal();
    let stdout_is_tty = std::io::stdout().is_terminal();

    let mut cmd = if cfg!(debug_assertions) {
        let mut cmd = Command::new("cargo");
        cmd.arg("run")
            .arg("--target=x86_64-pc-windows-gnu")
            .arg("--bin=xsel")
            .arg("--");
        cmd
    } else {
        Command::new(EXE)
    };

    cmd.args(std::env::args_os().skip(1))
        .arg(format!("--stdin-is-tty={stdin_is_tty}"))
        .arg(format!("--stdout-is-tty={stdout_is_tty}"));

    let status = cmd.status();

    std::process::exit(match status {
        Err(err) => {
            if err.kind() == ErrorKind::NotFound {
                eprintln!("xsel: could not find '{EXE}'");
                127
            } else {
                eprintln!("xsel: failed to start '{EXE}': {err:?}");
                126
            }
        }
        Ok(status) => {
            if let Some(code) = status.code() {
                code
            } else {
                eprintln!("xsel: '{EXE}' exited with {status}");
                status.signal().unwrap_or_default().saturating_add(128)
            }
        }
    });
}
