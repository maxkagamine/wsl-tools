// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]
#![allow(clippy::pedantic, unused_variables)]

use crate::recycle_error::RecycleError;
use std::{cell::UnsafeCell, fs};
use windows::{
    Win32::UI::Shell::{
        COPYENGINE_E_ACCESS_DENIED_SRC, COPYENGINE_E_SHARING_VIOLATION_SRC,
        COPYENGINE_E_USER_CANCELLED, IFileOperationProgressSink, IFileOperationProgressSink_Impl,
        IShellItem, SIGDN_DESKTOPABSOLUTEEDITING,
    },
    core::{Error, HRESULT, PCWSTR, Ref, Result, implement},
};

fn get_shell_item_path(
    psiitem: &Ref<'_, IShellItem>,
) -> std::result::Result<String, Box<dyn std::error::Error>> {
    Ok(unsafe {
        psiitem
            .ok()?
            .GetDisplayName(SIGDN_DESKTOPABSOLUTEEDITING)?
            .to_string()?
    })
}

#[implement(IFileOperationProgressSink)]
#[allow(clippy::type_complexity)]
pub struct RecycleProgressSink<'a> {
    callback: UnsafeCell<Box<dyn FnMut(String, Option<RecycleError>) + 'a>>,
}

impl<'a> RecycleProgressSink<'a> {
    pub fn new<T>(callback: T) -> Self
    where
        T: FnMut(String, Option<RecycleError>) + 'a,
    {
        RecycleProgressSink {
            callback: UnsafeCell::new(Box::new(callback)),
        }
    }
}

impl IFileOperationProgressSink_Impl for RecycleProgressSink_Impl<'_> {
    fn PostDeleteItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        hrdelete: HRESULT,
        psinewlycreated: Ref<'_, IShellItem>,
    ) -> Result<()> {
        let path = get_shell_item_path(&psiitem).unwrap_or_else(|err| {
            // This should probably never happen, but better than panicking in case it does
            format!("<Error: {err}>")
        });

        let error = if hrdelete.is_ok() {
            // Note: PostDeleteItem is sometimes called *before* the item is actually deleted, so
            // checking here to see if the file was actually deleted or not will not work. We'll
            // assume it's ok, as that's an edge case, but we need to double-check after calling
            // PerformOperations, even if GetAnyOperationsAborted returns false. IFileOperation is a
            // buggy mess. See https://github.com/maxkagamine/wsl-tools/issues/5.
            None
        } else if hrdelete == COPYENGINE_E_ACCESS_DENIED_SRC {
            Some(RecycleError::AccessDenied)
        } else if hrdelete == COPYENGINE_E_SHARING_VIOLATION_SRC {
            if fs::symlink_metadata(&path).is_ok_and(|m| m.is_dir()) {
                Some(RecycleError::FolderInUse)
            } else {
                Some(RecycleError::FileInUse)
            }
        } else if hrdelete == COPYENGINE_E_USER_CANCELLED {
            Some(RecycleError::Canceled)
        } else {
            Some(Error::from_hresult(hrdelete).into())
        };

        unsafe {
            (*self.callback.get())(path, error);
        }

        Ok(())
    }

    fn PreDeleteItem(&self, dwflags: u32, psiitem: Ref<'_, IShellItem>) -> Result<()> {
        // This is how we might attempt to prevent the shell from permanently deleting items when
        // dialogs are disabled. As discussed, though, this doesn't work in all circumstances.
        // const TSF_DELETE_RECYCLE_IF_POSSIBLE: u32 = 0x80;
        // if dwflags & TSF_DELETE_RECYCLE_IF_POSSIBLE == TSF_DELETE_RECYCLE_IF_POSSIBLE {
        //     Ok(())
        // } else {
        //     println!("aborting from sink! dwflags = {dwflags}"); // TODO: Remove
        //     Err(Error::from_hresult(E_ABORT))
        // }
        Ok(())
    }

    // region: No-ops
    fn StartOperations(&self) -> Result<()> {
        Ok(())
    }

    fn FinishOperations(&self, hrresult: HRESULT) -> Result<()> {
        Ok(())
    }

    fn PreRenameItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
    ) -> Result<()> {
        Ok(())
    }

    fn PostRenameItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
        hrrename: HRESULT,
        psinewlycreated: Ref<'_, IShellItem>,
    ) -> Result<()> {
        Ok(())
    }

    fn PreMoveItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        psidestinationfolder: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
    ) -> Result<()> {
        Ok(())
    }

    fn PostMoveItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        psidestinationfolder: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
        hrmove: HRESULT,
        psinewlycreated: Ref<'_, IShellItem>,
    ) -> Result<()> {
        Ok(())
    }

    fn PreCopyItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        psidestinationfolder: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
    ) -> Result<()> {
        Ok(())
    }

    fn PostCopyItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        psidestinationfolder: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
        hrcopy: HRESULT,
        psinewlycreated: Ref<'_, IShellItem>,
    ) -> Result<()> {
        Ok(())
    }

    fn PreNewItem(
        &self,
        dwflags: u32,
        psidestinationfolder: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
    ) -> Result<()> {
        Ok(())
    }

    fn PostNewItem(
        &self,
        dwflags: u32,
        psidestinationfolder: Ref<'_, IShellItem>,
        psznewname: &PCWSTR,
        psztemplatename: &PCWSTR,
        dwfileattributes: u32,
        hrnew: HRESULT,
        psinewitem: Ref<'_, IShellItem>,
    ) -> Result<()> {
        Ok(())
    }

    fn UpdateProgress(&self, iworktotal: u32, iworksofar: u32) -> Result<()> {
        Ok(())
    }

    fn ResetTimer(&self) -> Result<()> {
        Ok(())
    }

    fn PauseTimer(&self) -> Result<()> {
        Ok(())
    }

    fn ResumeTimer(&self) -> Result<()> {
        Ok(())
    }
    // endregion
}
