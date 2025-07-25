// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use std::{
    env,
    error::Error,
    fs::{self, File},
    path::Path,
    process::Command,
};
use windows::{Win32::Foundation::ERROR_INVALID_PARAMETER, core::HRESULT};
use wsl_tools::recycle_bin::{
    self, RECYCLE_DANGEROUSLY_IN_BACKGROUND, RECYCLE_IGNORE_NOT_FOUND, RECYCLE_NORMAL, RecycleError,
};

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
    recycle_bin::recycle(
        [
            r"recycles_relative_paths\file.txt",
            "recycles_relative_paths.txt",
        ],
        RECYCLE_NORMAL,
    )?;

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
    recycle_bin::recycle(
        [subdir.to_str().unwrap(), curdir_file.to_str().unwrap()],
        RECYCLE_NORMAL,
    )?;

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
    recycle_bin::recycle([file.to_str().unwrap()], RECYCLE_NORMAL)?;

    // Check that it was removed
    assert!(!fs::exists(&file)?);

    Ok(())
}

#[test]
fn errors_if_not_found() {
    let file = "does-not-exist.txt";
    assert!(!fs::exists(file).unwrap());

    let err = recycle_bin::recycle([file], RECYCLE_NORMAL).expect_err("should not have succeeded");

    if let RecycleError::NotFound(str) = err {
        assert_eq!(str, file);
    } else {
        panic!("was expecting NotFound but got {err:?}");
    }
}

#[test]
fn errors_if_invalid_path() {
    // std::path::absolute errors if empty string
    let err = recycle_bin::recycle([""], RECYCLE_NORMAL)
        .expect_err("empty string should not have succeeded");

    assert!(
        matches!(err, RecycleError::InvalidPath(_, ref inner) if inner.is::<std::io::Error>()),
        "was expecting InvalidPath(_, io::Error) for empty string but got {err:?}"
    );

    // SHCreateItemFromParsingName errors if the path contains invalid characters
    let err =
        recycle_bin::recycle(["foo?"], RECYCLE_NORMAL).expect_err("foo? should not have succeeded");

    match err {
        RecycleError::InvalidPath(_, ref inner) if inner.is::<windows::core::Error>() => {
            let win32 = inner.downcast_ref::<windows::core::Error>().unwrap();
            assert_eq!(win32.code(), HRESULT::from_win32(ERROR_INVALID_PARAMETER.0));
        }
        _ => panic!("was expecting InvalidPath(_, windows::core::Error) for foo? but got {err:?}"),
    }
}

#[test]
fn option_to_ignore_not_found() -> Result<(), Box<dyn Error>> {
    // Create test files
    let temp_dir = env::temp_dir();
    let exists_1 = temp_dir.join("option_to_ignore_not_found_1.test");
    let exists_2 = temp_dir.join("option_to_ignore_not_found_2.test");
    let not_exist = temp_dir.join("option_to_ignore_not_found_3.test");

    drop(File::create(&exists_1)?);
    drop(File::create(&exists_2)?);

    assert!(fs::exists(&exists_1)?);
    assert!(fs::exists(&exists_2)?);
    assert!(!fs::exists(&not_exist)?);

    recycle_bin::recycle(
        [
            exists_1.to_str().unwrap(),
            not_exist.to_str().unwrap(),
            exists_2.to_str().unwrap(),
        ],
        RECYCLE_IGNORE_NOT_FOUND,
    )
    .expect("should not have failed due to option_to_ignore_not_found_3.test missing");

    assert!(
        !fs::exists(&exists_1)? && !fs::exists(&exists_2)?,
        "the files that did exist should have been recycled"
    );

    Ok(())
}

#[test]
fn does_nothing_if_nothing_to_do() -> Result<(), Box<dyn Error>> {
    // Check with empty paths
    let empty: [&str; 0] = [];
    recycle_bin::recycle(empty, RECYCLE_NORMAL)
        .expect("if paths is empty, it should not call PerformOperations as that will cause a 'Catastrophic failure'");

    // Check with ignore not found and only nonexistent files
    let temp_dir = env::temp_dir();
    let not_exist = temp_dir.join("does_nothing_if_nothing_to_do.test");
    assert!(!fs::exists(&not_exist)?);
    recycle_bin::recycle(
        [
            not_exist.to_str().unwrap(),
        ],
        RECYCLE_IGNORE_NOT_FOUND,
    )
        .expect("if all of the paths were ignored, it should not call PerformOperations as that will cause a 'Catastrophic failure'");

    Ok(())
}

/// Creates a file in /tmp with the given name and returns its Windows path. (Using bash to keep
/// tests portable since it has the distro name in it, e.g. \\wsl.localhost\Arch\tmp\foo)
fn create_file_in_wsl(name: &str) -> String {
    let output = Command::new("bash.exe")
        .arg("-c")
        .arg(format!("touch /tmp/{name} && wslpath -aw /tmp/{name}"))
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn option_to_silently_nuke() -> Result<(), Box<dyn Error>> {
    // Create a file in a location that we know won't have a recycle bin
    let file = create_file_in_wsl("IF_DIALOG_APPEARS_PRESS_NO_TEST_HAS_FAILED");

    assert!(fs::exists(&file)?);

    // The failure mode is a bit awkward for an integration test: if the correct flags aren't set,
    // a dialog will appear and the test will block until it's dismissed.
    recycle_bin::recycle([&file], RECYCLE_DANGEROUSLY_IN_BACKGROUND)?;

    assert!(!fs::exists(&file)?);

    Ok(())
}

#[test]
fn options_can_be_combined() -> Result<(), Box<dyn Error>> {
    // Create a file in a location that we know won't have a recycle bin
    let name = "IF_DIALOG_APPEARS_PRESS_NO_TEST_HAS_FAILED-2";
    let exists = create_file_in_wsl(name);
    let not_exist = exists.replace(name, "DOES_NOT_EXIST");

    assert!(fs::exists(&exists)?);
    assert!(!fs::exists(&not_exist)?);

    recycle_bin::recycle(
        [&exists, &not_exist],
        RECYCLE_DANGEROUSLY_IN_BACKGROUND | RECYCLE_IGNORE_NOT_FOUND,
    )?;

    assert!(!fs::exists(&exists)?);

    Ok(())
}

#[test]
fn fires_callback() -> Result<(), Box<dyn Error>> {
    // Create test file
    let temp_dir = env::temp_dir();
    let file = temp_dir.join("fires_callback.test");
    drop(File::create(&file)?);

    let mut callback_fired = false;

    recycle_bin::recycle_with_callback([file.to_str().unwrap()], RECYCLE_NORMAL, |item, error| {
        assert!(!callback_fired);
        assert!(error.is_none());
        assert_eq!(file.to_str().unwrap(), item);

        callback_fired = true;
    })?;

    assert!(callback_fired);
    assert!(!fs::exists(&file)?);

    Ok(())
}
