use std::ffi::c_void;
use tokio::time::{sleep, Duration};
use windows::Win32::{
    Foundation::HGLOBAL,
    System::{
        DataExchange::CloseClipboard,
        Memory::{GlobalLock, GlobalUnlock},
    },
    UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
        KEYEVENTF_UNICODE, VIRTUAL_KEY, VK_RETURN,
    },
};
use windows::Win32::{
    Foundation::HWND,
    System::DataExchange::{GetClipboardData, OpenClipboard},
};

fn get_clipboard() -> Result<Vec<u16>, &'static str> {
    const CF_UNICODETEXT: u32 = 13;
    let mut result: Vec<u16> = vec![];

    //参考 https://learn.microsoft.com/zh-cn/windows/win32/dataxchg/using-the-clipboard#pasting-information-from-the-clipboard
    unsafe {
        OpenClipboard(HWND(0)).or(Err("打开剪切板错误"))?;
        let hglb = GetClipboardData(CF_UNICODETEXT).map_err(|_| {
            if let Err(_) = CloseClipboard() {
                return "关闭剪切板失败";
            }
            "获取剪切板数据错误"
        })?;
        let locker = HGLOBAL(hglb.0 as *mut c_void);
        let raw_data = GlobalLock(locker);
        let data = raw_data as *const u16;
        let mut i = 0usize;

        loop {
            let item = *data.add(i);
            i += 1;
            if item == 0 {
                break;
            }
            if item == 13 {
                //舍弃'\r'
                continue;
            }
            result.push(item);
        }

        GlobalUnlock(locker).map_err(|_| {
            if let Err(_) = CloseClipboard() {
                return "关闭剪切板失败";
            }
            "解除剪切板锁定失败"
        })?;
        CloseClipboard().or(Err("关闭剪切板失败"))?;
    }
    return Ok(result);
}

/// 执行粘贴操作的核心逻辑
async fn do_paste(utf16_units: Vec<u16>, stand: u32, float: u32) -> Result<(), &'static str> {
    for item in utf16_units {
        if item == 10 {
            //必须特别处理回车
            let input = [
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        //参考 https://learn.microsoft.com/zh-cn/windows/win32/api/winuser/ns-winuser-keybdinput
                        ki: KEYBDINPUT {
                            wVk: VK_RETURN,
                            wScan: 0,
                            dwFlags: KEYBD_EVENT_FLAGS(0),
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
                INPUT {
                    r#type: INPUT_KEYBOARD,
                    Anonymous: INPUT_0 {
                        ki: KEYBDINPUT {
                            wVk: VK_RETURN,
                            wScan: 0,
                            dwFlags: KEYEVENTF_KEYUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                },
            ];
            unsafe {
                SendInput(&input, std::mem::size_of::<INPUT>() as i32);
            }
        } else {
            let input = [INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: item,
                        dwFlags: KEYEVENTF_UNICODE,
                        time: 0,
                        dwExtraInfo: 0,
                    },
                },
            }];
            unsafe {
                SendInput(&input, std::mem::size_of::<INPUT>() as i32);
            }
        };

        let random = rand::random::<u32>();
        sleep(Duration::from_millis((stand + random % float) as u64)).await;
    }

    Ok(())
}

/// 通过按钮触发的粘贴（有倒计时）
#[tauri::command]
pub async fn paste(stand: u32, float: u32) -> Result<(), &'static str> {
    let utf16_units: Vec<u16> = get_clipboard()?;
    do_paste(utf16_units, stand, float).await
}

/// 通过快捷键触发的粘贴（无倒计时，立即执行）
#[tauri::command]
pub async fn paste_instant(stand: u32, float: u32) -> Result<(), &'static str> {
    let utf16_units: Vec<u16> = get_clipboard()?;
    do_paste(utf16_units, stand, float).await
}
