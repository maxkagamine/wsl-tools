// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use crate::recycle_progress_sink::RecycleProgressSink;
use std::{io, os::windows::ffi::OsStrExt};
use windows::{
    Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
        System::Com::{
            CLSCTX_ALL, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
            CoInitializeEx,
        },
        UI::Shell::{
            FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, FOFX_RECYCLEONDELETE, FileOperation,
            IFileOperation, IFileOperationProgressSink, IShellItem, SHCreateItemFromParsingName,
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
/// Otherwise, if recycling fails, returns the Win32 error (see `windows::core::Error`).
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
        //
        // FOF_ALLOWUNDO is equivalent to FOFX_ADDUNDORECORD | FOFX_RECYCLEONDELETE. That is,
        // FOFX_RECYCLEONDELETE recycles without adding to Explorer's right click -> Undo stack,
        // while FOFX_ADDUNDORECORD adds to the undo stack without any other side effects, which
        // means that, without any other flags, a dialog will appear to permanently delete the file.
        //
        // For our purposes, since the recycling might not necessarily be initiated by the user,
        // messing with Explorer's undo could be surprising (the user might for example try to undo
        // a rename only to inadvertently restore a file to some unknown location, with no visual
        // indication of what happened).
        //
        // Even with FOF_SILENT | FOF_NOERRORUI, a dialog will still be shown if file(s) can't be
        // recycled, prompting whether to delete them permanently. FOF_NOCONFIRMATION prevents this
        // prompt, but only by answering "yes"; there's no flag combination that will disable
        // dialogs and not permanently delete. And it gets worse: with those flags, if _any_ file
        // can't be recycled, IFileOperation will permanently delete ALL of them.
        //
        // The _only_ way to safely recycle files is to set up an IFileOperationProgressSink and use
        // the PreDeleteItem hook to check if the file is about to be permanently deleted instead of
        // recycled and abort the operation if so.
        //
        // Unfortunately, besides to not being able to "skip" (you can only abort the entire
        // operation), this still doesn't work as one would expect: the `dwflags` which tells you if
        // it can recycle or not isn't per-item; you get the same flags for all of them. Meaning if
        // _any_ item can't be recycled, PreDeleteItem will falsely tell you that _none_ of them can
        // be. Which means prompting the user for each file is out of the question, nor is it
        // possible to avoid failing on the first error; in both cases you have to recycle
        // one-by-one (still with the sink, though, to prevent it from permanently deleting what it
        // can't recycle) -- no way around it.
        //
        // Windows really does not make it easy to recycle files programmatically!
        //
        // https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags
        op.SetOperationFlags(
            FOF_SILENT | FOF_NOERRORUI | FOF_NOCONFIRMATION | FOFX_RECYCLEONDELETE,
        )?;

        // Set up the progress sink as described above
        let progress: IFileOperationProgressSink = RecycleProgressSink.into();
        op.Advise(&progress)?;

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
