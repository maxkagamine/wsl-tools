// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

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
    core::{Error, PWSTR, Result},
};

const CF_UNICODETEXT: u32 = 13;

/// Copies `text` to the clipboard.
///
/// # Errors
/// Error result contains the Win32 error if the operation failed.
pub fn set_text(text: &str) -> Result<()> {
    unsafe {
        // Open the clipboard
        OpenClipboard(None)?;

        // Can replace this with a try block once that feature is stable
        let result = (|| -> Result<()> {
            // Clear the clipboard
            EmptyClipboard()?;

            // Convert the text to UTF-16 and add a null terminator
            let mut utf16: Vec<u16> = text.encode_utf16().collect();
            utf16.push(0);
            let length = utf16.len();

            // Allocate global memory to hold the text
            let hglobal = GlobalAlloc(GMEM_MOVEABLE, length * 2)?;

            // Acquire a pointer to the memory
            let dest = GlobalLock(hglobal).cast::<u16>();
            if dest.is_null() {
                return Err(Error::from_win32());
            }

            // Copy the text to the buffer
            std::ptr::copy_nonoverlapping(utf16.as_ptr(), dest, length);

            // Release the pointer
            //
            // Note: the GlobalUnlock wrapper is implemented incorrectly. The returned boolean
            // indicates whether the memory object is still locked; in this case it will always be
            // false, which windows-rs wrongly interprets as "failed." However, in doing so it calls
            // GetLastError for us, so we can check its HRESULT to see if it really failed or not.
            GlobalUnlock(hglobal).or_else(|err| {
                if err.code().is_err() {
                    return Err(err);
                }
                Ok(())
            })?;

            // Place the handle on the clipboard
            SetClipboardData(CF_UNICODETEXT, Some(HANDLE(hglobal.0)))?;

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
        OpenClipboard(None)?;

        // Can replace this with a try block once that feature is stable
        let result = (|| -> Result<Option<String>> {
            // Get a handle to the text on the clipboard
            let hglobal = match GetClipboardData(CF_UNICODETEXT) {
                Ok(handle) => HGLOBAL(handle.0),
                Err(_) => return Ok(None), // Clipboard changed while we were opening it
            };

            // Acquire a pointer to the buffer
            let ptr = GlobalLock(hglobal).cast::<u16>();
            if ptr.is_null() {
                return Err(Error::from_win32());
            }

            // Read the buffer as a PWSTR (null-terminated UTF-16) and convert it to a string
            let str = String::from_utf16_lossy(PWSTR::from_raw(ptr).as_wide());

            // Release the pointer (see note in set_text above)
            GlobalUnlock(hglobal).or_else(|err| {
                if err.code().is_err() {
                    return Err(err);
                }
                Ok(())
            })?;

            Ok(Some(str))
        })();

        // Close the clipboard and return the result
        let _ = CloseClipboard();
        result
    }
}
