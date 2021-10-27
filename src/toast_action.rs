use std::fmt;
use xml::escape::escape_str_attribute;

#[derive(Display, Clone, Copy, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
pub enum ToastActivationType {
    // Default value. Your foreground app is launched.
    Foreground,

    // Your corresponding background task (assuming you set everything up) is triggered, and you can execute code in the background (like sending the user's quick reply message) without interrupting the user.
    Background,

    // Launch a different app using protocol activation.
    Protocol,

    // System handles the activation.
    System,
}

#[derive(Display, Clone, Copy, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
pub enum ToastAfterActivationBehavior {
    // Default behavior. The toast will be dismissed when the user takes action on the toast.
    Default,
    // After the user clicks a button on your toast, the notification will remain present, in a "pending update" visual state. You should immediately update your toast from a background task so that the user does not see this "pending update" visual state for too long.
    PendingUpdate,
}

#[derive(Display, Clone, Copy, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
pub enum ToastActionPlacement {
    Inline,

    ContextMenu,
}

pub struct ToastAction {
    // The text to be displayed on the button
    content: String,
    // The arguments attribute describing the app-defined data that the app can later retrieve once it is activated from user taking this action.
    arguments: String,

    activation_type: ToastActivationType, //activationType

    // Gets or sets the target PFN if you are using Protocol. You can optionally specify, so that regardless of whether multiple apps are registered to handle the same protocol uri, your desired app will always be launched.
    protocol_activation_target_application_pfn: String, //protocolActivationTargetApplicationPfn

    // Gets or sets the behavior that the toast should use when the user invokes this action. Note that this option only works on ToastButton and ToastContextMenuItem. Desktop-only, supported in builds 16251 or higher. New in Fall Creators Update
    after_activation_behavior: ToastAfterActivationBehavior, // afterActivationBehavior

    image_uri: String, // imageUrl

    hint_input_id: String, // hint-inputId

    placement: ToastActionPlacement,

    // Gets or sets an identifier used in telemetry to identify your category of action. This should be something like "Delete", "Reply", or "Archive". In the upcoming toast telemetry dashboard in Dev Center, you will be able to view how frequently your actions are being clicked.
    hint_action_id: String, // hint-actionId
}

impl ToastAction {
    #[allow(dead_code)]
    pub fn new() -> ToastAction {
        ToastAction {
            content: String::new(),
            arguments: String::new(),
            activation_type: ToastActivationType::Foreground,
            protocol_activation_target_application_pfn: String::new(),
            after_activation_behavior: ToastAfterActivationBehavior::Default,
            image_uri: String::new(),
            hint_input_id: String::new(),
            placement: ToastActionPlacement::Inline,
            hint_action_id: String::new(),
        }
    }

    pub fn text(mut self, content: &str) -> ToastAction {
        self.content = escape_str_attribute(content).to_string();
        self
    }

    pub fn arguments(mut self, argument: &str) -> ToastAction {
        self.arguments = escape_str_attribute(argument).to_string();
        self
    }

    // Id of the input this action should be associated with
    pub fn hint_input_id(mut self, hint_input_id: &str) -> ToastAction {
        self.hint_input_id = escape_str_attribute(hint_input_id).to_string();
        self
    }

    pub fn activation_type(mut self, activation_type: ToastActivationType) -> ToastAction {
        self.activation_type = activation_type;
        self
    }
}

impl fmt::Display for ToastAction {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let action_text = format!(r#"<action content="{}" arguments="{}" />"#, self.content, self.arguments);
        fmt.write_str(&action_text)?;
        Ok(())
    }
}
