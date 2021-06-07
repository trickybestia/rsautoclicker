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

use lazy_static::lazy_static;
use std::mem;
use std::ptr;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use winapi::shared::windef::HHOOK;
use winapi::um::{libloaderapi, winuser};

lazy_static! {
    static ref HOOK_HANDLE: Mutex<Option<usize>> = Mutex::new(None);
    static ref CALLBACKS: Mutex<Vec<Arc<dyn Fn(u32) + Send + Sync + 'static>>> =
        Mutex::new(Vec::new());
}

pub struct KeyboardHook {
    callback: Arc<dyn Fn(u32) + Send + Sync + 'static>,
}

impl Drop for KeyboardHook {
    fn drop(&mut self) {
        let mut lock = CALLBACKS.lock().unwrap();
        let position = lock
            .iter()
            .position(|callback| Arc::as_ptr(callback) == Arc::as_ptr(&self.callback))
            .unwrap();
        lock.remove(position);
    }
}

impl KeyboardHook {
    fn init() {
        let mut hook_handle_lock = HOOK_HANDLE.lock().unwrap();
        if hook_handle_lock.is_none() {
            let (mutex, condvar) = (Arc::new(Mutex::new(None)), Arc::new(Condvar::new()));
            let (cloned_mutex, cloned_condvar) = (mutex.clone(), condvar.clone());
            thread::spawn(move || {
                {
                    *cloned_mutex.lock().unwrap() = Some(unsafe {
                        winuser::SetWindowsHookExA(
                            winuser::WH_KEYBOARD_LL,
                            Some(Self::callback),
                            libloaderapi::LoadLibraryA("User32".as_ptr() as *const i8),
                            0,
                        ) as usize
                    });
                    cloned_condvar.notify_one();
                }
                unsafe {
                    let mut msg: winuser::MSG = mem::zeroed();
                    while winuser::GetMessageA(&mut msg, ptr::null_mut(), 0, 0) != 0 {
                        winuser::TranslateMessage(&msg);
                        winuser::DispatchMessageA(&msg);
                    }
                }
            });
            let mut lock = mutex.lock().unwrap();
            while lock.is_none() {
                lock = condvar.wait(lock).unwrap();
            }
            *hook_handle_lock = *lock;
        }
    }

    pub fn new(callback: Arc<dyn Fn(u32) + Send + Sync + 'static>) -> Self {
        Self::init();
        CALLBACKS.lock().unwrap().push(callback.clone());
        Self { callback }
    }

    unsafe extern "system" fn callback(n_code: i32, w_param: usize, l_param: isize) -> isize {
        let key_info: winuser::KBDLLHOOKSTRUCT = *(l_param as *const winuser::KBDLLHOOKSTRUCT);
        if (n_code == winuser::HC_ACTION)
            && ((w_param == winuser::WM_SYSKEYDOWN as usize)
                || (w_param == winuser::WM_KEYDOWN as usize))
        {
            for callback in CALLBACKS.lock().unwrap().iter() {
                callback(key_info.vkCode);
            }
        }
        winuser::CallNextHookEx(
            HOOK_HANDLE.lock().unwrap().unwrap() as HHOOK,
            n_code,
            w_param,
            l_param,
        )
    }
}
