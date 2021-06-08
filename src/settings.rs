/*
*   Copyright (c) 2021 trickybestia

*   This program is free software: you can redistribute it and/or modify
*   it under the terms of the GNU General Public License as published by
*   the Free Software Foundation, either version 3 of the License, or
*   (at your option) any later version.

*   This program is distributed in the hope that it will be useful,
*   but WITHOUT ANY WARRANTY; without even the implied warranty of
*   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
*   GNU General Public License for more details.

*   You should have received a copy of the GNU General Public License
*   along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

use crate::keys::Keys;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::{io::Write, time::Duration};
use strum::EnumIter;

#[derive(PartialEq, EnumIter, Copy, Clone, Deserialize, Serialize)]
pub enum ClickType {
    Left,
    Right,
    Middle,
}

impl Default for ClickType {
    fn default() -> Self {
        ClickType::Left
    }
}

impl fmt::Display for ClickType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ClickType::Left => "Left",
                ClickType::Right => "Right",
                ClickType::Middle => "Middle",
            }
        )
    }
}

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Settings {
    pub activation_key: Option<Keys>,
    pub click_delay: Duration,
    pub click_duration: Duration,
    pub click_type: ClickType,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            activation_key: None,
            click_delay: Duration::from_millis(100),
            click_duration: Duration::from_millis(0),
            click_type: ClickType::Left,
        }
    }
}

impl Settings {
    fn get_config_directory_path() -> PathBuf {
        ProjectDirs::from("com.github", "trickybestia", "rsautoclicker")
            .unwrap()
            .config_dir()
            .to_path_buf()
    }

    fn get_config_file_path() -> PathBuf {
        Self::get_config_directory_path().join("config.json")
    }

    pub fn save(&self) {
        let config_file_path_buf = Self::get_config_file_path();
        std::fs::create_dir_all(Self::get_config_directory_path().as_path()).unwrap();
        File::create(config_file_path_buf.as_path())
            .unwrap()
            .write_all(serde_json::to_string(self).unwrap().as_bytes())
            .unwrap();
    }

    pub fn load_or_default() -> Self {
        if let Ok(settings) = Self::load() {
            settings
        } else {
            Default::default()
        }
    }

    pub fn load() -> Result<Self, ()> {
        let path_buf = Self::get_config_file_path();
        if let Ok(mut file) = File::open(path_buf.as_path()) {
            let mut buffer = String::new();
            if let Ok(_) = file.read_to_string(&mut buffer) {
                if let Ok(settings) = serde_json::from_str(buffer.as_str()) {
                    return Ok(settings);
                }
            }
        }
        Err(())
    }
}
