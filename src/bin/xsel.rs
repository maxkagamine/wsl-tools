// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#[cfg(windows)]
fn main() -> Result<(), windows::core::Error> {
    // wsl_tools::clipboard::set_text("鏡音リン")
    match wsl_tools::clipboard::get_text()? {
        Some(text) => println!("*{text}*"),
        None => println!("No text on clipboard"),
    };
    Ok(())
}

#[cfg(unix)]
fn main() {
    unimplemented!();
}
