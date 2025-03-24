// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(windows)]

use wsl_tools::recycle_bin;

#[test]
fn does_thing() {
    // recycle_bin::recycle(r"C:\Users\max\Downloads\c.txt").expect("recycle c.txt");
    recycle_bin::initialize_com();

    println!("c.txt");
    if let Err(err) = recycle_bin::recycle(r"C:\Users\max\Downloads\c.txt") {
        println!("error: {err:?}");
    }
    println!("s.txt");
    if let Err(err) = recycle_bin::recycle(r"S:\s.txt") {
        println!("error: {err:?}");
    }
    println!("20gb.bin");
    if let Err(err) = recycle_bin::recycle(r"D:\20gb.bin") {
        println!("error: {err:?}");
    }

    panic!("done");
}
