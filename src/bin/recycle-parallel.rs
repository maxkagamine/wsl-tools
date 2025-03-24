// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#[cfg(windows)]
fn main() {
    use std::time::Instant;
    use tokio::{runtime::Builder, task::JoinSet};
    use wsl_tools::recycle_bin;

    Builder::new_current_thread()
        .on_thread_start(recycle_bin::initialize_com)
        .build()
        .unwrap()
        .block_on(async {
            let files =
                (1..=10000).map(|i| format!("C:\\Users\\max\\Downloads\\test-files\\{i}.bin"));
            let time = Instant::now();

            {
                let mut tasks = JoinSet::new();

                for f in files {
                    tasks.spawn_blocking(|| {
                        recycle_bin::recycle(f).unwrap();
                    });
                }

                tasks.join_all().await;
                // while let Some(res) = tasks.join_next().await {
                //     res.unwrap();
                // }
            }

            let dur = time.elapsed();
            println!("took {} ns", dur.as_nanos());
        });
}

#[cfg(unix)]
fn main() {
    todo!();
}
