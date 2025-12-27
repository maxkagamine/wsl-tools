// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use anyhow::{Error, Result};
use windows::{
    Win32::{
        Foundation::HGLOBAL,
        System::Memory::{GlobalLock, GlobalUnlock},
    },
    core::Error as Win32Error,
};

/// Smart pointer for global memory objects, wrapping the [GlobalLock] and [GlobalUnlock] functions
/// to ensure the pointer is released when the `GlobalMemory<T>` is dropped.
///
/// [GlobalLock]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-globallock
/// [GlobalUnlock]: https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-globalunlock
pub struct GlobalMemory<'a, T>(&'a HGLOBAL, *mut T);

impl<T> GlobalMemory<'_, T> {
    pub fn as_ptr(&self) -> *mut T {
        self.1
    }
}

impl<T> Drop for GlobalMemory<'_, T> {
    fn drop(&mut self) {
        // The GlobalUnlock wrapper is implemented incorrectly. The return value indicates whether
        // the memory object is still locked, not whether the call succeeded. In our case it will
        // always be false, which windows-rs wrongly interprets as "failed," however in doing so it
        // calls GetLastError for us, so we can check its HRESULT to see if it really failed or not.
        if let Err(err) = unsafe { GlobalUnlock(*self.0) } {
            assert!(!err.code().is_err(), "GlobalUnlock: {err}");
        }
    }
}

pub trait Lock {
    fn lock<T>(&self) -> Result<GlobalMemory<'_, T>>;
}

impl Lock for HGLOBAL {
    fn lock<T>(&self) -> Result<GlobalMemory<'_, T>> {
        let ptr = unsafe { GlobalLock(*self).cast::<T>() };
        if ptr.is_null() {
            Err(Error::new(Win32Error::from_win32()).context("GlobalLock"))
        } else {
            Ok(GlobalMemory(self, ptr))
        }
    }
}
