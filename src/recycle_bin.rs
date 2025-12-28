// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

pub use crate::recycle_error::RecycleError;
use crate::recycle_progress_sink::RecycleProgressSink;
use std::{fs, os::windows::ffi::OsStrExt};
use windows::{
    Win32::{
        Foundation::{ERROR_CANCELLED, ERROR_FILE_NOT_FOUND},
        System::Com::{
            CLSCTX_ALL, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoCreateInstance,
            CoInitializeEx,
        },
        UI::Shell::{
            COPYENGINE_E_USER_CANCELLED, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT,
            FOFX_ADDUNDORECORD, FOFX_RECYCLEONDELETE, FileOperation, IFileOperation,
            IFileOperationProgressSink, IShellItem, SHCreateItemFromParsingName,
        },
    },
    core::{HRESULT, PCWSTR},
};

pub type RecycleOptions = u8;

/// Show progress dialog etc. and add to undo stack, the same as if deleting from Explorer.
pub const RECYCLE_NORMAL: RecycleOptions = 0x0;

/// Hide all dialogs and let the shell permanently delete anything it can't recycle. This may result
/// in files that could have been recycled being nuked instead; see `recycle()`'s remarks.
pub const RECYCLE_DANGEROUSLY_IN_BACKGROUND: RecycleOptions = 0x1;

/// Skip files/directories that don't exist instead of returning `RecycleError::NotFound`.
pub const RECYCLE_IGNORE_NOT_FOUND: RecycleOptions = 0x2;

const FILE_NOT_FOUND: HRESULT = HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0);
const CANCELLED: HRESULT = HRESULT::from_win32(ERROR_CANCELLED.0);

/// Sends the given files/directories to the Recycle Bin. Paths may be relative to the current
/// directory. Use the `RecycleOptions` flags to control the function's behavior.
///
/// The default behavior (`RECYCLE_NORMAL`) is to let the shell display the normal progress and
/// confirmation dialogs and add to Explorer's undo history, the same as if the user had deleted the
/// files in Explorer. **This should only be done in response to a user action, not automatically
/// behind the scenes.**
///
/// To recycle in the background, use `RECYCLE_DANGEROUSLY_IN_BACKGROUND`. As the name implies, this
/// is dangerous: if _any_ files cannot be recycled, _all_ of them will be permanently deleted. Due
/// to Windows API limitations, it is not possible to fully prevent this while preventing dialogs
/// from appearing: Even when using `IFileOperationProgressSink` to attempt to abort if a file is
/// about to be permanently deleted, if a file is too big for the recycle bin, it will be silently
/// permanently deleted anyway with no indication to the code of what happened. `IFileOperation` is
/// _full_ of gotchas like this. _See the Remarks below for details._
///
/// By default, if any files/directories do not exist, `RecycleError::NotFound` is returned without
/// recycling anything. Set the `RECYCLE_IGNORE_NOT_FOUND` flag to skip them instead.
///
/// # Errors
///
/// If any paths do not exist (and `RECYCLE_IGNORE_NOT_FOUND` is not set) or are otherwise invalid
/// (i.e. empty string or either `GetFullPathNameW` or `SHCreateItemFromParsingName` threw an
/// error), returns `NotFound` or `InvalidPath` with the original path and (if invalid) the error
/// without recycling.
///
/// If the operation was canceled by the user, or completed but not all files were recycled (e.g.
/// user responded "Skip" or "No" to a dialog prompt), returns `Canceled`.
///
/// If any other Win32 error occurred, returns `Win32`, which includes the HRESULT.
///
/// # Remarks
///
/// Originally, I had expected to be able to call a winapi and recycle files _programmatically_,
/// without any dialogs. This turned out to be impossible / dangerous, as the shell will happily
/// delete files permanently that it cannot recycle, and in some cases there is no way to prevent
/// this from happening, or even know that it happened at all. What follows are my notes for anyone
/// who wishes to implement recycling in their own application; I may organize this into an article
/// on the topic someday, but for now it lives here:
///
/// > FOF_ALLOWUNDO is equivalent to FOFX_ADDUNDORECORD | FOFX_RECYCLEONDELETE. That is,
/// > FOFX_RECYCLEONDELETE recycles without adding to Explorer's right click -> Undo stack, while
/// > FOFX_ADDUNDORECORD adds to the undo stack without any other implications (meaning
/// > FOFX_ADDUNDORECORD by itself will show a dialog asking to permanently delete the file).
/// >
/// > If the recycling isn't initiated by the user, messing with Explorer's undo could be surprising
/// > (the user might for example try to undo a rename only to inadvertently restore a file to some
/// > unknown location, with no visual indication of what happened). If this is reused as part of a
/// > library crate, it would be a good idea to add a parameter to override the default flags.
/// >
/// > Note that even with FOF_SILENT | FOF_NOERRORUI, a dialog will still be shown if file(s)
/// > can't be recycled (e.g. due to being on a network drive w/o a recycle bin), prompting whether
/// > to delete them permanently. FOF_NOCONFIRMATION prevents this prompt, but only by answering
/// > "yes"; there is NO flag combination that will disable dialogs and not permanently delete. And
/// > it gets worse: if _any_ file in the batch can't be recycled, IFileOperation will permanently
/// > delete ALL of them.
/// >
/// > The _only_ way to safely\* recycle files is to set up an IFileOperationProgressSink and use
/// > the PreDeleteItem hook to check if the file is about to be permanently deleted instead of
/// > recycled and abort the operation if so. (\*not actually safe though, more on that below)
/// >
/// > > Side note: A common misconception is that "each drive has its own recycle bin." This is not
/// > > true. In fact, each _folder_ can potentially have its own recycle bin! For example, if you
/// > > create a mapped network drive, by default it won't have a recycle bin, but if you relocate a
/// > > special folder there, such as Music or Videos, that folder will have a $RECYCLE.BIN in it,
/// > > while the root of the drive does not and can still only permanently delete. You can take
/// > > advantage of this and create a custom "known folder" to enable recycling in a location that
/// > > normally cannot: <https://gist.github.com/maxkagamine/0c31f5ec6fdd3fb43a1d72ae033b4c90>
/// >
/// > Unfortunately, besides not being able to "skip" an item in a batch (you can only abort the
/// > entire operation), this still doesn't work as one would expect: the `dwflags` which tells you
/// > if it can recycle or not isn't per-item; you get the same flags for all of them. **Meaning if
/// > _any_ item can't be recycled, PreDeleteItem will falsely tell you that _none_ of them can
/// > be.** This the case whether you use Advise() or pass the sink to separate DeleteItem() calls
/// > per file. Which means prompting the user for each file is out of the question, nor is it
/// > possible to avoid failing on the first error; in both cases you have to recycle one-by-one
/// > (still with the sink, though, to prevent it from permanently deleting what it can't recycle)
/// > -- no way around it.
/// >
/// > Except there's a small problem with that: instantiating and running IFileOperation is slow. In
/// > my tests, recycling 10,000 small files one-by-one took 85 seconds on average, while in a
/// > single operation took just 15 seconds. Attemping to parallelize the one-by-one approach using
/// > tokio's spawn_blocking (max 255 threads) only brought it down to 65 seconds, which suggests
/// > the shell may be locking the recycle bin (or perhaps it's repeating some expensive work
/// > internally per operation, like checking the registry and locating the applicable recycle
/// > bins). So that means we need to first _try_ to recycle en batch, but fall back to one-by-one
/// > operations if dwflags doesn't have the "can recycle" flag set. Which isn't ideal, because it
/// > might just be one file that can't be recycled, and now we're tanking performance because
/// > Windows won't tell us which one it is. But it doesn't even matter, anyway, because:
/// >
/// > IF A FILE IS TOO BIG TO RECYCLE, WINDOWS WILL LIE AND SAY IT CAN BE RECYCLED, BUT THEN
/// > PERMANENTLY DELETE IT INSTEAD!
/// >
/// > This was the final nail in the coffin for the "no dialogs" approach. In this case, `dwflags`
/// > will be 0x282, just like any other file that can be recycled. Everything will succeed, but
/// > when you look in the recycle bin, the file won't be there; it was permanently deleted. The
/// > reason is that same FOF_NOCONFIRMATION from before: You see, there's one more flag I haven't
/// > mentioned, because up until this point it hasn't appeared to have any effect:
/// > FOF_WANTNUKEWARNING. The docs do not give an accurate description of this flag. It turns out,
/// > its real purpose is to override FOF_NOCONFIRMATION and show a dialog if the file is too big to
/// > recycle, asking whether to permanently delete it or not. Without that dialog,
/// > FOF_NOCONFIRMATION answers "yes", and since the sink's `dwflags` is, as we've seen, absolutely
/// > broken, we don't have any way whatsoever to detect this situation and abort. (It's worth
/// > noting that FOF_WANTNUKEWARNING does not do anything if the file is going to be nuked because
/// > there's no recycle bin for it, only if there _is_ a recycle bin and it's too big, so it's not
/// > very effective anyway.)
/// >
/// > Frankly, the whole IFileOperation API is problematic. Not only is it cumbersome to use and
/// > poorly documented, but if you use it the way it seems you're supposed to, you'll very easily
/// > either accidentally permanently delete files, or completely miss errors, because
/// > PerformOperations can succeed even if it failed: you have to check GetAnyOperationsAborted and
/// > hook into PostDeleteItem to see the hresult for any failed operations. (Sometimes recycling
/// > can fail with 0x80070050 ERROR_FILE_EXISTS. Like of course it exists, that's the problem!)
/// > I've not found a single use of this API for recycling in the wild that does all of this
/// > properly -- which to me is the sign of a bad API.
/// >
/// > There is supposedly an undocumented IRecycleBinManager API which _might_ be capable of both
/// > telling if a file can be recycled or not and recycling files without permanently deleting
/// > them. More investigation is necessary.
/// > <https://stackoverflow.com/questions/23720519/how-to-safely-delete-folder-into-recycle-bin>
/// >
/// > Windows really does not make it easy to recycle files programmatically!
/// >
/// > ...But wait, there's more: Windows has a little-known feature dating back to Windows 2000
/// > where if you have an HTML file "foo.html" and a directory "foo_files" ("files" is localized),
/// > deleting one will delete the other (attempting to rename one of them displays a unique dialog,
/// > too). This might have been useful in Explorer, but it could be unexpected when recycling
/// > programmatically. FOF_NO_CONNECTED_ELEMENTS prevents that, or it can be disabled system-wide:
/// > <https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shfileoperationw#connecting-files>
/// >
/// > Also, I've not yet checked what IFileOperation does when a file requires admin to delete and
/// > dialogs are turned off, which is another possible failure mode besides being too big to
/// > recycle and not having a recycle bin to recycle to. With dialogs _on_, you get a prompt to
/// > continue as admin, skip, or cancel, unless you add FOFX_REQUIREELEVATION, in which case the
/// > program will immediately try to elevate itself with a UAC prompt.
pub fn recycle<TIter, TItem>(paths: TIter, options: RecycleOptions) -> Result<(), RecycleError>
where
    TIter: IntoIterator<Item = TItem>,
    TItem: AsRef<str>,
{
    recycle_internal(
        paths,
        options,
        None::<Box<dyn FnMut(String, Option<RecycleError>)>>,
    )
}

/// See `recycle`.
#[allow(clippy::missing_errors_doc)]
pub fn recycle_with_callback<'a, TIter, TItem, TCallback>(
    paths: TIter,
    options: RecycleOptions,
    callback: TCallback,
) -> Result<(), RecycleError>
where
    TIter: IntoIterator<Item = TItem>,
    TItem: AsRef<str>,
    TCallback: FnMut(String, Option<RecycleError>) + 'a,
{
    recycle_internal(paths, options, Some(callback))
}

fn recycle_internal<'a, TIter, TItem, TCallback>(
    paths: TIter,
    options: RecycleOptions,
    callback: Option<TCallback>,
) -> Result<(), RecycleError>
where
    TIter: IntoIterator<Item = TItem>,
    TItem: AsRef<str>,
    TCallback: FnMut(String, Option<RecycleError>) + 'a,
{
    unsafe {
        // Initialize COM. This is normally done in main(), but it's safe to call multiple times
        // with the same threading model. IFileOperation requires an STA thread.
        CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE).ok()?;

        // Instantiate an IFileOperation and set flags for recycling.
        // https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nn-shobjidl_core-ifileoperation
        let op: IFileOperation = CoCreateInstance(&FileOperation, None, CLSCTX_ALL)?;
        if options & RECYCLE_DANGEROUSLY_IN_BACKGROUND == RECYCLE_DANGEROUSLY_IN_BACKGROUND {
            op.SetOperationFlags(
                FOF_SILENT | FOF_NOERRORUI | FOF_NOCONFIRMATION | FOFX_RECYCLEONDELETE,
            )?;
        } else {
            op.SetOperationFlags(FOFX_ADDUNDORECORD | FOFX_RECYCLEONDELETE)?;
        }

        let mut paths_queued_for_deletion: Vec<String> = Vec::new();

        for path in paths {
            // Resolve relative paths and convert to a null-terminated UTF-16 string.
            // path::absolute() calls GetFullPathNameW internally on Windows.
            let rel_path = path.as_ref();
            let mut abs_path = std::path::absolute(rel_path)
                .map_err(|err| RecycleError::InvalidPath(rel_path.to_owned(), err.into()))?
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect::<Vec<_>>();

            // Create an IShellItem. This will error if the file does not exist.
            let item: IShellItem =
                match SHCreateItemFromParsingName(PCWSTR::from_raw(abs_path.as_mut_ptr()), None) {
                    Ok(item) => Ok(item),
                    Err(err) if err.code() == FILE_NOT_FOUND => {
                        if options & RECYCLE_IGNORE_NOT_FOUND == RECYCLE_IGNORE_NOT_FOUND {
                            continue;
                        }
                        Err(RecycleError::NotFound(rel_path.to_owned()))
                    }
                    Err(err) => Err(RecycleError::InvalidPath(rel_path.to_owned(), err.into())),
                }?;

            // Mark for deletion. Based on the example code for copying[0][1], it seems to be safe
            // to drop the shell item (which calls Release) once it's been added to the operation.
            //
            // [0]: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-copyitem#examples
            // [1]: https://github.com/microsoft/Windows-classic-samples/blob/main/Samples/Win7Samples/winui/shell/appplatform/FileOperationProgressSink/ProgressSinkSampleApp.cpp#L435
            op.DeleteItem(&item, None)?;
            paths_queued_for_deletion.push(rel_path.to_owned());
        }

        // Bail if there's nothing to recycle (empty paths or all nonexistent and ignored), as
        // PerformOperations will throw a "Catastrophic failure" if the operation is empty.
        if paths_queued_for_deletion.is_empty() {
            return Ok(());
        }

        // Set up the sink if a callback was provided
        let sink: IFileOperationProgressSink;
        if let Some(cb) = callback {
            sink = RecycleProgressSink::new(cb).into();
            op.Advise(&sink)?;
        }

        // Execute
        op.PerformOperations().map_err(|err| match err.code() {
            CANCELLED | COPYENGINE_E_USER_CANCELLED => RecycleError::Canceled,
            _ => RecycleError::Win32(err),
        })?;

        // It's important to check GetAnyOperationsAborted, since PerformOperations may succeed even
        // if the operation failed. The HRESULT of each failure can be accessed from the
        // PostDeleteItem hook in IFileOperationProgressSink.
        if op.GetAnyOperationsAborted()?.as_bool() {
            return Err(RecycleError::Canceled);
        }

        // Unfortunately, we're not done, because IFileOperation will sometimes return OK despite
        // not actually doing anything. This can happen when the file is in WSL and the default user
        // doesn't have permission to delete it (e.g. directory owned by root). We can't rely on
        // PostDeleteItem for this, either; see https://github.com/maxkagamine/wsl-tools/issues/5.
        for path in paths_queued_for_deletion {
            if fs::exists(&path).unwrap_or_default() {
                return Err(RecycleError::Unknown);
            }
        }

        Ok(())
    }
}
