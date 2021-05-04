// see https://microsoft.github.io/windows-docs-rs/doc/bindings/windows/ for possible bindings
fn main() {
    windows::build!(
        Windows::Win32::SystemServices::NTSTATUS,
        Windows::Win32::WindowsProgramming::OSVERSIONINFOEXA,
        Windows::Win32::WindowsProgramming::OSVERSIONINFOEXW,
        Windows::Data::Xml::Dom::XmlDocument,
        Windows::UI::Notifications::{ToastNotification, ToastNotificationManager, ToastNotifier},
    );
}
