use log::warn;
use std::ffi::c_void;

#[cfg(windows)]
mod win32 {
    use std::ffi::c_void;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn IsDebuggerPresent() -> i32;
        pub fn CheckRemoteDebuggerPresent(hProcess: *mut c_void, pbDebuggerPresent: *mut i32) -> i32;
        pub fn GetCurrentProcess() -> *mut c_void;
        pub fn VirtualLock(lpAddress: *const c_void, dwSize: usize) -> i32;
        pub fn VirtualUnlock(lpAddress: *const c_void, dwSize: usize) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        pub fn SetWindowDisplayAffinity(hWnd: *mut c_void, dwAffinity: u32) -> i32;
    }

    pub const WDA_NONE: u32 = 0x00000000;
    pub const WDA_MONITOR: u32 = 0x00000001;
    pub const WDA_EXCLUDEFROMCAPTURE: u32 = 0x00000011;
}

/// Detect whether the current process is attached to an active debugger
/// (e.g. x64dbg, IDA Pro, Cheat Engine, WinDbg, gdb)
pub fn is_debugger_attached() -> bool {
    #[cfg(windows)]
    unsafe {
        if win32::IsDebuggerPresent() != 0 {
            warn!("[SECURITY ALERT] Local debugger detected via IsDebuggerPresent!");
            return true;
        }

        let mut is_remote = 0;
        let proc = win32::GetCurrentProcess();
        if win32::CheckRemoteDebuggerPresent(proc, &mut is_remote) != 0 && is_remote != 0 {
            warn!("[SECURITY ALERT] Remote debugger detected via CheckRemoteDebuggerPresent!");
            return true;
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Linux: check /proc/self/status for TracerPid != 0
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("TracerPid:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[1] != "0" {
                        warn!("[SECURITY ALERT] Linux TracerPid debugger attached: {}", parts[1]);
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Lock memory pages into physical RAM to prevent swapping/paging to disk (pagefile.sys / swapfile)
pub fn lock_memory_buffer(buffer: &[u8]) -> bool {
    if buffer.is_empty() {
        return true;
    }

    #[cfg(windows)]
    unsafe {
        let res = win32::VirtualLock(buffer.as_ptr() as *const c_void, buffer.len());
        res != 0
    }

    #[cfg(unix)]
    unsafe {
        extern "C" {
            fn mlock(addr: *const c_void, len: usize) -> i32;
        }
        mlock(buffer.as_ptr() as *const c_void, buffer.len()) == 0
    }

    #[cfg(not(any(windows, unix)))]
    {
        true
    }
}

/// Release memory page lock
pub fn unlock_memory_buffer(buffer: &[u8]) -> bool {
    if buffer.is_empty() {
        return true;
    }

    #[cfg(windows)]
    unsafe {
        let res = win32::VirtualUnlock(buffer.as_ptr() as *const c_void, buffer.len());
        res != 0
    }

    #[cfg(unix)]
    unsafe {
        extern "C" {
            fn munlock(addr: *const c_void, len: usize) -> i32;
        }
        munlock(buffer.as_ptr() as *const c_void, buffer.len()) == 0
    }

    #[cfg(not(any(windows, unix)))]
    {
        true
    }
}

/// Set anti-screen capture / display affinity on Windows
/// When active, OBS, Discord, and Screen Capture software see only a pitch-black box
pub fn set_display_affinity_protection(hwnd: *mut c_void, enabled: bool) -> bool {
    #[cfg(windows)]
    unsafe {
        if hwnd.is_null() {
            return false;
        }
        let affinity = if enabled {
            win32::WDA_EXCLUDEFROMCAPTURE // Win10 2004+ and Win11; falls back to WDA_MONITOR if unsupported
        } else {
            win32::WDA_NONE
        };

        let res = win32::SetWindowDisplayAffinity(hwnd, affinity);
        if res == 0 && enabled {
            // Fallback to WDA_MONITOR
            win32::SetWindowDisplayAffinity(hwnd, win32::WDA_MONITOR) != 0
        } else {
            res != 0
        }
    }

    #[cfg(not(windows))]
    {
        let _ = hwnd;
        let _ = enabled;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_detection_call() {
        // Under normal cargo test execution without interactive debugger attached, should return false
        let attached = is_debugger_attached();
        assert!(!attached, "No debugger should be attached during standard cargo test");
    }

    #[test]
    fn test_memory_page_locking() {
        let sensitive_data = vec![0x42u8; 4096]; // One page of sensitive memory
        let locked = lock_memory_buffer(&sensitive_data);
        assert!(locked, "VirtualLock/mlock should successfully lock memory buffer");

        let unlocked = unlock_memory_buffer(&sensitive_data);
        assert!(unlocked, "VirtualUnlock/munlock should successfully unlock memory buffer");
    }
}
