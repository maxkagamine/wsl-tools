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
