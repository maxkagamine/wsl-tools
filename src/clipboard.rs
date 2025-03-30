// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use anyhow::{Context, Error, Result};
use std::time::Duration;
use windows::{
    Win32::{
        Foundation::{HANDLE, HGLOBAL},
        System::{
            DataExchange::{
                CloseClipboard, EmptyClipboard, GetClipboardData, IsClipboardFormatAvailable,
                OpenClipboard, SetClipboardData,
            },
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
        },
    },
    core::{Error as Win32Error, PWSTR},
};

const CF_UNICODETEXT: u32 = 13;

/// Copies `text` to the clipboard.
///
/// # Errors
/// Error result contains the Win32 error if the operation failed.
pub fn set_text(text: &str) -> Result<()> {
    unsafe {
        // Open the clipboard
        open_clipboard()?;

        // Can replace this with a try block once that feature is stable
        let result = (|| -> Result<()> {
            // Clear the clipboard
            EmptyClipboard().context("EmptyClipboard")?;

            // Convert the text to UTF-16 and add a null terminator
            let mut utf16: Vec<u16> = text.encode_utf16().collect();
            utf16.push(0);
            let length = utf16.len();

            // Allocate global memory to hold the text
            let hglobal = GlobalAlloc(GMEM_MOVEABLE, length * 2).context("GlobalAlloc")?;

            // Acquire a pointer to the memory
            let dest = global_lock(hglobal)?;

            // Copy the text to the buffer
            std::ptr::copy_nonoverlapping(utf16.as_ptr(), dest, length);

            // Release the pointer
            global_unlock(hglobal)?;

            // Place the handle on the clipboard
            SetClipboardData(CF_UNICODETEXT, Some(HANDLE(hglobal.0)))
                .context("SetClipboardData")?;

            Ok(())
        })();

        // Close the clipboard and return the result
        let _ = CloseClipboard();
        result
    }
}

/// Gets the contents of the clipboard as Unicode text.
///
/// # Returns
/// `Some` if the clipboard contains text; `None` if it does not.
///
/// # Errors
/// Error result contains the Win32 error if the operation failed.
pub fn get_text() -> Result<Option<String>> {
    unsafe {
        // Check if clipboard contains text
        if IsClipboardFormatAvailable(CF_UNICODETEXT).is_err() {
            return Ok(None);
        }

        // Open the clipboard
        open_clipboard()?;

        // Can replace this with a try block once that feature is stable
        let result = (|| -> Result<Option<String>> {
            // Get a handle to the text on the clipboard
            let hglobal = match GetClipboardData(CF_UNICODETEXT) {
                Ok(handle) => HGLOBAL(handle.0),
                Err(_) => return Ok(None), // Clipboard changed while we were opening it
            };

            // Acquire a pointer to the buffer
            let ptr = global_lock(hglobal)?;

            // Read the buffer as a PWSTR (null-terminated UTF-16) and convert it to a string
            let str = String::from_utf16_lossy(PWSTR::from_raw(ptr).as_wide());

            // Release the pointer
            global_unlock(hglobal)?;

            Ok(Some(str))
        })();

        // Close the clipboard and return the result
        let _ = CloseClipboard();
        result
    }
}

/// Clears the clipboard.
///
/// # Errors
/// Error result contains the Win32 error if the operation failed.
pub fn clear() -> Result<()> {
    unsafe {
        open_clipboard()?;
        let result = EmptyClipboard().context("EmptyClipboard");
        let _ = CloseClipboard();
        result
    }
}

/// Tries to open the clipboard.
///
/// `OpenClipboard` can fail with `ERROR_ACCESS_DENIED` if another program is using it at the same
/// time (I ran into this with my [clips] bash function while trying to copy URLs into an array, so
/// it's not [inconceivable] that this could happen; simply calling `get_text()` in a loop will
/// trigger an access denied eventually as well). [TextCopy] solves this by retrying up to 10 times
/// with a small delay; we'll do the same.
///
/// [clips]: https://github.com/maxkagamine/dotfiles/blob/1139d1117e0b89ec14305036c27b520d7f92e307/mods/bash/.bashrc#L106-L122
/// [TextCopy]: https://github.com/CopyText/TextCopy/blob/6.2.1/src/TextCopy/WindowsClipboard.cs#L87
/// [inconceivable]: https://youtu.be/D9MS2y2YU_o?t=126
unsafe fn open_clipboard() -> Result<()> {
    let mut i = 10;
    loop {
        let result = unsafe { OpenClipboard(None).context("OpenClipboard") };
        if result.is_ok() || i == 1 {
            return result;
        }
        std::thread::sleep(Duration::from_millis(100));
        i -= 1;
    }
}

// TODO: We could turn this into a smart pointer by implementing a struct that calls GlobalUnlock
// when dropped
unsafe fn global_lock(hglobal: HGLOBAL) -> Result<*mut u16> {
    let ptr = unsafe { GlobalLock(hglobal).cast::<u16>() };
    if ptr.is_null() {
        Err(Error::new(Win32Error::from_win32()).context("GlobalLock"))
    } else {
        Ok(ptr)
    }
}

unsafe fn global_unlock(hglobal: HGLOBAL) -> Result<()> {
    // The GlobalUnlock wrapper is implemented incorrectly. The returned boolean indicates whether
    // the memory object is still locked, not whether the call succeeded. In our case it will always
    // be false, which windows-rs wrongly interprets as "failed," however in doing so it calls
    // GetLastError for us, so we can check its HRESULT to see if it really failed or not.
    unsafe {
        match GlobalUnlock(hglobal) {
            Err(err) if err.code().is_err() => Err(Error::new(err).context("GlobalUnlock")),
            _ => Ok(()),
        }
    }
}
