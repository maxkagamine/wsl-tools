// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

use std::process::Command;

use winresource::WindowsResource;

fn main() {
    if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
        let mut res = WindowsResource::new();

        // This prevents the recycling progress dialog from appearing blurry
        res.set_manifest(r#"
<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" xmlns:asmv3="urn:schemas-microsoft-com:asm.v3" manifestVersion="1.0">
  <asmv3:application>
    <asmv3:windowsSettings>
      <dpiAware xmlns="http://schemas.microsoft.com/SMI/2005/WindowsSettings">true/pm</dpiAware>
      <dpiAwareness xmlns="http://schemas.microsoft.com/SMI/2016/WindowsSettings">permonitorv2,permonitor</dpiAwareness>
    </asmv3:windowsSettings>
  </asmv3:application>
    <dependency>
        <dependentAssembly>
            <assemblyIdentity
                type="win32"
                name="Microsoft.Windows.Common-Controls"
                version="6.0.0.0"
                processorArchitecture="*"
                publicKeyToken="6595b64144ccf1df"
                language="*"/>
        </dependentAssembly>
    </dependency>
</assembly>
"#);

        // Add commit hash to version
        let version = std::env::var("CARGO_PKG_VERSION").unwrap();
        let git_hash = {
            let output = Command::new("git")
                .arg("rev-parse")
                .arg("HEAD")
                .output()
                .unwrap();
            String::from_utf8(output.stdout).unwrap().trim().to_string()
        };
        res.set("ProductVersion", format!("{version}+{git_hash}").as_str());

        res.compile().unwrap();
    }
    println!("cargo:rerun-if-changed=build.rs");
}
