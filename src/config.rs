/* MIT License
 *
 * Copyright (c) 2026 FatDawlf
 *
 * SPDX-License-Identifier: MIT
 */

use serde::Deserialize;
use std::{
    env::var,
    eprintln,
    fs::{self, File},
    io::{Read, Write},
};

// COMPILE TIME CONSTANTS
pub static GETTEXT_PACKAGE: &str = "pikolaunch";
pub static LOCALEDIR: &str = "/app/share/locale";

pub static DEFAULT_CONFIG: &str = include_str!("../data/default.toml");

// Configuration file definition
// Each section should implement default themselves
#[derive(Default, Debug, Deserialize)]
pub struct PikolaunchConfig {
    // Aesthetic
    #[serde(rename = "Aesthetic")]
    pub aesthetic: Aesthetic,
    // Functionality
    #[serde(rename = "Behavior")]
    pub beavior: Behavior,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Aesthetic {
    pub opacity: f32,
    pub radius: u32,
    pub entries: u32,
    pub entry_size: u32,
}

impl Default for Aesthetic {
    fn default() -> Self {
        Self {
            opacity: 1.0,
            radius: 12,
            entries: 5,
            entry_size: 64,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Behavior {
    pub close_when_unfocused: bool,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            close_when_unfocused: true,
        }
    }
}

impl PikolaunchConfig {
    pub fn load_config() -> Self {
        let path = config_path();

        let file = File::open(path);

        match file {
            Ok(mut f) => {
                let mut string = String::new();
                match f.read_to_string(&mut string) {
                    Ok(_) => {
                        let conf: PikolaunchConfig = match toml::from_str(&string) {
                            Ok(c) => c,
                            Err(e) => {
                                eprintln!("[Error] Failed to parse config file: {}", e);
                                Self::default()
                            }
                        };
                        conf
                    }
                    Err(e) => {
                        eprintln!("[Error] Failed to read config file: {}", e);
                        Self::default()
                    }
                }
            }
            Err(_) => {
                Self::save_default_config();

                Self::default()
            }
        }
    }

    fn save_default_config() {
        let path = config_path();

        match fs::create_dir_all(&path.replace("conf.toml", "")) {
            Ok(_) => {
                let file = File::create(&path);

                match file {
                    Ok(mut f) => {
                        match f.write_all(DEFAULT_CONFIG.as_bytes()) {
                            Ok(_) => {
                                println!("[Info] Successfuly generated default config at {}", path);
                            }
                            Err(e) => {
                                eprintln!(
                                    "[Error] Couldn't generate a new config at {}: {}",
                                    path, e
                                )
                            }
                        };
                    }
                    Err(e) => {
                        eprintln!("[Error] Failed to create a config file at {}: {}", path, e);
                    }
                };
            }
            Err(e) => {
                eprintln!("[Error] Failed to create config folder at {}: {}", path, e);
            }
        };
    }
}

fn config_path() -> String {
    let config_path = match var("XDG_CONFIG_HOME") {
        Ok(s) => s,
        Err(_) => match var("HOME") {
            Ok(s) => format!("{}/.config", s),
            Err(e) => {
                eprintln!("[Error] Could not find a suitable config path: {}", e);
                return "/etc/".to_string();
            }
        },
    };

    return format!("{}/pikolaunch/conf.toml", config_path);
}
