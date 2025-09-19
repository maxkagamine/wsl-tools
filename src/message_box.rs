// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

// Separating this out into a module serves an extra purpose: for some reason, even though build.rs
// runs and produces the libresource.a file containing the manifest, it doesn't get linked to the
// final exe unless the bin module references the lib in some way.

use windows::{
    Win32::UI::WindowsAndMessaging::{MB_OK, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE, MessageBoxW},
    core::PCWSTR,
};

pub fn show(
    text: impl AsRef<str>,
    caption: Option<impl AsRef<str>>,
    style: Option<MESSAGEBOX_STYLE>,
) -> MESSAGEBOX_RESULT {
    unsafe {
        let mut text_utf16 = encode_utf16(text);
        let mut caption_utf16 = caption.map(encode_utf16);

        MessageBoxW(
            None,
            PCWSTR::from_raw(text_utf16.as_mut_ptr()),
            match caption_utf16.as_mut() {
                Some(x) => PCWSTR::from_raw(x.as_mut_ptr()),
                None => PCWSTR::null(),
            },
            style.unwrap_or(MB_OK),
        )
    }
}

fn encode_utf16(str: impl AsRef<str>) -> Vec<u16> {
    str.as_ref().encode_utf16().chain(Some(0)).collect()
}
