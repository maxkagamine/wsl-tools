<h1>
  <img src="rustuxdows.svg" height="80" align="left" />
  wsl-tools
  <br />
  <sup><sub>
    <a href="README.ja.md">日本語</a>
    &nbsp;│&nbsp;
    <a href="#install">Install</a>
    &nbsp;│&nbsp;
    <a href="#xsel"><code>xsel</code></a>
    &nbsp;│&nbsp;
    <a href="#recycle"><code>recycle</code></a>
    &nbsp;│&nbsp;
    <a href="#open"><code>open</code></a>
  </sub></sup>
</h1>

Clipboard (xsel), recycle, and open commands for WSL written in Rust. I created this after getting fed up with PowerShell-based solutions being slow and janky (e.g. not handling Unicode properly). Human-coded, as with all of my work.

The programs come with both Linux and Windows binaries: the former is used to translate paths / check pipes before passing things along to the exe to call the relevant winapis. The exe's aren't WSL-specific and can be used by themselves in batch scripts etc. if needed.

As a bonus, I've added two options to the installer (if you choose to use it):

1. Make launching VS Code from Explorer (either by double-clicking a file or "<img src="https://code.visualstudio.com/assets/apple-touch-icon.png" height="16" align="center" /> Open with Code" in the context menu) open in WSL. This uses a [small program](src/bin/code-wsl.rs) to do the path translation and launch VS Code the same way the `code` shell script does in WSL.

2. Add "Run" to the context menu of .sh files. Similarly uses a [helper program](src/bin/run-in-wsl.rs) to run wsl.exe, making sure the environment is the same as if you had run the script from a terminal while simultaneously avoiding quoting issues and respecting the shebang. _(Note: You can prevent Windows Terminal from closing automatically by changing Settings → Defaults → Advanced → Profile termination behavior.)_

## Install

> [!IMPORTANT]
> Exe's in the Linux filesystem can sometimes be 10x slower to execute than if they were on the Windows filesystem. I've found this to be especially pronounced on Windows 11. To make things easy, I've created an installer that copies the binaries to Program Files and adds them to your PATH.

### [Download Installer](https://github.com/maxkagamine/wsl-tools/releases/latest/download/wsl-tools-installer.exe) (recommended)

or [download zip](https://github.com/maxkagamine/wsl-tools/releases/latest/download/wsl-tools-portable.zip), or compile from source:

1. [Install Rust](https://rustup.rs/) (in WSL, not Windows)
2. [Install Inno Setup 6.6+](https://jrsoftware.org/isdl.php) (Optional)
3. Set up for cross-compilation:
   - Ubuntu: `sudo apt-get install mingw-w64 && rustup target add x86_64-pc-windows-gnu`
   - Arch: `sudo pacman -Syu mingw-w64 && rustup target add x86_64-pc-windows-gnu`
4. Run `make`

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

Recycles files & directories either interactively (as though deleted from Explorer) or as a replacement for `rm` (attempt recycle, nuke otherwise). See the [remarks in source](src/recycle_bin.rs) for details.

> [!TIP]
> You can alias `rm` to `recycle` in your .bashrc or similar to avoid accidentally deleting things in locations where recycling is possible:
> ```bash
> alias rm='recycle --rm'
> ```
> Shell scripts won't see the alias and should be modified to use `recycle --rm` if desired; for example:
> ```bash
> if command -v recycle &>/dev/null; then
>   rm() { recycle --rm "$@"; }
> fi
> ```
> Note that the `-i`/`-I` and `-d` options from [`rm`](https://linux.die.net/man/1/rm) are not currently implemented here.

> [!NOTE]
> If you get an "Element not found." error when trying to recycle files in the WSL filesystem, try `wsl.exe --update` and/or restarting. This appears to be a bug in WSL ([microsoft/WSL#12444](https://github.com/microsoft/WSL/issues/12444), [microsoft/WSL#11252](https://github.com/microsoft/WSL/issues/11252)).

```
Usage: recycle [OPTIONS] <PATHS>...

Sends the given files/directories to the Recycle Bin.

The default behavior (without --rm) is to let the shell display the normal
progress and confirmation dialogs and add to Explorer's undo history, the same
as if the user had deleted the files from Explorer. This is due to Windows API
limitations: it is not possible to recycle without any dialogs without also
risking the shell permanently deleting files. Consequently, this command MUST
NOT be used without --rm in scripts where the user is not expecting it.

Arguments:
  <PATHS>...
          Files/directories to recycle, relative to the current directory. Linux
          paths are automatically converted to Windows paths.

Options:
  -f, --force
          Ignore nonexistent files.
      --rm
          Hide all dialogs and let the shell permanently delete anything it
          can't recycle. Directories will produce an error unless --recursive.
          Files in the WSL filesystem will be deleted Linux-side.
          
          Warning: this may result in files that could have been recycled
          beingnuked instead; see comment in `recycle_bin.rs` for details.
  -r, --recursive
          Allow recycling and deleting directories when --rm is used. No effect
          without --rm (the shell will display a dialog instead).
  -v, --verbose
          Show recycle progress in the terminal.
  -h, --help
          Print help
  -V, --version
          Print version

```

## open

Simple wrapper for [ShellExecuteExW](https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw).

> [!IMPORTANT]
> Add `export BROWSER='open'` to your .bashrc or similar.
> 
> On Ubuntu/Debian, the [alternatives system](https://manpages.ubuntu.com/manpages/trusty/man8/update-alternatives.8.html) may be managing symlinks for /usr/bin/open and two commands used for opening links in the default browser, all of which can be pointed at wsl-tools instead:
> ```bash
> for cmd in open www-browser x-www-browser; do
>   sudo update-alternatives --install /usr/bin/$cmd $cmd '/mnt/c/Program Files/wsl-tools/open' 999
> done
> ```

> [!TIP]
> Running `open -e image.png` is equivalent to right-clicking on image.png and selecting Edit. By default this opens Paint; you can change it to your preferred image editor using Winaero Tweaker or by modifying this key in regedit: `HKEY_CLASSES_ROOT\SystemFileAssociations\image\shell\edit\command`

```
Usage: open [OPTIONS] <PATHS>...

Opens the given files or URLs in their default programs (directories open in
Explorer).

For details regarding --verb, see:
https://learn.microsoft.com/en-us/windows/win32/shell/launch#object-verbs

Arguments:
  <PATHS>...  Files, directories, and/or URLs to open. Linux paths are
              automatically converted to Windows paths.

Options:
  -e, --edit         Alias for --verb edit
      --runas        Alias for --verb runas
      --verb <VERB>  Verb to execute
  -h, --help         Print help
  -V, --version      Print version
```

## Legal stuff

Copyright © Max Kagamine  
Licensed under the [Apache License, Version 2.0](LICENSE.txt)

## Illegal stuff

[Pirates!](https://www.youtube.com/watch?v=NSZhIAfR6dA)
