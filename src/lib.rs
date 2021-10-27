//! An incomplete wrapper over the WinRT toast api
//!
//! Tested in Windows 10 and 8.1. Untested in Windows 8, might work.
//!
//! Todo:
//!
//! * Add support for Adaptive Content
//! * Add support for Actions
//!
//! Known Issues:
//!
//! * Will not work for Windows 7.
//! * Will not build when targeting the 32-bit gnu toolchain (i686-pc-windows-gnu).
//!
//! Limitations:
//!
//! * Windows 8.1 only supports a single image, the last image (icon, hero, image) will be the one on the toast

/// for xml schema details check out:
///
/// * https://docs.microsoft.com/en-us/uwp/schemas/tiles/toastschema/root-elements
/// * https://docs.microsoft.com/en-us/windows/uwp/controls-and-patterns/tiles-and-notifications-toast-xml-schema
/// * https://docs.microsoft.com/en-us/windows/uwp/controls-and-patterns/tiles-and-notifications-adaptive-interactive-toasts
/// * https://msdn.microsoft.com/library/14a07fce-d631-4bad-ab99-305b703713e6#Sending_toast_notifications_from_desktop_apps

/// for Windows 7 and older support look into Shell_NotifyIcon
/// https://msdn.microsoft.com/en-us/library/windows/desktop/ee330740(v=vs.85).aspx
/// https://softwareengineering.stackexchange.com/questions/222339/using-the-system-tray-notification-area-app-in-windows-7
extern crate windows;
extern crate xml;

#[macro_use]
extern crate strum;

#[allow(dead_code)]
mod bindings {
    ::windows::include_bindings!();
}

use bindings::{
    Windows::Data::Xml::Dom::XmlDocument,
    Windows::Foundation::TypedEventHandler,
    Windows::Foundation::IReference,
    Windows::UI::Notifications::ToastNotificationManager,
};
use windows::Interface;

use std::fmt;
use std::path::Path;

use xml::escape::escape_str_attribute;
mod toast_action;
mod toast_input;
mod windows_check;

pub use windows::{Error, HSTRING};

pub use toast_action::{ToastAction, ToastActivationType};
pub use toast_input::ToastInput;

pub use bindings::Windows::UI::Notifications::{ToastActivatedEventArgs, ToastDismissedEventArgs, ToastFailedEventArgs, ToastNotification};

pub struct Toast {
    duration: String,
    title: String,
    line1: String,
    line2: String,
    images: String,
    audio: String,
    app_id: String,
    actions: String,
}

#[derive(Clone, Copy)]
pub enum Duration {
    /// 7 seconds
    Short,

    /// 25 seconds
    Long,
}

#[derive(Display, Debug, EnumString, Clone, Copy)]
pub enum Sound {
    Default,
    IM,
    Mail,
    Reminder,
    SMS,
    /// Play the loopable sound only once
    #[strum(disabled)]
    Single(LoopableSound),
    /// Loop the loopable sound for the entire duration of the toast
    #[strum(disabled)]
    Loop(LoopableSound),
}

/// Sounds suitable for Looping
#[allow(dead_code)]
#[derive(Display, Debug, Clone, Copy)]
pub enum LoopableSound {
    Alarm,
    Alarm2,
    Alarm3,
    Alarm4,
    Alarm5,
    Alarm6,
    Alarm7,
    Alarm8,
    Alarm9,
    Alarm10,
    Call,
    Call2,
    Call3,
    Call4,
    Call5,
    Call6,
    Call7,
    Call8,
    Call9,
    Call10,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum IconCrop {
    Square,
    Circular,
}

impl Toast {
    /// This can be used if you do not have a AppUserModelID.
    ///
    /// However, the toast will erroniously report its origin as powershell.
    pub const POWERSHELL_APP_ID: &'static str = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\
                                                 \\WindowsPowerShell\\v1.0\\powershell.exe";
    /// Constructor for the toast builder.
    ///
    /// app_id is the running application's [AppUserModelID][1].
    ///
    /// [1]: https://msdn.microsoft.com/en-us/library/windows/desktop/dd378459(v=vs.85).aspx
    ///
    /// If the program you are using this in was not installed, use Toast::POWERSHELL_APP_ID for now
    #[allow(dead_code)]
    pub fn new(app_id: &str) -> Toast {
        Toast {
            duration: String::new(),
            title: String::new(),
            line1: String::new(),
            line2: String::new(),
            images: String::new(),
            audio: String::new(),
            actions: String::new(),
            app_id: app_id.to_string(),
        }
    }

    /// Sets the title of the toast.
    ///
    /// Will be white.
    /// Supports Unicode ✓
    pub fn title(mut self, content: &str) -> Toast {
        self.title = format!(r#"<text id="1">{}</text>"#, escape_str_attribute(content));
        self
    }

    /// Add/Sets the first line of text below title.
    ///
    /// Will be grey.
    /// Supports Unicode ✓
    pub fn text1(mut self, content: &str) -> Toast {
        self.line1 = format!(r#"<text id="2">{}</text>"#, escape_str_attribute(content));
        self
    }

    /// Add/Sets the second line of text below title.
    ///
    /// Will be grey.
    /// Supports Unicode ✓
    pub fn text2(mut self, content: &str) -> Toast {
        self.line2 = format!(r#"<text id="3">{}</text>"#, escape_str_attribute(content));
        self
    }

    /// Set the length of time to show the toast
    pub fn duration(mut self, duration: Duration) -> Toast {
        self.duration = match duration {
            Duration::Long => "duration=\"long\"",
            Duration::Short => "duration=\"short\"",
        }
        .to_owned();
        self
    }

    /// Set the icon shown in the upper left of the toast
    ///
    /// The default is determined by your app id.
    /// If you are using the powershell workaround, it will be the powershell icon
    pub fn icon(mut self, source: &Path, crop: IconCrop, alt_text: &str) -> Toast {
        if windows_check::is_newer_than_windows81() {
            let crop_type_attr = match crop {
                IconCrop::Square => "".to_string(),
                IconCrop::Circular => "hint-crop=\"circle\"".to_string(),
            };

            self.images = format!(
                r#"{}<image placement="appLogoOverride" {} src="file:///{}" alt="{}" />"#,
                self.images,
                crop_type_attr,
                escape_str_attribute(&source.display().to_string()),
                escape_str_attribute(alt_text)
            );
            self
        } else {
            // Win81 rejects the above xml so we fallback to a simpler call
            self.image(source, alt_text)
        }
    }

    /// Add/Set a Hero image for the toast.
    ///
    /// This will be above the toast text and the icon.
    pub fn hero(mut self, source: &Path, alt_text: &str) -> Toast {
        if windows_check::is_newer_than_windows81() {
            self.images = format!(
                r#"{}<image placement="Hero" src="file:///{}" alt="{}" />"#,
                self.images,
                escape_str_attribute(&source.display().to_string()),
                escape_str_attribute(alt_text)
            );
            self
        } else {
            // win81 rejects the above xml so we fallback to a simpler call
            self.image(source, alt_text)
        }
    }

    /// Add an image to the toast
    ///
    /// May be done many times.
    /// Will appear below text.
    pub fn image(mut self, source: &Path, alt_text: &str) -> Toast {
        if !windows_check::is_newer_than_windows81() {
            // win81 cannot have more than 1 image and shows nothing if there is more than that
            self.images = "".to_owned();
        }
        self.images = format!(
            r#"{}<image id="1" src="file:///{}" alt="{}" />"#,
            self.images,
            escape_str_attribute(&source.display().to_string()),
            escape_str_attribute(alt_text)
        );
        self
    }

    /// Set the sound for the toast or silence it
    ///
    /// Default is [Sound::IM](enum.Sound.html)
    pub fn sound(mut self, src: Option<Sound>) -> Toast {
        self.audio = match src {
            None => "<audio silent=\"true\" />".to_owned(),
            Some(Sound::Default) => "".to_owned(),
            Some(Sound::Loop(sound)) => format!(r#"<audio loop="true" src="ms-winsoundevent:Notification.Looping.{}" />"#, sound),
            Some(Sound::Single(sound)) => format!(r#"<audio src="ms-winsoundevent:Notification.Looping.{}" />"#, sound),
            Some(sound) => format!(r#"<audio src="ms-winsoundevent:Notification.{}" />"#, sound),
        };

        self
    }

    pub fn input(mut self, input: &ToastInput) -> Toast {
        self.actions = format!("{}{}", self.actions, input.to_string());
        self
    }

    pub fn action(mut self, action: &ToastAction) -> Toast {
        self.actions = format!("{}{}", self.actions, action.to_string());
        self
    }

    fn create_template(&self) -> windows::Result<ToastNotification> {
        //using this to get an instance of XmlDocument
        let toast_xml = XmlDocument::new()?;

        let template_binding = if windows_check::is_newer_than_windows81() {
            "ToastGeneric"
        } else
        //win8 or win81
        {
            // Need to do this or an empty placeholder will be shown if no image is set
            if self.images == "" {
                "ToastText04"
            } else {
                "ToastImageAndText04"
            }
        };

        toast_xml.LoadXml(HSTRING::from(format!(
            "<toast {}>
                    <visual>
                        <binding template=\"{}\">
                        {}
                        {}{}{}
                        </binding>
                    </visual>
                    {}
                    <actions>{}</actions>
                </toast>",
            self.duration, template_binding, self.images, self.title, self.line1, self.line2, self.audio, self.actions,
        )))?;

        // Create the toast
        ToastNotification::CreateToastNotification(toast_xml)
    }

    pub fn show_with_action<F: 'static>(&self, on_action: F) -> windows::Result<()>
    where
        F: Fn(&ToastNotification, &ToastActivatedEventArgs) -> windows::Result<()>,
    {
        self.show_with_optional_action_dismiss_failure(
            Some(on_action),
            None::<fn(&ToastNotification, &ToastDismissedEventArgs) -> windows::Result<()>>,
            None::<fn(&ToastNotification, &ToastFailedEventArgs) -> windows::Result<()>>,
        )
    }

    pub fn show_with_action_dismiss<F: 'static, F2: 'static>(&self, on_action: F, on_dismiss: F2) -> windows::Result<()>
    where
        F: Fn(&ToastNotification, &ToastActivatedEventArgs) -> windows::Result<()>,
        F2: Fn(&ToastNotification, &ToastDismissedEventArgs) -> windows::Result<()>,
    {
        self.show_with_optional_action_dismiss_failure(
            Some(on_action),
            Some(on_dismiss),
            None::<fn(&ToastNotification, &ToastFailedEventArgs) -> windows::Result<()>>,
        )
    }

    pub fn show_with_action_dismiss_failure<F: 'static, F2: 'static, F3: 'static>(
        &self,
        on_action: F,
        on_dismiss: F2,
        on_failure: F3,
    ) -> windows::Result<()>
    where
        F: Fn(&ToastNotification, &ToastActivatedEventArgs) -> windows::Result<()>,
        F2: Fn(&ToastNotification, &ToastDismissedEventArgs) -> windows::Result<()>,
        F3: Fn(&ToastNotification, &ToastFailedEventArgs) -> windows::Result<()>,
    {
        self.show_with_optional_action_dismiss_failure(Some(on_action), Some(on_dismiss), Some(on_failure))
    }

    /// Display the toast on the screen with the following handlers connected
    pub fn show_with_optional_action_dismiss_failure<F: 'static, F2: 'static, F3: 'static>(
        &self,
        on_action: Option<F>,
        on_dismiss: Option<F2>,
        on_failure: Option<F3>,
    ) -> windows::Result<()>
    where
        F: Fn(&ToastNotification, &ToastActivatedEventArgs) -> windows::Result<()>,
        F2: Fn(&ToastNotification, &ToastDismissedEventArgs) -> windows::Result<()>,
        F3: Fn(&ToastNotification, &ToastFailedEventArgs) -> windows::Result<()>,
    {
        let toast_template = self.create_template()?;

        // wire up actions if they've been set
        if let Some(action) = on_action {
            toast_template
                .Activated(TypedEventHandler::<ToastNotification, ::windows::IInspectable>::new(
                    move |sender, result| {
                        if let Some(obj) = &*result {
                            if let Some(sender) = &*sender {
                                if let Ok(args) = obj.cast::<ToastActivatedEventArgs>() {
                                    return action(&sender, &args);
                                }
                            }
                        }
                        Ok(())
                    },
                ))
                .expect("can this fail?");
        }

        if let Some(dismissed) = on_dismiss {
            toast_template
                .Dismissed(TypedEventHandler::<ToastNotification, ToastDismissedEventArgs>::new(
                    move |sender, result| {
                        if let Some(args) = &*result {
                            if let Some(sender) = &*sender {
                                return dismissed(&sender, &args);
                            }
                        }
                        Ok(())
                    },
                ))
                .expect("can this fail?");
        }

        if let Some(failed) = on_failure {
            toast_template
                .Failed(TypedEventHandler::<ToastNotification, ToastFailedEventArgs>::new(
                    move |sender, result| {
                        if let Some(args) = &*result {
                            if let Some(sender) = &*sender {
                                return failed(&sender, &args);
                            }
                        }
                        Ok(())
                    },
                ))
                .expect("can this fail?");
        }

        let toast_notifier = ToastNotificationManager::CreateToastNotifierWithId(HSTRING::from(&self.app_id))?;

        // Show the toast.
        let result = toast_notifier.Show(&toast_template);
        std::thread::sleep(std::time::Duration::from_millis(10));
        result
    }

    /// Display the toast on the screen
    pub fn show(&self) -> windows::Result<()> {
        self.show_with_optional_action_dismiss_failure(
            None::<fn(&ToastNotification, &ToastActivatedEventArgs) -> windows::Result<()>>,
            None::<fn(&ToastNotification, &ToastDismissedEventArgs) -> windows::Result<()>>,
            None::<fn(&ToastNotification, &ToastFailedEventArgs) -> windows::Result<()>>,
        )
    }
}

pub trait UserInputExt {
    fn lookup<T: ::windows::RuntimeType>(&self, key: &str) -> windows::Result<IReference<T>>;
    fn lookup_string(&self, key: &str) -> windows::Result<HSTRING>;
}

impl UserInputExt for ToastActivatedEventArgs {
    fn lookup<T: windows::RuntimeType>(&self, key: &str) -> windows::Result<IReference<T>> {
        self.UserInput()?.Lookup(key)?.cast::<IReference<T>>()
    }

    fn lookup_string(&self, key: &str) -> windows::Result<HSTRING> {
        self.lookup::<HSTRING>(key)?.GetString()
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::{path::Path, thread::sleep};

    #[test]
    fn simple_toast() {
        let toast = Toast::new(Toast::POWERSHELL_APP_ID);
        toast
            .hero(&Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/flower.jpeg"), "flower")
            .icon(
                &Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/test/chick.jpeg"),
                IconCrop::Circular,
                "chicken",
            )
            .title("title")
            .text1("line1")
            .text2("line2")
            .input(&ToastInput::new("myField")
                .title("SomeTitle")
                .place_holder_content("type things here")
            )
            .action(
                &ToastAction::new()
                    .text("The bird")
                    .arguments("bird")
                    .activation_type(ToastActivationType::System),
            )
            .action(&ToastAction::new().text("The flower").arguments("flower"))
            .duration(Duration::Short)
            //.sound(Some(Sound::Loop(LoopableSound::Call)))
            //.sound(Some(Sound::SMS))
            .sound(None)
            .show_with_action(|_, args| {
                println!("{}", args.Arguments()?);
                if let Ok(my_field) = args.lookup_string("myField") {
                    println!("{}", my_field)
                }
                Ok(())
            })
            // silently consume errors
            .expect("notification failed");

        sleep(std::time::Duration::from_secs(10));
    }
}
