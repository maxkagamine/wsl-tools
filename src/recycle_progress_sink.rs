// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]
#![allow(clippy::pedantic, unused_variables)]

use windows::{
    Win32::{
        Foundation::E_ABORT,
        UI::Shell::{
            IFileOperationProgressSink, IFileOperationProgressSink_Impl, IShellItem,
            SIGDN_DESKTOPABSOLUTEEDITING,
        },
    },
    core::{Error, HRESULT, PCWSTR, Ref, Result, implement},
};

const TSF_DELETE_RECYCLE_IF_POSSIBLE: u32 = 0x80;

#[implement(IFileOperationProgressSink)]
pub struct RecycleProgressSink;

impl IFileOperationProgressSink_Impl for RecycleProgressSink_Impl {
    fn PreDeleteItem(&self, dwflags: u32, psiitem: Ref<'_, IShellItem>) -> Result<()> {
        if cfg!(debug_assertions) {
            let path = unsafe {
                psiitem
                    .unwrap()
                    .GetDisplayName(SIGDN_DESKTOPABSOLUTEEDITING)
                    .unwrap()
                    .to_string()
                    .unwrap()
            };
            let recycle_is_possible =
                dwflags & TSF_DELETE_RECYCLE_IF_POSSIBLE == TSF_DELETE_RECYCLE_IF_POSSIBLE;
            println!(
                "PreDeleteItem: dwflags = {dwflags} ({recycle_is_possible}), psiitem = {path}"
            );
        }

        // Despite the name, this flag is set when recycling is possible (for ALL items; see the
        // lengthly comment in recycle_bin.rs). If not set, it's going to permanently delete
        // instead, so we need to abort.
        if dwflags & TSF_DELETE_RECYCLE_IF_POSSIBLE == TSF_DELETE_RECYCLE_IF_POSSIBLE {
            Ok(())
        } else {
            Err(Error::from_hresult(E_ABORT))
        }
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

    fn PostDeleteItem(
        &self,
        dwflags: u32,
        psiitem: Ref<'_, IShellItem>,
        hrdelete: HRESULT,
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
