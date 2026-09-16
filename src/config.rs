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
    io::Write,
};

use crate::DEFAULT_CONFIG;

// Configuration file definition
// Each section should implement default themselves
#[derive(Default, Debug, Deserialize)]
pub struct LupaConfig {
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
    pub window_color: String,
    pub opacity: f32,
    pub entry_elevation: f32,
    pub radius: u32,
    pub entries: f32,
    pub entry_size: u32,
    pub anchors: String,
}

impl Default for Aesthetic {
    fn default() -> Self {
        Self {
            window_color: "var(--window-bg-color)".to_owned(),
            opacity: 1.0,
            entry_elevation: 1.2,
            radius: 15,
            entries: 5.0,
            entry_size: 64,
            anchors: String::new(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct Behavior {
    pub close_when_unfocused: bool,
    pub max_file_entries: u32,
    pub fallback_providers: String,
}

impl Default for Behavior {
    fn default() -> Self {
        Self {
            close_when_unfocused: true,
            max_file_entries: 25,
            fallback_providers: "system, app, file, emoji, calc".to_string(),
        }
    }
}

impl LupaConfig {
    pub fn load_config() -> Self {
        let path = config_path();

        match fs::read_to_string(path) {
            Ok(buf) => {
                let conf: LupaConfig = match toml::from_str(&buf) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("[Error] Failed to parse config file: {}", e);
                        Self::default()
                    }
                };
                conf
            }
            Err(_) => {
                Self::save_default_config();

                Self::default()
            }
        }
    }

    fn save_default_config() {
        let path = config_path();

        match fs::create_dir_all(path.replace("conf.toml", "")) {
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

fn config_folder() -> String {
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

    format!("{}/lupa", config_path)
}

pub fn plugin_path() -> String {
    format!("{}/plugins", config_folder())
}

pub fn config_path() -> String {
    format!("{}/conf.toml", config_folder())
}
