// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

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

        // The winresource crate automatically adds metadata from Cargo.toml
        res.compile().unwrap();
    }
    println!("cargo:rerun-if-changed=build.rs");
}
