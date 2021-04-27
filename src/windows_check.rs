// Lifted from mattmccarty's work in os_info
#[allow(dead_code)]
mod bindings {
    ::windows::include_bindings!();
}

use bindings::{
    Windows::Win32::SystemServices::NTSTATUS,
    Windows::Win32::WindowsProgramming::*,
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
            info.dwMajorVersion > 6
        } else {
            false
        }
    }
}
