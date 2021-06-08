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

use crate::settings::{ClickType, Settings};
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

macro_rules! status_internal {
    ($mutex_guard:expr) => {
        if $mutex_guard.is_some() {
            ClickerStatus::Clicking
        } else {
            ClickerStatus::Idle
        }
    };
}

pub enum ClickerStatus {
    Clicking,
    Idle,
}

enum ClickerMessage {
    Stop,
}

pub struct Clicker {
    thread_info: Mutex<Option<(mpsc::Sender<ClickerMessage>, thread::JoinHandle<()>)>>,
    settings: Mutex<Settings>,
}

impl Drop for Clicker {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Clicker {
    pub fn new(settings: Settings) -> Arc<Self> {
        Arc::new(Clicker {
            thread_info: Mutex::new(None),
            settings: Mutex::new(settings),
        })
    }

    pub fn start(self: &Arc<Self>) {
        let mut lock = self.thread_info.lock().unwrap();
        if matches!(status_internal!(lock), ClickerStatus::Idle) {
            let cloned_self = self.clone();
            let (sender, receiver) = mpsc::channel();
            *lock = Some((
                sender,
                thread::spawn(move || loop {
                    if let Ok(message) = receiver.try_recv() {
                        match message {
                            ClickerMessage::Stop => break,
                        }
                    }
                    let settings;
                    {
                        settings = cloned_self.settings.lock().unwrap().clone();
                    }
                    click(settings.click_type, settings.click_duration);
                    thread::sleep(settings.click_delay);
                }),
            ));
        }
    }

    pub fn stop(&self) {
        let mut lock = self.thread_info.lock().unwrap();
        if matches!(status_internal!(lock), ClickerStatus::Clicking) {
            let (sender, join_handle) = lock.take().unwrap();
            sender.send(ClickerMessage::Stop).unwrap();
            join_handle.join().unwrap();
        }
    }

    pub fn set_settings(&self, settings: Settings) {
        let mut lock = self.settings.lock().unwrap();
        *lock = settings;
    }

    pub fn status(&self) -> ClickerStatus {
        status_internal!(self.thread_info.lock().unwrap())
    }
}

fn click(click_type: ClickType, duration: Duration) {
    use winapi::um::winuser;

    let (first_event_type, second_event_type) = match click_type {
        ClickType::Left => (winuser::MOUSEEVENTF_LEFTDOWN, winuser::MOUSEEVENTF_LEFTUP),
        ClickType::Right => (winuser::MOUSEEVENTF_RIGHTDOWN, winuser::MOUSEEVENTF_RIGHTUP),
        ClickType::Middle => (
            winuser::MOUSEEVENTF_MIDDLEDOWN,
            winuser::MOUSEEVENTF_MIDDLEUP,
        ),
    };

    unsafe {
        winuser::mouse_event(first_event_type, 0, 0, 0, 0);
    }
    thread::sleep(duration);
    unsafe {
        winuser::mouse_event(second_event_type, 0, 0, 0, 0);
    }
}
