// see https://microsoft.github.io/windows-docs-rs/doc/bindings/windows/ for possible bindings
fn main() {
    windows::build!(
        windows::win32::system_services::NTSTATUS,
        windows::win32::windows_programming::OSVERSIONINFOEXA,
        windows::win32::windows_programming::OSVERSIONINFOEXW,
        windows::data::xml::dom::XmlDocument,
        windows::ui::notifications::{ToastNotification, ToastNotificationManager},
    );
}
