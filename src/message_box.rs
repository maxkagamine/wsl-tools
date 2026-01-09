// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

// Separating this out into a module serves an extra purpose: for some reason, even though build.rs
// runs and produces the libresource.a file containing the manifest, it doesn't get linked to the
// final exe unless the bin module references the lib in some way.

use windows::{
    Win32::UI::WindowsAndMessaging::{
        MB_ICONERROR, MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MessageBoxW,
    },
    core::PCWSTR,
};

/// Shows a message box.
///
/// `caption` defaults to the package name if None; `style` defaults to `MB_OK | MB_ICONERROR`.
pub fn show(
    text: impl AsRef<str>,
    caption: Option<&str>,
    style: Option<MESSAGEBOX_STYLE>,
) -> MESSAGEBOX_RESULT {
    unsafe {
        let mut text_utf16 = encode_utf16(text.as_ref());
        let mut caption_utf16 = encode_utf16(caption.unwrap_or(env!("CARGO_PKG_NAME")));

        MessageBoxW(
            None,
            PCWSTR::from_raw(text_utf16.as_mut_ptr()),
            PCWSTR::from_raw(caption_utf16.as_mut_ptr()),
            style.unwrap_or(MB_OK | MB_ICONERROR),
        )
    }
}

fn encode_utf16(str: &str) -> Vec<u16> {
    str.encode_utf16().chain(Some(0)).collect()
}
