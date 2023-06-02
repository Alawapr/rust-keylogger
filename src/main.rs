use std::ffi::CString;
use std::mem;
use std::ptr;

use winapi::ctypes::c_void;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, DispatchMessageA, GetMessageA, GetRawInputData,
    RegisterClassExA, RegisterRawInputDevices, TranslateMessage, HRAWINPUT, HWND_MESSAGE, MSG,
    RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER, RIDEV_INPUTSINK, RID_INPUT, RIM_TYPEKEYBOARD,
    WM_INPUT, WNDCLASSEXA,
};

unsafe extern "system" fn wind_proc(
    h_wnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if msg == WM_INPUT {
        let h_raw_input: HRAWINPUT = l_param as HRAWINPUT;
        let mut size: UINT = mem::size_of::<RAWINPUT>() as UINT;

        GetRawInputData(
            h_raw_input,
            RID_INPUT,
            ptr::null_mut(),
            &mut size,
            mem::size_of::<RAWINPUTHEADER>() as UINT,
        );

        let mut buffer: Vec<u8> = Vec::with_capacity(size as usize);
        GetRawInputData(
            h_raw_input,
            RID_INPUT,
            buffer.as_mut_ptr() as *mut c_void,
            &mut size,
            mem::size_of::<RAWINPUTHEADER>() as UINT,
        );
        buffer.set_len(size as usize);

        let raw_input = &*(buffer.as_ptr() as *const RAWINPUT);

        if raw_input.header.dwType == RIM_TYPEKEYBOARD {
            let vkey = raw_input.data.keyboard().VKey;
            let flags = raw_input.data.keyboard().Flags;

            if flags == 1 {
                // if it's a key release, we just throw it out the window :D
                return 0;
            }

            println!("Key pressed: {:?}, flags: {:?}", key_to_string(vkey), flags);
        }
    }

    DefWindowProcA(h_wnd, msg, w_param, l_param)
}

fn main() {
    let raw_input_cstring = CString::new("RawInputClass").unwrap();
    let wcx: WNDCLASSEXA = WNDCLASSEXA {
        cbSize: mem::size_of::<WNDCLASSEXA>() as UINT,
        lpfnWndProc: Some(wind_proc),
        hInstance: unsafe { GetModuleHandleA(ptr::null_mut()) },
        lpszClassName: raw_input_cstring.as_ptr().cast(),
        ..unsafe { std::mem::zeroed() }
    };
    unsafe { RegisterClassExA(&wcx) };
    println!("Successfully registered class {:?}", wcx.lpszClassName);

    let h_wnd = unsafe {
        CreateWindowExA(
            0,
            raw_input_cstring.as_ptr().cast(),
            ptr::null_mut(),
            0,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            ptr::null_mut(),
            GetModuleHandleA(ptr::null_mut()),
            ptr::null_mut(),
        )
    };

    println!("Successfully created window {:?}", h_wnd);

    let rid = RAWINPUTDEVICE {
        usUsagePage: 0x01,
        usUsage: 0x06, // keyboard
        dwFlags: RIDEV_INPUTSINK,
        hwndTarget: h_wnd,
    };

    println!("Successfully registered device");

    unsafe {
        RegisterRawInputDevices(
            &rid as *const RAWINPUTDEVICE,
            1,
            mem::size_of::<RAWINPUTDEVICE>() as u32,
        );
    }

    let mut msg: MSG = unsafe { mem::zeroed() };

    while unsafe { GetMessageA(&mut msg, ptr::null_mut(), 0, 0) } != 0 {
        unsafe {
            TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
    }
}

fn key_to_string(key_pressed: u16) -> String {
    match key_pressed {
        0x41..=0x5A => ((key_pressed as u8 - 0x41 + b'A') as char).to_string(), // A-Z
        0x30..=0x39 => ((key_pressed as u8 - 0x30 + b'0') as char).to_string(), // 0-9
        0x0D => "Enter".to_string(),
        0x10 => "Shift".to_string(),
        0x11 => "Ctrl".to_string(),
        0x12 => "Alt".to_string(),
        0x14 => "Caps lock".to_string(),
        0x08 => "Backspace".to_string(),
        0x20 => "Space".to_string(),
        0x1B => "Escape".to_string(),
        _ => "Unknown".to_string(),
    }
}
