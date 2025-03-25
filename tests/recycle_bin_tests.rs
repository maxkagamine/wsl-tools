// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
};
use windows::{Win32::Foundation::ERROR_INVALID_PARAMETER, core::HRESULT};
use wsl_tools::recycle_bin::{self, RecycleError};

// This is merely a smoke test; we would need a way to click on dialog buttons in order to fully
// test the error handling.

#[test]
fn recycles_relative_paths() -> Result<(), Box<dyn Error>> {
    // Create test files
    let temp_dir = env::temp_dir();
    let subdir = temp_dir.join("recycles_relative_paths");
    let subdir_file = subdir.join("file.txt");
    let curdir_file = temp_dir.join("recycles_relative_paths.txt");

    fs::create_dir_all(&subdir)?;
    drop(File::create(&subdir_file)?);
    drop(File::create(&curdir_file)?);

    assert!(fs::exists(&subdir_file)?);
    assert!(fs::exists(&curdir_file)?);

    // Set current directory to test relative paths
    env::set_current_dir(&temp_dir)?;

    // Try to recycle them
    recycle_bin::recycle([
        r"recycles_relative_paths\file.txt",
        "recycles_relative_paths.txt",
    ])?;

    // Check that they were removed
    assert!(!fs::exists(&subdir_file)?);
    assert!(!fs::exists(&curdir_file)?);

    // Cleanup
    fs::remove_dir(&subdir)?;

    Ok(())
}

#[test]
fn recycles_absolute_paths() -> Result<(), Box<dyn Error>> {
    // Create test file & directory
    let temp_dir = env::temp_dir();
    assert!(Path::is_absolute(&temp_dir));

    let subdir = temp_dir.join("recycles_absolute_paths");
    let curdir_file = temp_dir.join("recycles_absolute_paths.txt");

    fs::create_dir_all(&subdir)?;
    drop(File::create(&curdir_file)?);

    assert!(fs::exists(&subdir)?);
    assert!(fs::exists(&curdir_file)?);

    // Try to recycle them
    recycle_bin::recycle([subdir.to_str().unwrap(), curdir_file.to_str().unwrap()])?;

    // Check that they were removed
    assert!(!fs::exists(&subdir)?);
    assert!(!fs::exists(&curdir_file)?);

    Ok(())
}

#[test]
fn supports_unicode() -> Result<(), Box<dyn Error>> {
    // Create test file
    let temp_dir = env::temp_dir();
    let file = temp_dir.join("ユニコードを対応する.テスト");

    drop(File::create(&file)?);

    assert!(fs::exists(&file)?);

    // Try to recycle it
    recycle_bin::recycle([file.to_str().unwrap()])?;

    // Check that it was removed
    assert!(!fs::exists(&file)?);

    Ok(())
}

#[test]
fn errors_if_not_found() {
    let file = "does-not-exist.txt";
    assert!(!fs::exists(file).unwrap());

    let err = recycle_bin::recycle([file]).expect_err("should not have succeeded");

    if let RecycleError::NotFound(str) = err {
        assert_eq!(str, file);
    } else {
        panic!("was expecting NotFound but got {err:?}");
    }
}

#[test]
fn errors_if_invalid_path() {
    // std::path::absolute errors if empty string
    let err = recycle_bin::recycle([""]).expect_err("empty string should not have succeeded");

    assert!(
        matches!(err, RecycleError::InvalidPath(_, ref inner) if inner.is::<std::io::Error>()),
        "was expecting InvalidPath(_, io::Error) for empty string but got {err:?}"
    );

    // SHCreateItemFromParsingName errors if the path contains invalid characters
    let err = recycle_bin::recycle(["foo?"]).expect_err("foo? should not have succeeded");

    match err {
        RecycleError::InvalidPath(_, ref inner) if inner.is::<windows::core::Error>() => {
            let win32 = inner.downcast_ref::<windows::core::Error>().unwrap();
            assert_eq!(win32.code(), HRESULT::from_win32(ERROR_INVALID_PARAMETER.0));
        }
        _ => panic!("was expecting InvalidPath(_, windows::core::Error) for foo? but got {err:?}"),
    }
}
