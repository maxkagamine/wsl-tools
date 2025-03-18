// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use wsl_tools::recycle_bin;

#[test]
fn does_thing() {
    recycle_bin::recycle([r"C:\Users\max\Downloads\test file.txt"]).unwrap();
}
