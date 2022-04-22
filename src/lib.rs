//! An incomplete wrapper over the WinRT toast api
//!
//! Tested in Windows 10 and 8.1. Untested in Windows 8, might work.
//!
//! Todo:
//!
//! * Add support for Adaptive Content
//! * Improve support for Actions
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

use std::collections::HashMap;
use std::convert::TryInto;
use std::path::Path;
use windows::UI::Notifications::ToastActivatedEventArgs;
use windows::{
    Data::Xml::Dom::XmlDocument,
    Foundation::TypedEventHandler,
    UI::Notifications::ToastNotificationManager,
};

use xml::escape::escape_str_attribute;
mod windows_check;

pub use windows::runtime::{
    Error,
    IInspectable,
    Result,
    HSTRING,
};
pub use windows::UI::Notifications::{
    ToastDismissedEventArgs,
    ToastFailedEventArgs,
    ToastNotification,
};

pub struct Toast {
    duration: String,
    title: String,
    line1: String,
    line2: String,
    images: String,
    audio: String,
    app_id: String,
    scenario: String,
    inputs: String,
    actions: String,
}

type ToastHandler<Inner> = Option<Box<dyn FnMut(&Option<ToastNotification>, &Option<Inner>) -> windows::runtime::Result<()> + Send>>;

/// Wrapper for [`Toast`] to add handlers for the different events
pub struct ToastWithHandlers {
    toast: Toast,
    activate_handler: ToastHandler<IInspectable>,
    dismiss_handler: ToastHandler<ToastDismissedEventArgs>,
    fail_handler: ToastHandler<ToastFailedEventArgs>,
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
    /// This will be displayed pre-expanded and stay on the user's screen till dismissed.
    Reminder,
    /// This will be displayed pre-expanded in a special call format and stay on the user's screen till dismissed. Audio will loop by default and will use ringtone audio.
    IncomingCall,
}

/// Specifies the id and text of a selection item.
///
/// See <https://docs.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-selection>
#[derive(Debug, Clone)]
pub struct SelectionItem {
    /// The id of the selection item. If no id is given the content will be used instead.
    pub id: Option<String>,
    /// The content of the selection item.
    pub content: String,
}

impl SelectionItem {
    /// Create a [`SelectionItem`] with `id` set to `content`
    pub fn from_content<S: ToString>(content: S) -> Self {
        Self {
            id: None,
            content: content.to_string(),
        }
    }
}

/// Specifies an input, either text box or selection menu, shown in a toast notification.
///
/// See <https://docs.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-input>
#[derive(Debug, Clone)]
pub enum InputType {
    /// Placeholder of the text.
    Text(Option<String>),
    /// The contents of the selection items.
    Selection(Vec<SelectionItem>),
}

impl InputType {
    pub fn text_with_placeholder<S: ToString>(placeholder: S) -> Self {
        Self::Text(Some(placeholder.to_string()))
    }

    pub fn from_selections<I: IntoIterator<Item = SelectionItem>>(iter: I) -> Self {
        Self::Selection(iter.into_iter().collect())
    }

    /// See [`SelectionItem::from_content`]
    pub fn from_selection_contents<S: ToString, I: IntoIterator<Item = S>>(iter: I) -> Self {
        Self::from_selections(iter.into_iter().map(SelectionItem::from_content))
    }
}

/// Specifies a button shown in a toast.
///
/// See <https://docs.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-action>
#[derive(Debug, Clone, Default)]
pub struct Action {
    /// The content displayed on the button.
    pub content: String,
    /// App-defined string of arguments that the app will later receive if the user clicks this button.
    /// Can be used to get the value in [`ActivatedEventArgs::get_user_input`].
    pub arguments: String,
    /// When set the action becomes a context menu action added to the toast notification's context menu rather than a traditional toast button.
    pub place_to_context_menu: bool,
}

impl Action {
    /// Create an [`Action`] and set `arguments` to `content`
    pub fn from_content<S: ToString>(content: S) -> Self {
        let content = content.to_string();
        Self {
            arguments: content.clone(),
            content,
            place_to_context_menu: false,
        }
    }
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
            inputs: String::new(),
            actions: String::new(),
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

    /// Add an input to the toast
    ///
    /// May be done up to 5 times
    /// The `id` can be used to get the value in [`ActivatedEventArgs::get_user_input`]
    pub fn input(mut self, ty: InputType, id: &str) -> Toast {
        let (ty_attr, children, placeholder) = match ty {
            InputType::Selection(selections) => (
                "selection",
                selections
                    .into_iter()
                    .map(|selection| {
                        let id = selection.id.unwrap_or_else(|| selection.content.clone());
                        format!(
                            r#"<selection id="{}" content="{}" />"#,
                            escape_str_attribute(&id),
                            escape_str_attribute(&selection.content),
                        )
                    })
                    .collect(),
                String::new(),
            ),
            InputType::Text(placeholder) => (
                "text",
                String::new(),
                placeholder
                    .map(|placeholder| format!(r#"placeHolderContent="{}""#, escape_str_attribute(&placeholder)))
                    .unwrap_or_default(),
            ),
        };

        self.inputs = format!(
            r#"{}<input id="{}" type="{}" {}>{}</input>"#,
            self.inputs,
            escape_str_attribute(id),
            ty_attr,
            placeholder,
            children,
        );
        self
    }

    /// Add an action to the toast
    ///
    /// May be done up to 5 times
    pub fn action(mut self, action: Action) -> Toast {
        let placement = if action.place_to_context_menu {
            r#"placement="contextMenu""#
        } else {
            ""
        };
        self.actions = format!(
            r#"{}<action content="{}" arguments="{}" {}/>"#,
            self.actions,
            escape_str_attribute(&action.content),
            escape_str_attribute(&action.arguments),
            placement,
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

    fn create_template(&self) -> windows::runtime::Result<ToastNotification> {
        //using this to get an instance of XmlDocument
        let toast_xml = XmlDocument::new()?;

        let template_binding = if windows_check::is_newer_than_windows81() {
            "ToastGeneric"
        } else
        //win8 or win81
        {
            // Need to do this or an empty placeholder will be shown if no image is set
            if self.images.is_empty() {
                "ToastText04"
            } else {
                "ToastImageAndText04"
            }
        };

        toast_xml.LoadXml(HSTRING::from(format!(
            "<toast {} {}>
                    <visual>
                        <binding template=\"{}\">
                        {}
                        {}{}{}
                        </binding>
                    </visual>
                    {}
                    <actions>
                        {}
                        {}
                    </actions>
                </toast>",
            self.duration,
            self.scenario,
            template_binding,
            self.images,
            self.title,
            self.line1,
            self.line2,
            self.audio,
            self.inputs,
            self.actions,
        )))?;

        // Create the toast
        ToastNotification::CreateToastNotification(toast_xml)
    }

    /// Display the toast on the screen
    pub fn show(&self) -> windows::runtime::Result<()> {
        let toast_template = self.create_template()?;

        let toast_notifier = ToastNotificationManager::CreateToastNotifierWithId(HSTRING::from(&self.app_id))?;

        // Show the toast.
        let result = toast_notifier.Show(&toast_template);
        std::thread::sleep(std::time::Duration::from_millis(10));
        result
    }
}

impl ToastWithHandlers {
    pub fn new(toast: Toast) -> Self {
        Self {
            toast,
            activate_handler: None,
            dismiss_handler: None,
            fail_handler: None,
        }
    }

    /// Occurs when user activates a toast notification through a click or touch.
    ///
    /// See <https://docs.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastnotification.activated>
    pub fn on_activate(mut self, handler: impl Fn(ActivatedEventArgs) -> windows::runtime::Result<()> + 'static + Send) -> Self {
        // These explicit type names are required because of lifetime interference
        // See https://github.com/rust-lang/rust/issues/70263
        let handler = move |notification: &'_ Option<ToastNotification>, args: &'_ Option<IInspectable>| {
            let event_args = ActivatedEventArgs { notification, args };
            handler(event_args)
        };
        self.activate_handler = Some(Box::new(handler));
        self
    }

    /// Occurs when a toast notification leaves the screen, either by expiring or being explicitly dismissed by the user.
    ///
    /// See <https://docs.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastnotification.dismissed>
    pub fn on_dismiss(
        mut self,
        handler: impl FnMut(&Option<ToastNotification>, &Option<ToastDismissedEventArgs>) -> windows::runtime::Result<()> + 'static + Send,
    ) -> Self {
        self.dismiss_handler = Some(Box::new(handler));
        self
    }

    /// Occurs when an error is caused when Windows attempts to raise a toast notification.
    ///
    /// See <https://docs.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastnotification.failed>
    pub fn on_fail(
        mut self,
        handler: impl FnMut(&Option<ToastNotification>, &Option<ToastFailedEventArgs>) -> windows::runtime::Result<()> + 'static + Send,
    ) -> Self {
        self.fail_handler = Some(Box::new(handler));
        self
    }

    /// Register the handlers and display the toast on the screen
    ///
    /// see: [`Toast::show`]
    pub fn show(self) -> windows::runtime::Result<()> {
        let toast_template = self.toast.create_template()?;

        // Add event handlers
        if let Some(handler) = self.activate_handler {
            toast_template.Activated(TypedEventHandler::new(handler))?;
        }
        if let Some(handler) = self.dismiss_handler {
            toast_template.Dismissed(TypedEventHandler::new(handler))?;
        }
        if let Some(handler) = self.fail_handler {
            toast_template.Failed(TypedEventHandler::new(handler))?;
        }

        let toast_notifier = ToastNotificationManager::CreateToastNotifierWithId(HSTRING::from(&self.toast.app_id))?;

        // Show the toast.
        let result = toast_notifier.Show(&toast_template);
        std::thread::sleep(std::time::Duration::from_millis(10));

        result
    }
}

/// Arguments received on activation
pub struct ActivatedEventArgs<'a> {
    /// Notification from which the event came
    pub notification: &'a Option<ToastNotification>,
    /// Raw value. Prefer using [`Self::get_raw_event_args`] to work with this value
    pub args: &'a Option<IInspectable>,
}

impl<'a> ActivatedEventArgs<'a> {
    /// Exposes a method that retrieves the arguments associated with a toast action initiated by the user.
    /// This lets the app tell which action was taken when multiple actions were exposed.
    ///
    /// See <https://docs.microsoft.com/en-us/uwp/api/windows.ui.notifications.toastactivatedeventargs>
    pub fn get_raw_event_args(&self) -> Option<windows::runtime::Result<ToastActivatedEventArgs>> {
        self.args.as_ref().map(windows::runtime::Interface::cast)
    }

    /// Gets the arguments associated with a toast action initiated by the user
    pub fn get_arguments(&self) -> Option<windows::runtime::Result<String>> {
        self.get_raw_event_args().map(|args| Ok(args?.Arguments()?.to_string_lossy()))
    }

    /// For toast notifications that include text boxes or selections for user input, contains the user input.
    pub fn get_user_input(&self) -> Option<windows::runtime::Result<HashMap<String, String>>> {
        self.get_raw_event_args().map(|args| {
            args?
                .UserInput()?
                .into_iter()
                .map(|pair| {
                    let key = pair.Key()?;
                    let value: HSTRING = pair.Value()?.try_into()?;

                    Ok((key.to_string_lossy(), value.to_string_lossy()))
                })
                .collect()
        })
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
