use std::fmt;
use xml::escape::escape_str_attribute;

#[derive(Display, Debug, Clone, Copy, IntoStaticStr)]
#[strum(serialize_all = "camelCase")]
pub enum ToastInputType {
    // Textbox
    Text,

    // TODO what is this?
    Selection,
}

#[allow(non_snake_case)]
pub struct ToastInput {
    // for developers to retrieve user inputs once the app is activated (in the foreground or background)
    id: String,

    input_type: ToastInputType,

    // the optional title attribute and is for developers to specify a title for the input for shells to render when there is affordance.
    title: String,

    // the optional placeholderContent attribute and is the grey-out hint text for text input type. This attribute is ignored when the input type is not text
    placeHolderContent: String,

    // the optional defaultInput attribute and it allows developer to provide a default input value.
    defaultInput: String
}

impl ToastInput {
    #[allow(dead_code)]
    pub fn new(id: &str) -> ToastInput {
        ToastInput {
            id: escape_str_attribute(id).to_string(),
            input_type: ToastInputType::Text,
            title: String::new(),
            placeHolderContent: String::new(),
            defaultInput: String::new(),
        }
    }

    pub fn id(mut self, id: &str) -> ToastInput {
        self.id = escape_str_attribute(id).to_string();
        self
    }

    pub fn input_type(mut self, input_type: &ToastInputType) -> ToastInput {
        self.input_type = input_type.clone();
        self
    }

    pub fn title(mut self, title: &str) -> ToastInput {
        self.title = escape_str_attribute(title).to_string();
        self
    }

    pub fn place_holder_content(mut self, place_holder_content: &str) -> ToastInput {
        self.placeHolderContent = escape_str_attribute(place_holder_content).to_string();
        self
    }

    pub fn default_input(mut self, default_input: &str) -> ToastInput {
        self.defaultInput = escape_str_attribute(default_input).to_string();
        self
    }
}

impl fmt::Display for ToastInput {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut optional_fields = String::new();
        if self.title != "" {
            optional_fields += &format!(r#"title="{}" "#, self.title);
        }
        if self.placeHolderContent != "" {
            optional_fields += &format!(r#"placeHolderContent="{}" "#, self.placeHolderContent);
        }
        if self.defaultInput != "" {
            optional_fields += &format!(r#"defaultInput="{}" "#, self.defaultInput);
        }
        let input_text = format!(r#"<input id="{}" type="{}" {} />"#, self.id, self.input_type, optional_fields);
        fmt.write_str(&input_text)?;
        Ok(())
    }
}
