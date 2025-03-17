// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use wsl_tools::clipboard;

mod clipboard_via_powershell;
use clipboard_via_powershell::*;

#[test]
fn powershell_working() {
    set_clipboard_via_powershell("Hello, 鏡音リン");
    assert!(clipboard_contains_text());
    assert_eq!(get_clipboard_via_powershell(), "Hello, 鏡音リン");
    clear_clipboard_via_powershell();
    assert!(!clipboard_contains_text());
}

#[test]
fn gets_ascii_text() {
    let expected = "kagamine rin";

    set_clipboard_via_powershell(expected);

    let actual = clipboard::get_text()
        .unwrap_or_else(|err| panic!("get_text() failed: {err:?}"))
        .unwrap_or_else(|| panic!("get_text() returned None"));

    assert_eq!(actual, expected);
}

#[test]
fn gets_unicode_text() {
    let expected = "鏡音リン";

    set_clipboard_via_powershell(expected);

    let actual = clipboard::get_text()
        .unwrap_or_else(|err| panic!("get_text() failed: {err:?}"))
        .unwrap_or_else(|| panic!("get_text() returned None"));

    assert_eq!(actual, expected);
}

#[test]
fn gets_text_with_newlines() {
    let expected = "rin\r\nmiku\n";

    set_clipboard_via_powershell(expected);

    let actual = clipboard::get_text()
        .unwrap_or_else(|err| panic!("get_text() failed: {err:?}"))
        .unwrap_or_else(|| panic!("get_text() returned None"));

    assert_eq!(actual, expected);
}

#[test]
fn sets_ascii_text() {
    let expected = "kagamine rin";

    clear_clipboard_via_powershell();

    clipboard::set_text(expected).unwrap_or_else(|err| panic!("set_text() failed: {err:?}"));

    let actual = get_clipboard_via_powershell();
    assert_eq!(actual, expected);
}

#[test]
fn sets_unicode_text() {
    let expected = "鏡音リン";

    clear_clipboard_via_powershell();

    clipboard::set_text(expected).unwrap_or_else(|err| panic!("set_text() failed: {err:?}"));

    let actual = get_clipboard_via_powershell();
    assert_eq!(actual, expected);
}

#[test]
fn sets_text_with_newlines() {
    let expected = "rin\r\nmiku\n";

    clear_clipboard_via_powershell();

    clipboard::set_text(expected).unwrap_or_else(|err| panic!("set_text() failed: {err:?}"));

    let actual = get_clipboard_via_powershell();
    assert_eq!(actual, expected);
}

#[test]
fn clears_clipboard() {
    set_clipboard_via_powershell("riiiiiin");

    clipboard::clear().unwrap_or_else(|err| panic!("clear() failed: {err:?}"));

    assert!(!clipboard_contains_text());
}
