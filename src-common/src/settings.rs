use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

const BUILTIN_DARK: &'static str = "dark";
const BUILTIN_FCHAT: &'static str = "fchat";
const BUILTIN_LIGHT: &'static str = "light";

#[derive(Debug, Deserialize, Serialize)]
pub struct Settings {
	#[serde(default)]
	pub client: ClientSettings,
	
	#[serde(default)]
	pub logger: LoggerSettings,

	#[serde(default)]
	pub notifications: NotificationSettings,

	#[serde(default)]
	pub shortcuts: KeyboardShortcuts,
}

impl Default for Settings {
	fn default() -> Self {
		Settings {
			client: ClientSettings::default(),
			logger: LoggerSettings::default(),
			notifications: NotificationSettings::default(),
			shortcuts: KeyboardShortcuts::default(),
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ClientSettings {
	pub animate_eicons: bool,
	pub character_name_click_opens: ClickOpenTarget,
	pub clock_format: ClockFormat,
	pub display_size: DisplaySize,
	pub exclude_tags: Vec<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub inactivity_timer: Option<f32>,
	pub theme: ColorScheme,

	// tables must come last
	#[serde(default)]
	pub show_avatars_in: ProfileAvatarLocations,
}

impl Default for ClientSettings {
	fn default() -> Self {
		ClientSettings {
			animate_eicons: false,
			character_name_click_opens: ClickOpenTarget::CharacterProfile,
			clock_format: ClockFormat::Meridiem,
			display_size: DisplaySize::Large,
			exclude_tags: vec![],
			inactivity_timer: Some(15.0),
			show_avatars_in: ProfileAvatarLocations::default(),
			theme: ColorScheme::Dark,
		}
	}
}

/// What to open when a character's name is clicked.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClickOpenTarget {
	/// Open the character's profile.
	CharacterProfile,

	/// Open and focus a private messages channel with the character.
	PrivateMessages,
}

/// How to display time-based elements such as message timestamps.
#[derive(Debug, Deserialize, Serialize)]
pub enum ClockFormat {
	/// 12-hour format
	#[serde(rename(
		deserialize = "meridiem",
		deserialize = "12-hour",
		deserialize = "12 hour",
		deserialize = "12",
		
		serialize = "12-hour"
	))]
	Meridiem,

	/// 24-hour format
	#[serde(rename(
		deserialize = "plenadiem",
		deserialize = "24-hour",
		deserialize = "24 hour",
		deserialize = "24",
		
		serialize = "24-hour"
	))]
	Plenadiem,
}

#[derive(Debug)]
pub enum ColorScheme {
	/// A dark default color scheme based on Discord.
	Dark,

	/// A purplish color scheme based on F-Chat 3.0.
	FChat,

	/// A bright default color scheme based on Discord.
	Light,

	/// A custom color scheme provided by the user, identified by file name.
	Custom(String),
}

impl<'de> Deserialize<'de> for ColorScheme {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where D: serde::Deserializer<'de> {
		use std::fmt;

		struct FieldVisitor;
		impl<'de> serde::de::Visitor<'de> for FieldVisitor {
			type Value = ColorScheme;

			fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(formatter, "a theme name")
			}

			fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
			where E: serde::de::Error, {
				let kind = match value {
					self::BUILTIN_DARK => ColorScheme::Dark,
					self::BUILTIN_FCHAT => ColorScheme::FChat,
					self::BUILTIN_LIGHT => ColorScheme::Light,

					name => ColorScheme::Custom(name.to_owned())
				};

				Ok(kind)
			}
		}

		deserializer.deserialize_str(FieldVisitor)
	}
}

impl Serialize for ColorScheme {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where S: serde::Serializer {
		let theme = match &self {
			ColorScheme::Dark => self::BUILTIN_DARK,
			ColorScheme::FChat => self::BUILTIN_FCHAT,
			ColorScheme::Light => self::BUILTIN_LIGHT,
			ColorScheme::Custom(theme) => theme.as_str()
		};

		serializer.serialize_str(theme)
	}
}

/// How to display interactive elements such as messages and channels on the
/// user interface.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplaySize {
	/// Display items in a compact form if possible, with no supplement images.
	Compact,

	/// Display all items in an expanded form if possible, including supplement
	/// images.
	Large,
}

/// Where profile avatars should be displayed on the client
#[derive(Debug, Deserialize, Serialize)]
pub struct ProfileAvatarLocations {
	pub channels: bool,
	pub character_lists: bool,
	pub console: bool,
	pub private_conversations: bool,
	pub profile_links: bool,
}

impl Default for ProfileAvatarLocations {
	fn default() -> Self {
		ProfileAvatarLocations {
			channels: true,
			character_lists: true,
			console: true,
			private_conversations: true,
			profile_links: true,
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LoggerSettings {
	pub log_ads: bool,
	pub log_messages: bool,
	pub storage_method: LogStorageMethod,
}

impl Default for LoggerSettings {
	fn default() -> Self {
		LoggerSettings {
			log_ads: true,
			log_messages: true,
			storage_method: LogStorageMethod::Database,
		}
	}
}

/// How message logs should be stored by the client.
#[derive(Debug, Deserialize, Serialize)]
pub enum LogStorageMethod {
	/// Stores logs via a custom binary format. Less error-prone than plain
	/// text, but is slow to use and impossible to read with other tools.
	Binary,

	/// Stores logs via a quick and robust embedded database engine. Provides
	/// fast read and write times, but requires special tools to read logs
	/// outside of Snowcat.
	Database,

	/// Stores logs via plain text files. Easy to read ouutside of Snowcat, but
	/// is the slowest and most error-prone storage option, and vulnerable to
	/// message fabrication. Not recommended.
	Text,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationSettings {
	pub in_app_notifications: bool,
	pub in_app_notification_timer: Option<f32>,
	pub native_notifications: bool,
	pub word_list: Vec<String>,

	// tables must come last
	#[serde(default)]
	pub notify_for: NotificationSets,
}

impl Default for NotificationSettings {
	fn default() -> Self {
		NotificationSettings {
			in_app_notifications: false,
			in_app_notification_timer: None,
			native_notifications: true,
			notify_for: NotificationSets::default(),
			word_list: vec![],
		}
	}
}

/// What notifications should be displayed to the user
#[derive(Debug, Deserialize, Serialize)]
pub struct NotificationSets {
	pub announcements: bool,
	pub mentions: bool,
	pub private_messages: bool,
	pub word_list_entries: bool,
}

impl Default for NotificationSets {
	fn default() -> Self {
		NotificationSets {
			announcements: true,
			mentions: true,
			private_messages: true,
			word_list_entries: false,
		}
	}
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize)]
pub struct KeyboardShortcuts {
	pub use_custom_bindings: bool,

	#[serde(default = "default_movement_keybinds")]
	#[serde_as(as = "HashMap<DisplayFromStr, _>")]
	pub movement: HashMap<MovementShortcut, String>,

	#[serde(default = "default_text_format_keybinds")]
    #[serde_as(as = "HashMap<DisplayFromStr, _>")]
    pub text_format: HashMap<TextFormatShortcut, String>,
}

impl Default for KeyboardShortcuts {
	fn default() -> Self {
		KeyboardShortcuts {
			use_custom_bindings: false,

			movement: default_movement_keybinds(),
		    text_format: default_text_format_keybinds(),
		}
	}
}

#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum MovementShortcut {
	FocusTextBox,
	NextChannel,
	NextSection,
	OpenSearch,
	PreviousChannel,
	PreviousSection,
}

impl Display for MovementShortcut {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let str = match self {
			MovementShortcut::FocusTextBox => "focus_textbox",
			MovementShortcut::NextChannel => "next_channel",
			MovementShortcut::NextSection => "next_section",
			MovementShortcut::PreviousChannel => "previous_channel",
			MovementShortcut::PreviousSection => "previous_section",
			MovementShortcut::OpenSearch => "open_search",
		};

		write!(formatter, "{}", str)
	}
}

impl FromStr for MovementShortcut {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			"focus_textbox" => Ok(MovementShortcut::FocusTextBox),
			"next_channel" => Ok(MovementShortcut::NextChannel),
			"next_section" => Ok(MovementShortcut::NextSection),
			"previous_channel" => Ok(MovementShortcut::PreviousChannel),
			"previous_section" => Ok(MovementShortcut::PreviousSection),
			"open_search" => Ok(MovementShortcut::OpenSearch),

			_ => Err(format!("Expected navigation keybind, got `{}`", value))
		}
	}
}

#[derive(Debug, Eq, Hash, PartialEq, Deserialize, Serialize)]
pub enum TextFormatShortcut {
    Bold,
    Color,
	InsertCharacterIcon,
	InsertEicon,
	Italic,
    LinkCharacter,
    LinkUrl,
	NoParse,
	Spoiler,
	Strikethrough,
	Subscript,
	Superscript,
	Underline,
}

impl Display for TextFormatShortcut {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let str = match self {
			TextFormatShortcut::Bold => "b",
			TextFormatShortcut::Color => "color",
			TextFormatShortcut::InsertCharacterIcon => "icon",
			TextFormatShortcut::InsertEicon => "eicon",
			TextFormatShortcut::Italic => "i",
			TextFormatShortcut::LinkCharacter => "user",
			TextFormatShortcut::LinkUrl => "url",
			TextFormatShortcut::NoParse => "noparse",
			TextFormatShortcut::Spoiler => "spoiler",
			TextFormatShortcut::Strikethrough => "s",
			TextFormatShortcut::Subscript => "sub",
			TextFormatShortcut::Superscript => "sup",
			TextFormatShortcut::Underline => "u",
		};

		write!(formatter, "{}", str)
	}
}

impl FromStr for TextFormatShortcut {
	type Err = String;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		match value {
			// Tag names
			"b" => Ok(TextFormatShortcut::Bold),
			"color" => Ok(TextFormatShortcut::Color),
			"eicon" => Ok(TextFormatShortcut::InsertEicon),
			"i" => Ok(TextFormatShortcut::Italic),
			"icon" => Ok(TextFormatShortcut::InsertCharacterIcon),
			"noparse" => Ok(TextFormatShortcut::NoParse),
			"s" => Ok(TextFormatShortcut::Strikethrough),
			"spoiler" => Ok(TextFormatShortcut::Spoiler),
			"sub" => Ok(TextFormatShortcut::Subscript),
			"sup" => Ok(TextFormatShortcut::Superscript),
			"u" => Ok(TextFormatShortcut::Underline),
			"url" => Ok(TextFormatShortcut::LinkUrl),
			"user" => Ok(TextFormatShortcut::LinkCharacter),

			// Expanded forms
			"bold" => Ok(TextFormatShortcut::Bold),
			"character" => Ok(TextFormatShortcut::LinkCharacter),
			"character_icon" => Ok(TextFormatShortcut::InsertCharacterIcon),
			"italic" => Ok(TextFormatShortcut::Italic),
			"strikethrough" => Ok(TextFormatShortcut::Strikethrough),
			"subscript" => Ok(TextFormatShortcut::Subscript),
			"superscript" => Ok(TextFormatShortcut::Superscript),
			"underline" => Ok(TextFormatShortcut::Underline),

			_ => Err(format!("Expected valid bbcode chat tag, got `{}`", value))
		}
	}
}

fn default_movement_keybinds() -> HashMap<MovementShortcut, String> {
	HashMap::from([
		(MovementShortcut::FocusTextBox, String::from("Tab")),
		(MovementShortcut::NextChannel, String::from("Alt+ArrowDown")),
		(MovementShortcut::NextSection, String::from("Alt+ArrowRight")),
		(MovementShortcut::OpenSearch, String::from("Ctrl+KeyT")),
		(MovementShortcut::PreviousChannel, String::from("Alt+ArrowUp")),
		(MovementShortcut::PreviousSection, String::from("Alt+ArrowLeft")),
	])
}

fn default_text_format_keybinds() -> HashMap<TextFormatShortcut, String> {
	HashMap::from([
		(TextFormatShortcut::Bold, String::from("Ctrl+KeyB")),
		(TextFormatShortcut::Italic, String::from("Ctrl+KeyI")),
		(TextFormatShortcut::Underline, String::from("Ctrl+KeyU")),
		(TextFormatShortcut::Strikethrough, String::from("Ctrl+KeyS")),
		(TextFormatShortcut::Color, String::from("Ctrl+KeyD")),
		(TextFormatShortcut::Superscript, String::from("Ctrl+ArrowUp")),
		(TextFormatShortcut::Subscript, String::from("Ctrl+ArrowDown")),
		(TextFormatShortcut::LinkUrl, String::from("Ctrl+KeyL")),
		(TextFormatShortcut::LinkCharacter, String::from("Ctrl+KeyR")),
		(TextFormatShortcut::InsertCharacterIcon, String::from("Ctrl+KeyO")),
		(TextFormatShortcut::InsertEicon, String::from("Ctrl+KeyE")),
		(TextFormatShortcut::Spoiler, String::from("Ctrl+KeyK")),
		(TextFormatShortcut::NoParse, String::from("Ctrl+KeyN")),
	])
}
