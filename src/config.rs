// Copyright (c) Max Kagamine
// Licensed under the Apache License, Version 2.0

#![cfg(unix)]

use ini::Ini;
use std::{env, sync::OnceLock};

const CONFIG_FILENAME: &str = "wsl-tools.ini";

pub struct Config {
    pub ini_exists: bool,
    pub use_linux_trash: bool,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(load_config)
}

fn load_config() -> Config {
    let ini = env::current_exe()
        .ok()
        .and_then(|p| Some(p.parent()?.join(CONFIG_FILENAME)))
        .and_then(|p| Ini::load_from_file(p).ok());

    Config {
        ini_exists: ini.is_some(),
        use_linux_trash: get_bool(ini.as_ref(), "use_linux_trash", false),
    }
}

fn get_bool(ini: Option<&Ini>, key: &str, default: bool) -> bool {
    let value = ini.and_then(|x| x.get_from(Some("config"), key));
    match value {
        Some("yes") => true,
        Some("no") => false,
        _ => default,
    }
}
