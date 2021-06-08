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

use crate::keyboard_hook::KeyboardHook;
use crate::settings::{ClickType, Settings};
use num_traits::FromPrimitive;
use nwd::NwgUi;
use nwg::{Button, ComboBox, GridLayout, Icon, NativeUi, Notice, TextInput, Tooltip, Window};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use strum::IntoEnumIterator;

#[derive(NwgUi)]
pub struct SettingsEditor {
    settings: Arc<Mutex<Settings>>,

    is_changing_activator_key: Arc<Mutex<bool>>,

    keyboard_hook: RefCell<Option<KeyboardHook>>,

    tooltip: Tooltip,

    #[nwg_control(title: "RS Autoclicker Configuration", flags: "WINDOW|VISIBLE", size: (250, 200))]
    #[nwg_events(OnWindowClose: [SettingsEditor::on_close], OnInit: [SettingsEditor::on_init])]
    window: Window,

    #[nwg_layout(parent: window, max_row: Some(4), max_column: Some(1))]
    layout: GridLayout,

    #[nwg_control]
    #[nwg_events(OnNotice: [SettingsEditor::on_click_activator_update_text_notice])]
    click_activator_update_text_notice: Notice,

    #[nwg_control]
    #[nwg_events(OnComboxBoxSelection: [SettingsEditor::on_click_type_selected])]
    #[nwg_layout_item(layout: layout, col: 0, row: 0)]
    click_type_selector: ComboBox<ClickType>,

    #[nwg_control(text: &data.settings.lock().unwrap().click_delay.as_millis().to_string())]
    #[nwg_events(OnTextInput: [SettingsEditor::on_click_delay_changed])]
    #[nwg_layout_item(layout: layout, col: 0, row: 1)]
    click_delay_selector: TextInput,

    #[nwg_control(text: &data.settings.lock().unwrap().click_duration.as_millis().to_string())]
    #[nwg_events(OnTextInput: [SettingsEditor::on_click_duration_changed])]
    #[nwg_layout_item(layout: layout, col: 0, row: 2)]
    click_duration_selector: TextInput,

    #[nwg_control(text: &match data.settings.lock().unwrap().activation_key{
        Some(activation_key) => activation_key.to_string(),
        None => "No key selected".to_string(),
    })]
    #[nwg_events(OnButtonClick: [SettingsEditor::on_click_activator_click])]
    #[nwg_layout_item(layout: layout, col: 0, row: 3)]
    click_activator_button: Button,
}

impl SettingsEditor {
    pub fn show(settings: Settings) -> Settings {
        thread::spawn(move || {
            let settings_editor = SettingsEditor::build_ui(Self::new(settings)).unwrap();
            nwg::dispatch_thread_events();
            let lock = settings_editor.settings.lock().unwrap();
            lock.clone()
        })
        .join()
        .unwrap()
    }

    fn new(settings: Settings) -> Self {
        let mut tooltip = Default::default();
        Tooltip::builder().build(&mut tooltip).unwrap();
        Self {
            settings: Arc::new(Mutex::new(settings)),
            is_changing_activator_key: Arc::new(Mutex::new(false)),
            keyboard_hook: RefCell::new(None),
            tooltip,
            window: Default::default(),
            layout: Default::default(),
            click_activator_update_text_notice: Default::default(),
            click_type_selector: Default::default(),
            click_delay_selector: Default::default(),
            click_duration_selector: Default::default(),
            click_activator_button: Default::default(),
        }
    }

    fn on_init(&self) {
        self.window.set_icon(Some(
            &Icon::from_bin(include_bytes!("../../resources/icon.ico")).unwrap(),
        ));

        for click_type in ClickType::iter() {
            self.click_type_selector.push(click_type);
            if click_type == self.settings.lock().unwrap().click_type {
                self.click_type_selector
                    .set_selection(Some(self.click_type_selector.len() - 1));
            }
        }

        self.tooltip
            .register(&self.click_delay_selector, "Delay between clicks, ms");
        self.tooltip
            .register(&self.click_type_selector, "Click type");
        self.tooltip
            .register(&self.click_duration_selector, "Duration of click, ms");
        self.tooltip
            .register(&self.click_activator_button, "Clicker activation key");

        let mut keyboard_hook = KeyboardHook::new(Arc::new({
            let settings = self.settings.clone();
            let is_changing_activator_key = self.is_changing_activator_key.clone();
            let sender = self.click_activator_update_text_notice.sender();

            move |vk_code| {
                let mut lock = is_changing_activator_key.lock().unwrap();
                if *lock {
                    if let Some(key) = FromPrimitive::from_u32(vk_code) {
                        settings.lock().unwrap().activation_key = Some(key);
                        *lock = false;
                        sender.notice();
                    }
                }
            }
        }));
        keyboard_hook.start();
        *self.keyboard_hook.borrow_mut() = Some(keyboard_hook);
    }

    fn on_close(&self) {
        nwg::stop_thread_dispatch();
    }

    fn on_click_type_selected(&self) {
        self.settings.lock().unwrap().click_type =
            self.click_type_selector.collection()[self.click_type_selector.selection().unwrap()];
    }

    fn on_click_delay_changed(&self) {
        if let Ok(new_delay) = self.click_delay_selector.text().parse::<u64>() {
            self.settings.lock().unwrap().click_delay = Duration::from_millis(new_delay);
        }
    }

    fn on_click_duration_changed(&self) {
        if let Ok(new_duration) = self.click_duration_selector.text().parse::<u64>() {
            self.settings.lock().unwrap().click_duration = Duration::from_millis(new_duration);
        }
    }

    fn on_click_activator_click(&self) {
        if let Ok(mut lock) = self.is_changing_activator_key.try_lock() {
            if !*lock {
                self.click_activator_button.set_text("Press key...");
                *lock = true;
            }
        }
    }

    fn on_click_activator_update_text_notice(&self) {
        let lock = self.is_changing_activator_key.lock().unwrap();
        if !*lock {
            let lock = self.settings.lock().unwrap();
            self.click_activator_button
                .set_text(&lock.activation_key.unwrap().to_string());
        } else {
            self.click_activator_button.set_text("Press key...");
        }
    }
}
