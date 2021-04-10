// Lifted from mattmccarty's work in os_info
#[allow(dead_code)]
mod bindings {
    ::windows::include_bindings!();
}

use bindings::{
    windows::win32::system_services::NTSTATUS,
    windows::win32::windows_programming::*,
};

#[cfg(target_arch = "x86")]
use OSVERSIONINFOEXA;
#[cfg(target_arch = "x86")]
type OSVERSIONINFOEX = OSVERSIONINFOEXA;

#[cfg(not(target_arch = "x86"))]
use OSVERSIONINFOEXW;
#[cfg(not(target_arch = "x86"))]
type OSVERSIONINFOEX = OSVERSIONINFOEXW;

#[link(name = "ntdll")]
extern "system" {
    pub fn RtlGetVersion(lpVersionInformation: &mut OSVERSIONINFOEX) -> NTSTATUS;
}

pub fn is_newer_than_windows81() -> bool {
    unsafe {
        let mut info: OSVERSIONINFOEX = OSVERSIONINFOEX::default();

        if RtlGetVersion(&mut info) == NTSTATUS(0) {
            info.dw_major_version > 6
        } else {
            false
        }
    }
}
