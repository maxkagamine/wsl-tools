// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use std::{error::Error, fmt::Display};
use windows::core::Error as Win32Error;

#[derive(Debug)]
pub enum RecycleError {
    NotFound(String),
    InvalidPath(String, Box<dyn Error>),
    Win32(Win32Error),
    Canceled,

    // Per-item errors only applicable in the sink/callback.
    AccessDenied,
    FileInUse,
    FolderInUse,
    Unknown,
}

impl From<Win32Error> for RecycleError {
    fn from(value: Win32Error) -> Self {
        Self::Win32(value)
    }
}

impl Display for RecycleError {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "Failed to recycle \"{path}\": No such file or directory."),
            Self::InvalidPath(path, err) => write!(f, "Failed to recycle \"{path}\": {err}"),
            Self::Win32(err) => Display::fmt(err, f),
            Self::Canceled => write!(f, "The operation was canceled."),
            Self::AccessDenied => write!(f, "Access denied."),
            Self::FileInUse => write!(f, "The file is open in another program."),
            Self::FolderInUse => write!(f, "The folder or a file in it is open in another program."),
            Self::Unknown => write!(f, "Unknown error (file or directory still exists)."),
        }
    }
}

impl Error for RecycleError {}
