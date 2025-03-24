// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#[cfg(windows)]
fn main() {
    use std::time::Instant;
    use wsl_tools::recycle_bin;

    let files = (1..=10000).map(|i| format!("C:\\Users\\max\\Downloads\\test-files\\{i}.bin"));
    let time = Instant::now();

    {
        recycle_bin::initialize_com();
        recycle_bin::recycle_batch(files).unwrap();
    }

    let dur = time.elapsed();
    println!("took {} ns", dur.as_nanos());
}

#[cfg(unix)]
fn main() {
    todo!();
}
