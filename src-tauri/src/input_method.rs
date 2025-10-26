use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, PostMessageW, WM_INPUTLANGCHANGEREQUEST,
};

/// 切换到英文输入法（美式键盘布局）
/// 
/// 这个函数会尝试激活英文键盘布局，避免在粘贴时因中文输入法导致的问题
/// 使用 WM_INPUTLANGCHANGEREQUEST 消息来请求切换输入法
pub fn switch_to_english_ime() -> Result<(), String> {
    unsafe {
        // 美式英语键盘布局的 HKL 值
        // 0x04090409 表示美式英语键盘布局
        let english_layout: isize = 0x04090409;
        
        // 获取当前前台窗口
        let hwnd = GetForegroundWindow();
        
        if hwnd.0 == 0 {
            // 如果没有前台窗口，尝试使用广播消息
            let result = PostMessageW(
                HWND(0xFFFF as isize), // HWND_BROADCAST
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(0),
                LPARAM(english_layout),
            );
            
            if result.is_ok() {
                println!("已发送输入法切换请求（广播）");
                return Ok(());
            }
        } else {
            // 向前台窗口发送输入法切换消息
            let result = PostMessageW(
                hwnd,
                WM_INPUTLANGCHANGEREQUEST,
                WPARAM(0),
                LPARAM(english_layout),
            );
            
            if result.is_ok() {
                println!("已切换到英文输入法 (Layout: 0x{:X})", english_layout);
                return Ok(());
            } else {
                return Err("发送输入法切换消息失败".to_string());
            }
        }
        
        Err("无法切换输入法".to_string())
    }
}

/// 获取当前激活的键盘布局信息（简化版本）
pub fn get_current_layout_info() -> String {
    // 简化的版本，直接返回提示信息
    "Keyboard Layout Monitor".to_string()
}

