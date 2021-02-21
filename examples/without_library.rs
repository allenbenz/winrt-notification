// How to create a toast without using this library

// Note while this shows an action in the xml and buttons will appear on the toast
// these won't do anything other than dismiss the toast.
// Wiring it up to do that is still being explored.

extern crate xml;
use std::path::Path;
use xml::escape::escape_str_attribute;

#[allow(dead_code)]
mod bindings {
    ::windows::include_bindings!();
}

// You need to have the windows crate in your Cargo.toml
//
// and call windows::build! in a build.rs file
// or have pregenerated code that does the same thing
use bindings::{
    windows::data::xml::dom::XmlDocument,
    windows::ui::notifications::ToastNotification,
    windows::ui::notifications::ToastNotificationManager,
    windows::HString,
};

pub use windows::Error;

fn main() {
    do_toast().expect("not sure if this is actually failable");
    // this is a hack to workaround toasts not showing up if the application closes too quickly
    // you can put this in do_toast if you want.
    std::thread::sleep(std::time::Duration::from_millis(10));
}

fn do_toast() -> windows::Result<()> {
    let toast_xml = XmlDocument::new()?;

    toast_xml.load_xml(HString::from(
        format!(r#"<toast duration="long">
                <visual>
                    <binding template="ToastGeneric">
                        <text id="1">title</text>
                        <text id="2">first line</text>
                        <text id="3">third line</text>
                        <image placement="appLogoOverride" hint-crop="circle" src="file:///c:/path_to_image_above_toast.jpg" alt="alt text" />
                        <image placement="Hero" src="file:///C:/path_to_image_in_toast.jpg" alt="alt text2" />
                        <image id="1" src="file:///{}" alt="another_image" />
                    </binding>
                </visual>
                <audio src="ms-winsoundevent:Notification.SMS" />
                <!-- <audio silent="true" /> -->
                <!-- See https://docs.microsoft.com/en-us/windows/uwp/design/shell/tiles-and-notifications/toast-pending-update?tabs=xml for possible actions -->
                <actions>
                    <action content="check" arguments="check" />
                    <action content="cancel" arguments="cancel" />
                </actions>
            </toast>"#,
        escape_str_attribute(&Path::new("C:\\path_to_image_in_toast.jpg").display().to_string()),
    ))).expect("the xml is malformed");

    // Create the toast and attach event listeners
    let toast_template = ToastNotification::create_toast_notification(toast_xml)?;

    // If you have a valid app id, (ie installed using wix) then use it here.
    let toast_notifier = ToastNotificationManager::create_toast_notifier_with_id(HString::from(
        "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
    ))?;

    // Show the toast.
    // Note this returns success in every case, including when the toast isn't shown.
    toast_notifier.show(&toast_template)
}
