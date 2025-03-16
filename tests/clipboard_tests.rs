// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use assert_cmd::prelude::*;
use base64::engine::{Engine as _, general_purpose::STANDARD as base64};
use std::process::Command;

// The whole reason we're making this is to *avoid* using PowerShell, but we still need some way to
// test our implementation. Bypassing the cmdlets and using .NET/winforms instead, and in particular
// base64-encoding everything so the shell can't mess up Unicode text (due to Windows system-wide
// region settings), is the only reliable method I've found for using PowerShell to interact with
// the clipboard -- although there are obvious length limitations with inlining the base64 text into
// the command argument as I'm doing here (not to mention it's quite slow).
fn set_clipboard_via_powershell<T: AsRef<[u8]>>(text: T) {
    let base64str = base64.encode(text);

    Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(format!(
            r#"
            Add-Type -AssemblyName System.Windows.Forms;
            $bytes = [System.Convert]::FromBase64String("{base64str}");
            $text = [System.Text.Encoding]::UTF8.GetString($bytes);
            [System.Windows.Forms.Clipboard]::SetText($text);
            "#
        ))
        .assert()
        .success();
}

fn get_clipboard_via_powershell() -> Vec<u8> {
    let base64str = Command::new("powershell.exe")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(
            "
            Add-Type -AssemblyName System.Windows.Forms;
            $text = [System.Windows.Forms.Clipboard]::GetText();
            $bytes = [System.Text.Encoding]::UTF8.GetBytes($text);
            [System.Console]::Write([System.Convert]::ToBase64String($bytes));
            ",
        )
        .unwrap()
        .stdout;

    base64.decode(base64str).unwrap()
}

#[test]
fn sanity_check() {
    // Make sure the powershell functions aren't borked
    set_clipboard_via_powershell("Hello, 鏡音リン");
    let text = get_clipboard_via_powershell();
    assert_eq!(text, "Hello, 鏡音リン".as_bytes());
}
