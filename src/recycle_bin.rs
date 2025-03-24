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
            FOF_NO_CONNECTED_ELEMENTS, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT,
            FOFX_RECYCLEONDELETE, FileOperation, IFileOperation, IFileOperationProgressSink,
            IShellItem, SHCreateItemFromParsingName,
        },
    },
    core::{Error as Win32Error, HRESULT, PCWSTR},
};

#[derive(Debug)]
pub enum RecycleError {
    NotFound,
    InvalidPath(io::Error),
    NotRecyclable,
    Win32(Win32Error),
}

impl From<Win32Error> for RecycleError {
    fn from(value: Win32Error) -> Self {
        Self::Win32(value)
    }
}

/// Initializes COM using a [single-threaded apartment (STA)][STA]. This must be called once per
/// thread before calling `recycle()`. Panics if `CoInitializeEx` fails.
///
/// [STA]: https://learn.microsoft.com/en-us/previous-versions/ms809971(v=msdn.10)#single-threaded-apartments
pub fn initialize_com() {
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).unwrap();
    }
}

/// Sends the given file or directory to the Recycle Bin, failing if it cannot be recycled.
///
/// # Errors
///
/// If any paths do not exist or are otherwise invalid (empty string or `GetFullPathNameW` threw an
/// error), returns `NotFound` or `InvalidPath`, the latter with the error message in an
/// `io::Error`.
///
/// If the item cannot be recycled, e.g. due to being on a network drive [without a recycle
/// bin](https://gist.github.com/maxkagamine/0c31f5ec6fdd3fb43a1d72ae033b4c90), returns
/// `NotRecyclable`.
///
/// If recycling fails for any other reason, returns the Win32 error (see `windows::core::Error`).
pub fn recycle<T: AsRef<str>>(path: T) -> Result<(), RecycleError> {
    unsafe {
        // Instantiate an IFileOperation (this replaces the older SHFileOperation function)
        // https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileoperation
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL)?;

        // Set flags
        //
        // FOF_ALLOWUNDO is equivalent to FOFX_ADDUNDORECORD | FOFX_RECYCLEONDELETE. That is,
        // FOFX_RECYCLEONDELETE recycles without adding to Explorer's right click -> Undo stack,
        // while FOFX_ADDUNDORECORD adds to the undo stack without any other implications (meaning
        // FOFX_ADDUNDORECORD by itself will show a dialog asking to permanently delete the file).
        //
        // For our purposes, since the recycling might not necessarily be initiated by the user,
        // messing with Explorer's undo could be surprising (the user might for example try to undo
        // a rename only to inadvertently restore a file to some unknown location, with no visual
        // indication of what happened). If this is reused as part of a library crate, it would be a
        // good idea to add a parameter to override the default flags.
        //
        // Note that even with FOF_SILENT | FOF_NOERRORUI, a dialog will still be shown if file(s)
        // can't be recycled (e.g. due to being on a network drive w/o a recycle bin), prompting
        // whether to delete them permanently. FOF_NOCONFIRMATION prevents this prompt, but only by
        // answering "yes"; there is NO flag combination that will disable dialogs and not
        // permanently delete. And it gets worse: if _any_ file in the batch can't be recycled,
        // IFileOperation will permanently delete ALL of them.
        //
        // The _only_ way to safely recycle files is to set up an IFileOperationProgressSink and use
        // the PreDeleteItem hook to check if the file is about to be permanently deleted instead of
        // recycled and abort the operation if so.
        //
        // Unfortunately, besides to not being able to "skip" an item in a batch (you can only abort
        // the entire operation), this still doesn't work as one would expect: the `dwflags` which
        // tells you if it can recycle or not isn't per-item; you get the same flags for all of
        // them. **Meaning if _any_ item can't be recycled, PreDeleteItem will falsely tell you that
        // _none_ of them can be.** This the case whether you use Advise() or pass the sink to
        // separate DeleteItem() calls per file. Which means prompting the user for each file is out
        // of the question, nor is it possible to avoid failing on the first error; in both cases
        // you have to recycle one-by-one (still with the sink, though, to prevent it from
        // permanently deleting what it can't recycle) -- no way around it.
        //
        // Except there's a small problem with that: instantiating and running IFileOperation is
        // slow. In my tests, recycling 10,000 small files one-by-one took 85 seconds on average,
        // while in a single operation took just 15 seconds. Attemping to parallelize the one-by-one
        // approach using tokio's spawn_blocking (max 255 threads) only brought it down to 65
        // seconds, which suggests the shell may be locking the recycle bin (or perhaps it's
        // repeating some expensive work internally per operation, like checking the registry and
        // locating the applicable recycle bins). So that means we need to first _try_ to recycle en
        // batch, but fall back to one-by-one operations if dwflags doesn't have the "can recycle"
        // flag set. Which isn't ideal, because it might just be one file that can't be recycled,
        // and now we're tanking performance because Windows won't tell us which one it is. But it
        // doesn't even matter, anyway, because:
        //
        // IF A FILE IS TOO BIG TO RECYCLE, WINDOWS WILL LIE AND SAY IT CAN BE RECYCLED, BUT THEN
        // PERMANENTLY DELETE IT INSTEAD!
        //
        // This is the final nail in the coffin. In this case, `dwflags` will be 0x282, just like
        // any other file that can be recycled. Everything will succeed, but when you look in the
        // recycle bin, the file won't be there; it was permanently deleted. The reason is that same
        // FOF_NOCONFIRMATION from before: You see, there's one more flag I haven't mentioned,
        // because up until this point it hasn't appeared to have any effect: FOF_WANTNUKEWARNING.
        // The docs do not give an accurate description of this flag. It turns out, its real purpose
        // is to override FOF_NOCONFIRMATION and show a dialog if the file is too big to recycle,
        // asking whether to permanently delete it or not. Without that dialog, FOF_NOCONFIRMATION
        // answers "yes", and since the sink's `dwflags` is, as we've seen, absolutely broken, we
        // don't have any way whatsoever to detect this situation and abort. What's funny is
        // FOF_WANTNUKEWARNING does not do anything if the file is going to be nuked because there's
        // no recycle bin for it, only if there _is_ a recycle bin and it's too big, so it's not
        // even good at its job.
        //
        // Frankly, the whole IFileOperation API is awful (and dangerous). Not only is it cumbersome
        // to use and poorly documented, but if you use it the way it seems you're supposed to,
        // you'll very easily either accidentally permanently delete files, or completely miss
        // errors, because PerformOperations can succeed even if it failed: you have to check
        // GetAnyOperationsAborted and hook into PostDeleteItem to see the hresult for any failed
        // operations. I've not found a single use of this API for recycling in the wild that does
        // all of this properly.
        //
        // There is supposedly an undocumented IRecycleBinManager API which _might_ be capable of
        // both telling if a file can be recycled or not _and_ recycling files without permenently
        // deleting them (unlike IFileOperation & SHFileOperation). More investigation is necessary.
        // https://stackoverflow.com/questions/23720519/how-to-safely-delete-folder-into-recycle-bin
        //
        // Windows really does not make it easy to recycle files programmatically!
        //
        // ---
        //
        // But wait, there's more: Windows has a little-known feature dating back to Windows 2000
        // where if you have an HTML file "foo.html" and a directory "foo_files" ("files" is
        // localized), deleting one will delete the other (attempting to rename one of them displays
        // a unique dialog, too). This might have been useful in Explorer, but it would be
        // unexpected when recycling programmatically; FOF_NO_CONNECTED_ELEMENTS disables it.
        //
        // Also, I've not yet checked what IFileOperation does when a file requires admin to delete,
        // which is another possible failure mode besides being too big to recycle and not having a
        // recycle bin to recycle to. One report online suggests FOFX_REQUIREELEVATION can bypass
        // the UAC prompt, but surely that can't be right?
        //
        // https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags
        op.SetOperationFlags(
            FOF_SILENT
                | FOF_NOERRORUI
                | FOF_NOCONFIRMATION
                | FOFX_RECYCLEONDELETE
                | FOF_NO_CONNECTED_ELEMENTS,
        )?;

        // Resolve relative paths and convert to a null-terminated UTF-16 string. path::absolute()
        // calls GetFullPathNameW internally on Windows.
        let mut abs_path = std::path::absolute(path.as_ref())
            .map_err(RecycleError::InvalidPath)?
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();

        // Create an IShellItem. This will error if the file does not exist.
        let item: IShellItem =
            SHCreateItemFromParsingName(PCWSTR::from_raw(abs_path.as_mut_ptr()), None).map_err(
                |err| {
                    if err.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) {
                        RecycleError::NotFound
                    } else {
                        RecycleError::Win32(err)
                    }
                },
            )?;

        // Create the sink as described above. Passing it to DeleteItem() rather than Advise() saves
        // us one winapi call.
        let progress: IFileOperationProgressSink = RecycleProgressSink.into();

        // Do the thing
        op.DeleteItem(&item, &progress)?;
        op.PerformOperations()?;

        if op.GetAnyOperationsAborted()?.as_bool() {
            println!("operations aborted");
        }

        Ok(())
    }
}

// TODO: Remove (restored from previous commit for benchmarking)
pub fn recycle_batch<TIter, TItem>(paths: TIter) -> Result<(), RecycleError>
where
    TIter: IntoIterator<Item = TItem>,
    TItem: AsRef<str>,
{
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?;
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL)?;
        op.SetOperationFlags(
            FOF_SILENT
                | FOF_NOERRORUI
                | FOF_NOCONFIRMATION
                | FOFX_RECYCLEONDELETE
                | FOF_NO_CONNECTED_ELEMENTS,
        )?;
        let progress: IFileOperationProgressSink = RecycleProgressSink.into();
        op.Advise(&progress)?;
        for path in paths {
            let mut abs_path = std::path::absolute(path.as_ref())
                .map_err(RecycleError::InvalidPath)?
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>();
            let result: Result<IShellItem, Win32Error> =
                SHCreateItemFromParsingName(PCWSTR::from_raw(abs_path.as_mut_ptr()), None);
            match result {
                Ok(item) => op.DeleteItem(&item, None)?,
                Err(error) => {
                    if error.code() == HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) {
                        return Err(RecycleError::NotFound);
                    }
                    return Err(error.into());
                }
            }
        }
        op.PerformOperations()?;

        if op.GetAnyOperationsAborted()?.as_bool() {
            println!("operations aborted");
        }
        Ok(())
    }
}
