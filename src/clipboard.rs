// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use windows::{
    Win32::{
        Foundation::HANDLE,
        System::{
            DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData},
            Memory::{GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock},
            Ole::CF_UNICODETEXT,
        },
    },
    core::{Error, Result},
};

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
            SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(HANDLE(hglobal.0)))?;

            Ok(())
        })();

        // Close the clipboard and return the result
        let _ = CloseClipboard();
        result
    }
}
