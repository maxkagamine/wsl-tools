<h1>
  <img src="rustuxdows.svg" height="80" align="left" />
  wsl-tools
  <br />
  <sup><sub>
    <a href="#install">Install</a>
    &nbsp;│&nbsp;
    <a href="README.ja.md">日本語</a>
    &nbsp;│&nbsp;
    <a href="#xsel"><code>xsel</code></a>
    &nbsp;│&nbsp;
    <a href="#recycle"><code>recycle</code></a>
  </sub></sup>
</h1>

Clipboard (xsel) & recycle commands for WSL written in Rust. I created this after getting fed up with PowerShell-based solutions being slow and janky (e.g. not handling Unicode properly). Human-coded, as with all of my work.

The programs come with both Linux and Windows binaries: the former is used to translate paths / check pipes before passing things along to the exe to call the relevant winapis. The exe's aren't WSL-specific and can be used by themselves e.g. in batch scripts if needed.

## Install

> [!IMPORTANT]
> Exe's in the Linux filesystem can sometimes be 10x slower to execute than if they were on the Windows filesystem. I've found this to be especially pronounced on Windows 11. To make things easy, I've created an installer that copies the binaries to Program Files and adds them to your PATH.

### [Download Installer](https://github.com/maxkagamine/wsl-tools/releases/latest/download/wsl-tools-installer.exe) (recommended)

or [download zip](https://github.com/maxkagamine/wsl-tools/releases/latest/download/wsl-tools-portable.zip), or compile from source:

- [Install Rust](https://rustup.rs/) (in WSL, not Windows)
- [Install Inno Setup 6](https://jrsoftware.org/isdl.php) (optional)
- Set up for cross-compilation:
  - Ubuntu: `sudo apt-get install mingw-w64 && rustup target add x86_64-pc-windows-gnu`
  - Arch: `sudo pacman -Syu mingw-w64 && rustup target add x86_64-pc-windows-gnu`
- Run `make`

## xsel

Drop-in replacement for the common xsel program used to copy/paste in Linux. Since many programs and clipboard libraries look for xsel, having this in PATH will allow those that aren't WSL-aware to copy to the Windows clipboard.

```
Usage: xsel [options]

By default the clipboard is output if both standard input and standard output
are terminals (ttys). Otherwise, the clipboard is output if standard output is
not a terminal (tty), and the clipboard is set from standard input if standard
input is not a terminal (tty). If any input or output options are given then the
program behaves only in the requested mode.

If both input and output is required then the previous clipboard is output
before being replaced by the contents of standard input.

Input options
  -a, --append            Append standard input to the clipboard
  -f, --follow            <Not supported>
  -z, --zeroflush         <Not supported>
  -i, --input             Read standard input into the clipboard

Output options
  -o, --output            Write the clipboard to standard output

  --keep-crlf             <Windows-only addition> By default, CRLF is replaced
                          with LF when pasting. Pass this option to disable.

Action options
  -c, --clear             Clear the clipboard
  -d, --delete            <Not supported>

Selection options
  -p, --primary           PRIMARY and SECONDARY selections have no equivalent
  -s, --secondary         on Windows, but since some Linux clipboard managers
  -b, --clipboard         sync the selection and clipboard buffers, we pretend
                          that's the case and disregard the chosen selection.

  -k, --keep              <No-op>
  -x, --exchange          <No-op>

X options
  --display               <Not supported>
  -m, --name              <Not supported>
  -t, --selectionTimeout  <Not supported>

Miscellaneous options
  --trim                  Remove newline from end of input / output
  -l, --logfile           <Not supported>
  -n, --nodetach          <Ignored>
  -h, --help              Display this help and exit
  -v, --verbose           <Ignored>
  --version               Output version information and exit
```

## recycle

See [remarks in source](src/recycle_bin.rs).

```
Usage: recycle [OPTIONS] <PATHS>...

Sends the given files/directories to the Recycle Bin.

The default behavior (without --rm) is to let the shell display the normal
progress and confirmation dialogs and add to Explorer's undo history, the same
as if the user had deleted the files in Explorer. This is due to Windows API
limitations: it is not possible to recycle files without any dialogs without
also risking the shell permanently deleting files. Consequently, this command
MUST NOT be used without --rm in scripts where the user is not expecting it.

Arguments:
  <PATHS>...
          Files/directories to recycle, relative to the current directory. Linux
          paths are automatically converted to Windows paths.

Options:
  -f, --force
          Ignore nonexistent files

      --rm
          Hide all dialogs and let the shell permanently delete anything it
          can't recycle. Warnings:

          • This may result in files that could have been recycled being nuked
          instead; see comment in `recycle_bin.rs` for details.

          • Directories will be deleted recursively.

          • Files in the WSL filesystem that require sudo cannot be deleted with
          `recycle` (Explorer won't do it).

  -v, --verbose
          Show recycle progress in the terminal

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Legal stuff

Copyright © Max Kagamine  
Licensed under the [Apache License, Version 2.0](LICENSE.txt)

## Illegal stuff

[Pirates!](https://www.youtube.com/watch?v=NSZhIAfR6dA)
