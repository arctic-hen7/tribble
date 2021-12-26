use crate::errors::ParserError;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader};

/// Gets the default error message when the user doesn't fill out a mandatory field.
fn default_input_err_msg() -> String {
    "This field is required, please enter a value.".to_string()
}

/// The possible types of configuration files (this allows main files to be different from internationalization files).
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Config {
    /// A root configuration file that defines languages that have their own configuration files.
    Root {
        /// A map of the languages supported to filenames, a structure that separates each language into a separate Tribble file.
        languages: HashMap<String, String>,
    },
    /// A configuration file for a single language.
    Language {
        /// The error message when a user doesn't fill out a mandatory field. This is allowed to enable i18n at an arbitrary scale.
        #[serde(default = "default_input_err_msg")]
        input_err_msg: String,
        /// All the workflow in this Tribble instance. Each workflow is a separate contribution experience, and multiple workflows are generally best suited for things like separate products.
        workflows: HashMap<String, Workflow>,
    },
}
impl Config {
    /// Creates a new instance of the raw configuration from a file.
    pub fn new(filename: &str) -> Result<Self, ParserError> {
        // We'll parse it directly from a reader for efficiency
        let file = File::open(filename).map_err(|err| ParserError::FsError {
            filename: filename.to_string(),
            source: err,
        })?;
        let reader = BufReader::new(file);
        let contents: Self =
            serde_yaml::from_reader(reader).map_err(|err| ParserError::ParseRawError {
                filename: filename.to_string(),
                source: err,
            })?;

        Ok(contents)
    }
}

/// The components of a workflow.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Workflow {
    /// The tags supported for this page. These MUST NOT contains commas, or errors WILL occur outside of the Tribble interface, when attempting to automatically triage
    /// generated reports!
    pub tags: Vec<String>,
    /// The sections that the page can make use of.
    pub sections: HashMap<String, Section>,
    /// The section to start on, which must be a valid key in the `sections` map.
    pub index: String,
    /// The endpoints that the user can exit the process from.
    pub endpoints: HashMap<String, Endpoint>,
}
/// A type alias for a section, which is simply an ordered list of elements.
pub type Section = Vec<SectionElem>;
/// The possible parts of a section.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SectionElem {
    /// Simple text to be displayed to the user. If this begins with a `<` that's unescaped, it will be treated as arbitrary HTML, and will be directly injected into the page. In that
    /// case, it is assumed to be sanitized.
    Text(String),
    /// A progression option for moving to another section.
    Progression {
        /// The text to display to the user.
        text: String,
        /// The name of the section to navigate to. If this is prefixed with `endpoint:`, it will navigate to an endpoint instead of a section.
        link: String,
        /// Any tags that should be accumulated as a result of proceeding through this route.
        tags: Vec<String>,
    },
    /// A form input that the user can fill out. This must have an associated ID, because its value can be referenced later in an endpoint.
    Input(InputSectionElem),
}
/// The properties of an input element. This needs to be passed around, so it's broken out of the `SectionElem` input.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InputSectionElem {
    /// The input's ID, which can be used to reference its value later for interpolation in a formatted report.
    pub id: String,
    /// The label for the input.
    pub label: String,
    /// Whether or not the input is optional.
    #[serde(default)]
    pub optional: bool,
    /// The default value for the input. If the input is optional, this will be the value used for interpolation. If the input is not optional, this will be the default,
    /// which means it will be left as this if the user doesn't fill it in. If a value should be provided, you should make it mandatory and set a default, as optional fields should
    /// be assumed to potentially not contain any value (even though they always will if a default value is provided).
    ///
    /// If the input is a `Select`, this must correspond to an entry in `options`.
    pub default: Option<String>,
    /// The actual properties of the input (unique depending on the input's type).
    #[serde(flatten)]
    // The user can just continue to supply these properties without having to put them inside `input`
    pub input: Input,
}
/// The different types of inputs.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Input {
    /// Simple text.
    Text {
        /// The input's HTML type.
        #[serde(flatten)]
        // The user should be able to specify the properties in the same line as the rest of the input (with a `type` field as well)
        input_type: InputType,
    },
    /// A select element that provides a dropdown for the user to select a single option.
    Select {
        /// The options that the user can select from.
        options: Vec<SelectOption>,
        /// Whether or not the user can select multiple options.
        #[serde(default)]
        can_select_multiple: bool,
    },
}
/// The possible types an input can have.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum InputType {
    /// A boolean input.
    Boolean,
    /// A multiline text input.
    Multiline,
    /// A color picker (only in supported browsers).
    Color,
    /// A simple text element (default).
    Text,
    /// A date input.
    Date,
    /// A datetime input, with no time offset (by UTC has been deprecated at the standard-level).
    DatetimeLocal,
    /// An email input.
    Email,
    /// A month input.
    Month,
    /// A numerical input.
    Number {
        /// The smallest number the user can input.
        #[serde(default)]
        min: Option<i32>,
        /// The largest number the user can input.
        #[serde(default)]
        max: Option<i32>,
    },
    /// A password input (characters are obfuscated).
    Password,
    /// A range slider.
    Range {
        /// The minimum value on the slider.
        min: i32,
        /// The maximum value on the slider.
        max: i32,
    },
    /// A telephone number input.
    Tel,
    /// A time picker.
    Time,
    /// A URL input.
    Url,
    /// A week input.
    Week,
}
impl Default for InputType {
    fn default() -> Self {
        Self::Text
    }
}
impl ToString for InputType {
    fn to_string(&self) -> String {
        match self {
            Self::Boolean => "checkbox".to_string(),
            Self::Multiline => "multiline".to_string(),
            Self::Color => "color".to_string(),
            Self::Text => "text".to_string(),
            Self::Date => "date".to_string(),
            Self::DatetimeLocal => "datetime-local".to_string(),
            Self::Email => "email".to_string(),
            Self::Month => "month".to_string(),
            Self::Number { .. } => "number".to_string(),
            Self::Password => "password".to_string(),
            Self::Range { .. } => "range".to_string(),
            Self::Tel => "tel".to_string(),
            Self::Time => "time".to_string(),
            Self::Url => "url".to_string(),
            Self::Week => "week".to_string(),
        }
    }
}
/// The properties for an option for a select element. The text of this MUST NOT contain commas, otherwise all sorts of runtime errors WILL occur!
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum SelectOption {
    /// A select element that simply has a value.
    Simple(String),
    WithTags {
        /// The displayed text of the option.
        text: String,
        /// A list of tags that should be accumulated if this option is selected. If multiple options can be selected and there are duplications, tags will only be assigned once.
        tags: Vec<String>,
    },
}
/// The possible endpoint types (endpoints are sections that allow the user to exit the contribution process).
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Endpoint {
    /// A report endpoint, which gives the user a formatted report in Markdown to send to the project.
    // TODO Add functionality to actually send the report somewhere
    Report {
        /// The preamble text to display before the actual formatted report.
        preamble: String,
        /// The formatted report. The UI will not allow the user to edit this, but will provide a copy button. Interpolation of form values is allowed here with `${form_id}` syntax.
        text: String,
    },
    /// An instructional endpoint, which tells the user to do something.
    Instructional(String),
}
