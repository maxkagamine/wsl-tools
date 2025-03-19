// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use std::{io, os::windows::ffi::OsStrExt};
use windows::{
    Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
        System::Com::{
            CLSCTX_ALL, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
            CoInitializeEx,
        },
        UI::Shell::{
            FOF_ALLOWUNDO, FileOperation, IFileOperation, IShellItem, SHCreateItemFromParsingName,
        },
    },
    core::{Error as Win32Error, HRESULT, PCWSTR},
};

#[derive(Debug)]
pub enum RecycleError {
    NotFound(String),
    InvalidPath(String, io::Error),
    Win32(Win32Error),
}

impl From<Win32Error> for RecycleError {
    fn from(value: Win32Error) -> Self {
        Self::Win32(value)
    }
}

/// Sends the given files/directories to the Recycle Bin.
///
/// # Errors
/// If any paths do not exist or are otherwise invalid (empty string, or `GetFullPathNameW` threw an
/// error), returns `NotFound` or `InvalidPath` with the given path and (if invalid) the error
/// _without_ recycling any items.
///
/// Otherwise, if recyling fails, returns the Win32 error (see `windows::core::Error`).
pub fn recycle<TIter, TItem>(paths: TIter) -> Result<(), RecycleError>
where
    TIter: IntoIterator<Item = TItem>,
    TItem: AsRef<str>,
{
    unsafe {
        // Initialize COM. This is normally done in main(), but it's safe to call multiple times
        // with the same arguments. IFileOperation says it requires an STA thread.
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?;

        // Instantiate an IFileOperation (this replaces the older SHFileOperation function)
        // https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileoperation
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL)?;

        // Set flags
        // TODO: Check which flags to set & add options to set different flags
        // https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags
        op.SetOperationFlags(FOF_ALLOWUNDO)?;

        // Mark files for deletion
        for path in paths {
            // Resolve relative paths and convert to a null-terminated UTF-16 string.
            // path::absolute() calls GetFullPathNameW internally on Windows.
            let mut abs_path = std::path::absolute(path.as_ref())
                .map_err(|err| RecycleError::InvalidPath(path.as_ref().to_string(), err))?
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>();

            // Create an IShellItem and add it to the IFileOperation. This will cause recycle to
            // fail early if the file does not exist.
            let result: Result<IShellItem, Win32Error> =
                SHCreateItemFromParsingName(PCWSTR::from_raw(abs_path.as_mut_ptr()), None);

            match result {
                Ok(item) => op.DeleteItem(&item, None)?,
                Err(error) => {
                    // TODO: Option to ignore not found
                    if error.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) {
                        return Err(RecycleError::NotFound(path.as_ref().to_string()));
                    }

                    return Err(error.into());
                }
            }
        }

        // Do the thing
        op.PerformOperations()?;
        Ok(())
    }
}
