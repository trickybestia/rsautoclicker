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

use super::settings_editor::SettingsEditor;
use crate::{keyboard_hook::KeyboardHook, settings::Settings};
use nwd::NwgUi;
use nwg::{GridLayout, Icon, Menu, MenuItem, TextInput, Window};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};

#[derive(NwgUi)]
pub struct App {
    settings: RefCell<Settings>,

    on_settings_changed: Box<dyn Fn(&Settings)>,

    keyboard_hook: Arc<Mutex<KeyboardHook>>,

    #[nwg_resource(source_bin: Some(include_bytes!("../../resources/icon.ico")))]
    icon: Icon,

    #[nwg_control(title: "RS Autoclicker", flags: "WINDOW|VISIBLE", size: (250, 60), icon: Some(&data.icon))]
    #[nwg_events(OnWindowClose: [App::on_close], OnInit: [App::update])]
    window: Window,

    #[nwg_control(text: "Tools", parent: window)]
    tools_menu: Menu,

    #[nwg_control(text: "Options...", parent: tools_menu)]
    #[nwg_events(OnMenuItemSelected: [App::on_options_menu_click])]
    options_menu: MenuItem,

    #[nwg_layout(parent: window, max_row: Some(1))]
    layout: GridLayout,

    #[nwg_control(text: "Open Tools->Options...", readonly: true, align: HTextAlign::Center)]
    #[nwg_layout_item(layout: layout, col: 0, row: 0)]
    selected_key_text_input: TextInput,
}

impl App {
    pub fn new(
        settings: Settings,
        on_settings_changed: Box<dyn Fn(&Settings)>,
        keyboard_hook: Arc<Mutex<KeyboardHook>>,
    ) -> Self {
        Self {
            settings: RefCell::new(settings),
            on_settings_changed,
            keyboard_hook,
            icon: Default::default(),
            window: Default::default(),
            tools_menu: Default::default(),
            options_menu: Default::default(),
            layout: Default::default(),
            selected_key_text_input: Default::default(),
        }
    }

    fn update(&self) {
        if let Some(activation_key_code) = self.settings.borrow().activation_key {
            self.selected_key_text_input.set_text(&format!(
                "Press '{}' to toggle",
                activation_key_code.to_string()
            ));
        }
    }

    fn on_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn on_options_menu_click(&self) {
        self.keyboard_hook.lock().unwrap().stop();
        self.window.set_visible(false);
        self.settings
            .replace(SettingsEditor::show(self.settings.take()));
        self.update();
        (self.on_settings_changed)(&self.settings.borrow());
        self.window.set_visible(true);
        self.keyboard_hook.lock().unwrap().start();
    }
}
