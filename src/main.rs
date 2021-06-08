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

#![windows_subsystem = "windows"]

mod clicker;
mod gui;
mod keyboard_hook;
mod keys;
mod resources;
mod settings;

use clicker::{Clicker, ClickerStatus};
use gui::App;
use keyboard_hook::KeyboardHook;
use nwg::NativeUi;
use settings::Settings;
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    let settings = Arc::new(Mutex::new(Settings::load_or_default()));
    let clicker = Clicker::new(settings.lock().unwrap().clone());
    let keyboard_hook = Arc::new(Mutex::new({
        let settings = settings.clone();
        let clicker = clicker.clone();
        KeyboardHook::new(Arc::new(move |vk_code: u32| {
            if let Some(activation_key_code) = settings.lock().unwrap().activation_key {
                if vk_code == activation_key_code as u32 {
                    match clicker.status() {
                        ClickerStatus::Clicking => clicker.stop(),
                        ClickerStatus::Idle => clicker.start(),
                    }
                }
            }
        }))
    }));
    keyboard_hook.lock().unwrap().start();

    nwg::init().unwrap();
    nwg::Font::set_global_family("Segoe UI").unwrap();

    let _app = App::build_ui(App::new(
        settings.lock().unwrap().clone(),
        Box::new({
            let settings = settings.clone();
            let clicker = clicker.clone();
            move |changed_settings| {
                *settings.lock().unwrap() = changed_settings.clone();
                clicker.set_settings(changed_settings.clone());
            }
        }),
        keyboard_hook.clone(),
    ))
    .unwrap();

    nwg::dispatch_thread_events();

    settings.lock().unwrap().save();
}
