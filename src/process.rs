// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use anyhow::Result;
use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
use windows::Win32::{
    System::Com::{COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx},
    UI::{
        Shell::{SEE_MASK_FLAG_NO_UI, SEE_MASK_NOASYNC, SHELLEXECUTEINFOW, ShellExecuteExW},
        WindowsAndMessaging::SW_SHOWNORMAL,
    },
};
use windows_core::PCWSTR;

/// Converts `str` to a null-terminated UTF-16 string.
fn to_wstr(str: impl AsRef<OsStr>) -> Vec<u16> {
    str.as_ref()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>()
}

/// Opens the given file, directory, or URL.
///
/// For details on the `verb` parameter, see:
/// <https://learn.microsoft.com/en-us/windows/win32/shell/launch#object-verbs>
///
/// # Errors
/// Error result contains the Win32 error if the operation failed.
#[allow(clippy::cast_possible_truncation)]
pub fn shell_execute(path: &str, verb: Option<&str>) -> Result<()> {
    // Reference:
    // https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shellexecuteexw
    // https://github.com/dotnet/runtime/blob/v9.0.3/src/libraries/System.Diagnostics.Process/src/System/Diagnostics/Process.Win32.cs#L30
    unsafe {
        // Initialize COM. This is normally done in main(), but it's safe to call multiple times
        // with the same threading model. ShellExecuteEx requires an STA thread.
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?;

        // Convert to null-terminated UTF-16 strings
        let mut path_wstr = to_wstr(path);
        let verb_wstr: Option<Vec<u16>> = verb.map(to_wstr);

        // Call ShellExecuteEx
        let mut info = SHELLEXECUTEINFOW {
            cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
            lpFile: PCWSTR::from_raw(path_wstr.as_mut_ptr()),
            lpVerb: match verb_wstr {
                Some(mut v) => PCWSTR::from_raw(v.as_mut_ptr()),
                None => PCWSTR::null(),
            },
            fMask: SEE_MASK_NOASYNC | SEE_MASK_FLAG_NO_UI,
            nShow: SW_SHOWNORMAL.0,
            ..Default::default()
        };

        ShellExecuteExW(&raw mut info)?;

        Ok(())
    }
}
