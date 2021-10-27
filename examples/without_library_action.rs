// How to create a toast without using this library

extern crate xml;
use std::path::Path;
use ::windows::runtime::{self, IInspectable};
use xml::escape::escape_str_attribute;

// You need to have the windows crate in your Cargo.toml
// with the following features:
//    "Data_Xml_Dom"
//    "Win32_Foundation"
//    "Foundation_Collections"
//    "UI_Notifications"

use windows::{
    Data::Xml::Dom::XmlDocument,
    Foundation::TypedEventHandler,
    Foundation::IReference,
    UI::Notifications::ToastNotification,
    UI::Notifications::ToastNotificationManager,
    UI::Notifications::{ToastActivatedEventArgs, ToastDismissalReason, ToastDismissedEventArgs, ToastFailedEventArgs},
};

//https://social.msdn.microsoft.com/Forums/Windows/en-US/99e0d4bd-07cb-4ebd-8c92-c44ac6e7e5de/toast-notification-dismissed-event-handler-not-called-every-time?forum=windowsgeneraldevelopmentissues
pub use windows::runtime::{Error, HSTRING, Interface};

fn main() {
    do_toast().expect("not sure if this is actually failable");
    // this is a hack to workaround toasts not showing up if the application closes too quickly
    // you can put this in do_toast if you want.
    std::thread::sleep(std::time::Duration::from_millis(10000));
}

fn do_toast() -> windows::runtime::Result<()> {
    let toast_xml = XmlDocument::new()?;

    toast_xml.LoadXml(HSTRING::from(
        format!(r#"<toast duration="short">
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
                    <input id="myTextField" type="text" placeHolderContent="put stuff here" />
                    <action content="left" arguments="left button was clicked" />
                    <action content="Submit" hint-inputId="myTextField" arguments="submit button was clicked" />
                </actions>
            </toast>"#,
        escape_str_attribute(&Path::new("C:\\path_to_image_in_toast.jpg").display().to_string()),
    ))).expect("the xml is malformed");

    // Create the toast and attach event listeners
    let toast_notification = ToastNotification::CreateToastNotification(toast_xml)?;

    // happens if any of the toasts actions are interacted with (as a popup or in the action center)
    toast_notification.Activated(TypedEventHandler::<ToastNotification, IInspectable>::new(|_sender, result| {
        // Activated has the wrong type signature so you have to cast the object
        // Dismissed and Failed have the correct signature so they work without doing this
        if let Some(obj) = &*result {
            let args = obj.cast::<ToastActivatedEventArgs>()?;
            println!("The interacted element has an arguments value of: {}", args.Arguments()?);
            // You can use something similar to find the type of more complicated forum elements
            println!("{}", &args.UserInput()?.Lookup("myTextField")?.type_name()?);
            if let Ok(my_text_field) = user_input_lookup::<HSTRING>(&args, "myTextField") {
                println!("The text box contains: {}", my_text_field.GetString()?);
            }
        }

        Ok(())
    }))?;

    // happens if the toast is moved to the action center or dismissed in the action center
    // or if it ends without the user clicking on anything.
    // Note that Dismissed and then Activated can be triggered from the same toast.
    toast_notification.Dismissed(TypedEventHandler::<ToastNotification, ToastDismissedEventArgs>::new(
        |_sender, result| {

            if let Some(args) = &*result {
                let reason = match args.Reason() {
                    Ok(ToastDismissalReason::UserCanceled) => "User Canceled",
                    Ok(ToastDismissalReason::ApplicationHidden) => "Application Hidden",
                    Ok(ToastDismissalReason::TimedOut) => "Timed out",
                    Ok(_) => "Unknown reason",
                    Err(_) => "Error",
                };
                println!("{}", reason);
            };
            Ok(())
        },
    ))?;

    // happens if toasts are disabled
    toast_notification.Failed(TypedEventHandler::<ToastNotification, ToastFailedEventArgs>::new(
        |_sender, result| {
            println!("failed");
            if let Some(args) = &*result {
                println!("{}", args.ErrorCode()?.message())
            }
            Ok(())
        },
    ))?;

    // If you have a valid app id, (ie installed using wix) then use it here.
    let toast_notifier = ToastNotificationManager::CreateToastNotifierWithId(HSTRING::from(
        "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
    ))?;

    // Show the toast.
    // Note this returns success in every case, including when the toast isn't shown.
    toast_notifier.Show(&toast_notification)
}

pub fn user_input_lookup<T: runtime::RuntimeType>(args: &ToastActivatedEventArgs, key: &str) -> windows::runtime::Result<IReference<T>> {
    args.UserInput()?.Lookup(key)?.cast::<IReference<T>>()
}
