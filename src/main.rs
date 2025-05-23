#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use windows_sys::{
    core::PCWSTR,
    w,
    Win32::{
        Foundation::{GetLastError, INVALID_HANDLE_VALUE, RECT},
        Graphics::{
            Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
            Gdi::{GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST},
        },
        System::Threading::Sleep,
        UI::{
            Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON},
            WindowsAndMessaging::{
                GetForegroundWindow, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
                MessageBoxW, SetWindowPos, IDCANCEL, MB_OKCANCEL, MB_TOPMOST, MESSAGEBOX_RESULT,
                MESSAGEBOX_STYLE, SWP_NOSIZE,
            },
        },
    },
};

#[derive(Clone, Copy)]
enum LastOperationStatus {
    Success,
    Cancelled,
    GetForegroundWindowFailed,
    GetWindowTextLengthFailed,
    GetWindowTextFailed,
    GetMonitorInfoFailed,
    DwmGetWindowAttributeFailed,
    GetWindowRectFailed,
    SetWindowPosFailed,
}

impl LastOperationStatus {
    #[inline(always)]
    const fn as_str(self) -> &'static str {
        match self {
            Self::Success => "Successfully centered.\r\n\r\n",
            Self::Cancelled => "Operation was cancelled by user.\r\n\r\n",
            Self::GetForegroundWindowFailed => "GetForegroundWindow has failed.\r\n\r\n",
            Self::GetWindowTextLengthFailed => "GetWindowTextLength has failed.\r\n\r\n",
            Self::GetWindowTextFailed => "GetWindowText has failed.\r\n\r\n",
            Self::GetMonitorInfoFailed => "GetMonitorInfo has failed.\r\n\r\n",
            Self::DwmGetWindowAttributeFailed => "DwmGetWindowAttribute has failed.\r\n\r\n",
            Self::GetWindowRectFailed => "GetWindowRect has failed.\r\n\r\n",
            Self::SetWindowPosFailed => "SetWindowPos has failed.\r\n\r\n",
        }
    }
}

#[inline(always)]
#[allow(non_snake_case)]
unsafe fn ShowMessageBox(text: PCWSTR, style: MESSAGEBOX_STYLE) -> MESSAGEBOX_RESULT {
    MessageBoxW(std::ptr::null_mut(), text, w!("center-window"), style)
}

const RECT_ZERO: RECT = RECT {
    left: 0,
    top: 0,
    right: 0,
    bottom: 0,
};

fn main() {
    let mut last_status: Option<LastOperationStatus> = None;
    loop {
        unsafe {
            let result = ShowMessageBox(
                format!(
                    "{}Click the window you want to center after clicking OK.\0",
                    last_status.map_or("", LastOperationStatus::as_str)
                )
                .encode_utf16()
                .collect::<Vec<u16>>()
                .as_ptr(),
                MB_OKCANCEL | MB_TOPMOST,
            );
            if result == IDCANCEL {
                break;
            }
            while GetAsyncKeyState(VK_LBUTTON.into()) == 0 {
                Sleep(50);
            }
            let hwnd = GetForegroundWindow();
            if hwnd == INVALID_HANDLE_VALUE {
                last_status = Some(LastOperationStatus::GetForegroundWindowFailed);
                continue;
            }
            let title_length = GetWindowTextLengthW(hwnd);
            let title = {
                if title_length == 0 {
                    if GetLastError() != 0 {
                        last_status = Some(LastOperationStatus::GetWindowTextLengthFailed);
                        continue;
                    }
                    "(empty)".encode_utf16().collect::<Vec<u16>>()
                } else {
                    let mut title = vec![0; title_length as usize + 1];
                    let result = GetWindowTextW(hwnd, title.as_mut_ptr(), title_length + 1);
                    if result == 0 {
                        last_status = Some(LastOperationStatus::GetWindowTextFailed);
                        continue;
                    }
                    title[..title.len() - 1].to_vec()
                }
            };
            let result = ShowMessageBox(
                [
                    "Window title: ".encode_utf16().collect(),
                    title,
                    "\r\n\r\nClick OK to center the window.\0"
                        .encode_utf16()
                        .collect(),
                ]
                .concat()
                .as_ptr(),
                MB_OKCANCEL,
            );
            if result == IDCANCEL {
                last_status = Some(LastOperationStatus::Cancelled);
                continue;
            }
            let monitor_rect = {
                let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                let mut monitor_info: MONITORINFO = MONITORINFO {
                    cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                    rcMonitor: RECT_ZERO,
                    rcWork: RECT_ZERO,
                    dwFlags: 0,
                };
                if GetMonitorInfoW(monitor, &mut monitor_info) == 0 {
                    last_status = Some(LastOperationStatus::GetMonitorInfoFailed);
                    continue;
                }
                monitor_info.rcWork
            };
            let mut window_rect_exact: RECT = RECT_ZERO;
            if DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                &mut window_rect_exact as *mut _ as *mut _,
                std::mem::size_of::<RECT>() as u32,
            ) != 0
            {
                last_status = Some(LastOperationStatus::DwmGetWindowAttributeFailed);
                continue;
            }
            let mut window_rect: RECT = RECT_ZERO;
            if GetWindowRect(hwnd, &mut window_rect) == 0 {
                last_status = Some(LastOperationStatus::GetWindowRectFailed);
                continue;
            }
            let window_width = window_rect_exact.right - window_rect_exact.left;
            let window_height = window_rect_exact.bottom - window_rect_exact.top;
            let shadow_left = window_rect.left - window_rect_exact.left;
            let shadow_top = window_rect.top - window_rect_exact.top;
            let x = monitor_rect.left
                + (monitor_rect.right - monitor_rect.left - window_width) / 2
                + shadow_left;
            let y = monitor_rect.top
                + (monitor_rect.bottom - monitor_rect.top - window_height) / 2
                + shadow_top;
            if SetWindowPos(hwnd, std::ptr::null_mut(), x, y, 0, 0, SWP_NOSIZE) == 0 {
                last_status = Some(LastOperationStatus::SetWindowPosFailed);
                continue;
            }
            last_status = Some(LastOperationStatus::Success);
        }
    }
}
