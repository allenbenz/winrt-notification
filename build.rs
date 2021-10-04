// see https://microsoft.github.io/windows-docs-rs/doc/bindings/windows/ for possible bindings
fn main() {
    windows::build!(
        Windows::Win32::Foundation::NTSTATUS,
        Windows::Win32::System::SystemInformation::OSVERSIONINFOEXW,
        Windows::Win32::System::SystemInformation::OSVERSIONINFOEXA,
        Windows::Data::Xml::Dom::XmlDocument,
        Windows::UI::Notifications::{ToastNotification, ToastNotificationManager, ToastNotifier},
    );
}
