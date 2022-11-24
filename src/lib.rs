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

/// For actions look at https://docs.microsoft.com/en-us/dotnet/api/microsoft.toolkit.uwp.notifications.toastactionscustom?view=win-comm-toolkit-dotnet-7.0
extern crate windows;
extern crate xml;

#[macro_use]
extern crate strum;

use windows::{
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::ToastNotificationManager,
};

use std::path::Path;

use xml::escape::escape_str_attribute;
mod windows_check;

pub use windows::core::{
    Error,
    Result,
    HSTRING,
};
pub use windows::UI::Notifications::ToastNotification;

pub struct Toast {
    duration: String,
    title: String,
    line1: String,
    line2: String,
    images: String,
    audio: String,
    app_id: String,
    scenario: String,
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

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum Scenario {
    /// The normal toast behavior.
    Default,
    /// This will be displayed pre-expanded and stay on the user's screen till dismissed. Audio will loop by default and will use alarm audio.
    Alarm,
    /// This will be displayed pre-expanded and stay on the user's screen till dismissed..
    Reminder,
    /// This will be displayed pre-expanded in a special call format and stay on the user's screen till dismissed. Audio will loop by default and will use ringtone audio.
    IncomingCall,
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
            app_id: app_id.to_string(),
            scenario: String::new(),
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

    /// Set the scenario of the toast
    ///
    /// The system keeps the notification on screen until the user acts upon/dismisses it.
    /// The system also plays the suitable notification sound as well.
    pub fn scenario(mut self, scenario: Scenario) -> Toast {
        self.scenario = match scenario {
            Scenario::Default => "",
            Scenario::Alarm => "scenario=\"alarm\"",
            Scenario::Reminder => "scenario=\"reminder\"",
            Scenario::IncomingCall => "scenario=\"incomingCall\"",
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

    fn create_template(&self) -> windows::core::Result<ToastNotification> {
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

        toast_xml.LoadXml(&HSTRING::from(format!(
            "<toast {} {}>
                    <visual>
                        <binding template=\"{}\">
                        {}
                        {}{}{}
                        </binding>
                    </visual>
                    {}
                </toast>",
            self.duration, self.scenario, template_binding, self.images, self.title, self.line1, self.line2, self.audio,
        )))?;

        // Create the toast
        ToastNotification::CreateToastNotification(&toast_xml)
    }

    /// Display the toast on the screen
    pub fn show(&self) -> windows::core::Result<()> {
        let toast_template = self.create_template()?;

        let toast_notifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(&self.app_id))?;

        // Show the toast.
        let result = toast_notifier.Show(&toast_template);
        std::thread::sleep(std::time::Duration::from_millis(10));
        result
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::path::Path;

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
            .duration(Duration::Short)
            //.sound(Some(Sound::Loop(LoopableSound::Call)))
            //.sound(Some(Sound::SMS))
            .sound(None)
            .show()
            // silently consume errors
            .expect("notification failed");
    }
}
