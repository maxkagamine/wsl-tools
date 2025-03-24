// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use embed_manifest::{
    embed_manifest, empty_manifest,
    manifest::{AssemblyIdentity, DpiAwareness},
};

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        // This prevents the recycling progress dialog from appearing blurry
        embed_manifest(
            empty_manifest()
                .dependency(AssemblyIdentity::new(
                    "Microsoft.Windows.Common-Controls",
                    [6, 0, 0, 0],
                    0x6595_b641_44cc_f1df,
                ))
                .dpi_awareness(DpiAwareness::PerMonitorV2),
        )
        .expect("unable to embed manifest file");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
