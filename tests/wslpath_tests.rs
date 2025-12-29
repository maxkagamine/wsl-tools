// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use std::{env, process::Command};
use wsl_tools::wslpath;

fn cd_to_users() {
    // This is a path that's pretty much guaranteed to exist in any Windows/WSL environment
    env::set_current_dir(if cfg!(unix) {
        "/mnt/c/Users"
    } else {
        r"C:\Users"
    })
    .expect("failed to set current directory");
}

fn get_distro_name() -> String {
    if cfg!(unix) {
        env::var("WSL_DISTRO_NAME").unwrap()
    } else {
        let output = Command::new("wsl.exe")
            .arg("echo")
            .arg("$WSL_DISTRO_NAME")
            .output()
            .unwrap();
        String::from_utf8(output.stdout).unwrap().trim().to_string()
    }
}

#[test]
fn wsl_to_windows() {
    cd_to_users();

    let path = wslpath::to_windows("foo/bar").unwrap();
    assert_eq!(path, r"C:\Users\foo\bar");

    let path = wslpath::to_windows("/home").unwrap();
    assert_eq!(
        path,
        format!("\\\\wsl.localhost\\{}\\home", get_distro_name())
    );
}

#[test]
fn windows_to_wsl() {
    cd_to_users();

    let path = wslpath::to_wsl(r"foo\bar").unwrap();
    assert_eq!(path, "/mnt/c/Users/foo/bar");

    let path = wslpath::to_wsl(format!("\\\\wsl.localhost\\{}\\home", get_distro_name())).unwrap();
    assert_eq!(path, "/home");
}

#[test]
#[cfg(unix)]
fn symlink_wsl_to_windows() -> Result<(), Box<dyn std::error::Error>> {
    use std::{fs, os::unix::fs::symlink};

    let temp_dir = env::temp_dir();
    let subdir = temp_dir.join("symlink_wsl_to_windows");
    let subdir_symlink = subdir.join("symlink");
    let symlink_in_symlink = subdir_symlink.join("symlink");

    let _ = fs::remove_dir_all(&subdir);
    fs::create_dir(&subdir)?;
    symlink(&subdir, &subdir_symlink)?;

    let expected_windows_subdir = wslpath::to_windows(&subdir)?;
    let expected_windows_subdir_symlink = format!("{expected_windows_subdir}\\symlink");

    // Sanity check: Just calling wslpath -w will resolve all symlinks
    let actual_windows_subdir = wslpath::to_windows(&symlink_in_symlink)?;
    assert_eq!(actual_windows_subdir, expected_windows_subdir);

    // Using symlink_to_windows should give us a path to the symlink itself, albeit with the
    // directory path resolved
    let actual_windows_subdir_symlink = wslpath::symlink_to_windows(&symlink_in_symlink)?;
    assert_eq!(
        actual_windows_subdir_symlink,
        expected_windows_subdir_symlink
    );

    // Cleanup
    fs::remove_dir_all(&subdir)?;
    Ok(())
}
