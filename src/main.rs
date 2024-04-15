use std::ptr;
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use winapi::shared::minwindef::{LPARAM, LRESULT, WPARAM};
use winapi::um::winuser::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, SendMessageW, SetWindowsHookExW, TranslateMessage, UnhookWindowsHookEx, KBDLLHOOKSTRUCT, VK_LWIN, VK_RWIN, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP, WM_MBUTTONDOWN, WM_MBUTTONUP
};

type LowLevelKeyboardProc = unsafe extern "system" fn(nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;

unsafe extern "system" fn kbd_hook(n_code: i32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    if n_code >= 0 {
        let hook_struct = *(l_param as *const KBDLLHOOKSTRUCT);
        let key_code = hook_struct.vkCode;

        if key_code == VK_LWIN as _ || key_code == VK_RWIN as _ {
            let foreground_window = GetForegroundWindow();
            let mut window_title = String::new();
            let title_length = GetWindowTextLengthW(foreground_window) + 1;
            let mut buffer: Vec<u16> = Vec::with_capacity(title_length as usize);
            GetWindowTextW(foreground_window, buffer.as_mut_ptr(), title_length);
            buffer.set_len((title_length - 1) as usize);
            window_title.push_str(&OsString::from_wide(&buffer).to_string_lossy());

            if window_title.contains("Blender") && IsWindowVisible(foreground_window) != 0 {
                if w_param == WM_KEYDOWN as usize {
                    SendMessageW(foreground_window, WM_MBUTTONDOWN, 0, 0);
                } else if w_param == WM_KEYUP as usize {
                    SendMessageW(foreground_window, WM_MBUTTONUP, 0, 0);
                }

                //consume event
                //dbg!("Detected os key");
                return 1;
            }
        }
    }

    CallNextHookEx(ptr::null_mut(), n_code, w_param, l_param)
}

fn main() {
    unsafe {
        let hook_proc: LowLevelKeyboardProc = kbd_hook;
        let hook_id = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), ptr::null_mut(), 0);

        if hook_id.is_null() {
            panic!("Keyboard hook failed");
        }

        let mut msg = std::mem::zeroed();
        loop {
            GetMessageW(&mut msg, ptr::null_mut(), 0, 0);
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        //unreachable!();
        //UnhookWindowsHookEx(hook_id);
    }
}