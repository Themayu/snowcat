use crate::default;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::BTreeMap;
use std::fmt::{Debug, Display, Formatter};
use std::str::FromStr;

const BUILTIN_DARK: &str = "dark";
const BUILTIN_FCHAT: &str = "fchat";
const BUILTIN_LIGHT: &str = "light";

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Settings {
	pub appearance: AppearanceSettings,
	pub client: ClientSettings,
	pub keyboard_shortcuts: KeyboardShortcuts,
	pub logs: LoggerSettings,
	pub notifications: NotificationSettings,
}

pub enum SettingsUpdate {
	AppearanceSettingsUpdate(AppearanceSettingsUpdate),
	ClientSettingsUpdate(ClientSettingsUpdate),
	KeyboardShortcutUpdate(KeyboardShortcutUpdate),
	LoggerSettingsUpdate(LoggerSettingsUpdate),
	NotificationSettingsUpdate(NotificationSettingsUpdate),
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct AppearanceSettings {
	pub clock_format: ClockFormat,
	pub display_size: DisplaySize,
	
	#[serde(with = "serde_impl::color_scheme_to_string")]
	pub theme: ColorScheme,

	#[serde(rename = "use_native_appearance", with = "serde_impl::window_appearance_to_bool")]
	pub window_appearance: WindowAppearance,

	// tables must come last
	pub show_avatars_in: ProfileAvatarLocations,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppearanceSettingsUpdate {
	SetAvatarsInChannels(bool),
	SetAvatarsInConsole(bool),
	SetAvatarsInPrivateMessages(bool),
	SetAvatarsInProfileLinks(bool),
	SetAvatarsInSystemMessages(bool),
	
	SetClockFormat(ClockFormat),
	SetDisplaySize(DisplaySize),
	SetColorScheme(#[serde(with = "serde_impl::color_scheme_to_string")] ColorScheme),
	SetCustomTitlebar(bool),
}

/// How to display time-based elements such as message timestamps.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub enum ClockFormat {
	/// 12-hour format
	#[default]
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

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorScheme {
	/// A dark default color scheme based on Discord.
	#[default]
	Dark,

	/// A purplish color scheme based on F-Chat 3.0.
	FChat,

	/// A bright default color scheme based on Discord.
	Light,

	/// A custom color scheme provided by the user, identified by file name.
	Custom(String),
}

/// How to display interactive elements such as messages and channels on the
/// user interface.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DisplaySize {
	/// Display items in a compact form if possible, with no supplement images.
	Compact,

	/// Display all items in an expanded form if possible, including supplement
	/// images.
	#[default]
	Large,
}

/// Where profile avatars should be displayed on the client
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ProfileAvatarLocations {
	pub channels: bool,
	pub console: bool,
	pub private_messages: bool,
	pub profile_links: bool,
	pub system_messages: bool,
}

default!(ProfileAvatarLocations => ProfileAvatarLocations {
	channels: true,
	console: true,
	private_messages: true,
	profile_links: true,
	system_messages: true,
});

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowAppearance {
	Custom,

	#[default]
	Native,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ClientSettings {
	pub animate_eicons: bool,

	#[serde(rename = "character_name_click_opens")]
	pub click_open_target: ClickOpenTarget,
	pub exclude_tags: Vec<String>,

	// tables must come last
	pub inactivity_timer: InactivityTimer,
	pub system_messages: SystemMessages,
}

default!(ClientSettings => ClientSettings {
	animate_eicons: true,
	click_open_target: ClickOpenTarget::default(),
	exclude_tags: Vec::default(),
	inactivity_timer: InactivityTimer::default(),
	system_messages: SystemMessages::default(),
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ClientSettingsUpdate {
	SetAnimateEIcons(bool),
	SetClickOpenTarget(ClickOpenTarget),

	AddExcludedTagsEntry(String),
	AlterExcludedTagsEntry { old: String, new: String },
	RemoveExcludedTagsEntry(String),
	SetExcludedTags(Vec<String>),

	SetInactivityTimerEnabled(bool),
	SetInactivityTimer(f32),

	SetSystemMessagesEnabled(bool),
	SetDisplaySystemMessagesInActiveChannel(bool),
	SetDisplaySystemMessagesInConsole(bool),
	SetSystemMessageNotifications(bool),
}

/// What to open when a character's name is clicked.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClickOpenTarget {
	/// Open the character's profile.
	#[default]
	CharacterProfile,

	/// Open and focus a private messages channel with the character.
	PrivateMessages,
}

/// How long to wait before setting the user's status to idle.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct InactivityTimer {
	pub enabled: bool,
	pub timer: f32,
}

default!(InactivityTimer => InactivityTimer {
	enabled: true,
	timer: 15.0,
});

/// Where to show messages from the F-Chat server.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SystemMessages {
	pub enabled: bool,
	pub display_in_console: bool,
	pub display_in_chats: bool,
	pub notify: bool,
}

default!(SystemMessages => SystemMessages {
	enabled: true,
	display_in_console: true,
	display_in_chats: true,
	notify: true,
});

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct KeyboardShortcuts {
	pub use_custom_bindings: bool,

	// tables must come last
	#[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
	pub movement: BTreeMap<MovementShortcut, String>,

	#[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
	pub text_format: BTreeMap<TextFormatShortcut, String>,
}

default!(KeyboardShortcuts => KeyboardShortcuts {
	movement: defaults::default_movement_keybinds(),
	text_format: defaults::default_text_format_keybinds(),
	use_custom_bindings: false,
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum KeyboardShortcutUpdate {
	SetCustomBindings(bool),
	
	SetMovementShortcut(MovementShortcut, String),
	ResetMovementShortcut(MovementShortcut),

	SetTextFormatShortcut(TextFormatShortcut, String),
	ResetTextFormatShortcut(TextFormatShortcut),
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MovementShortcut {
	FocusTextBox,
	Navigate,
	NextItem,
	NextSection,
	OpenSearch,
	PreviousItem,
	PreviousSection,
}

impl Display for MovementShortcut {
	fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
		let str = match self {
			MovementShortcut::FocusTextBox => "focus_textbox",
			MovementShortcut::Navigate => "navigate",
			MovementShortcut::NextItem => "next_channel",
			MovementShortcut::NextSection => "next_section",
			MovementShortcut::PreviousItem => "previous_channel",
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
			"navigate" => Ok(MovementShortcut::Navigate),
			"next_channel" => Ok(MovementShortcut::NextItem),
			"next_section" => Ok(MovementShortcut::NextSection),
			"previous_channel" => Ok(MovementShortcut::PreviousItem),
			"previous_section" => Ok(MovementShortcut::PreviousSection),
			"open_search" => Ok(MovementShortcut::OpenSearch),

			_ => Err(format!("Expected navigation keybind, got `{}`", value))
		}
	}
}

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggerSettings {
	pub log_ads: bool,
	pub log_messages: bool,
	pub storage_method: LogStorageMethod,
}

default!(LoggerSettings => LoggerSettings {
	log_ads: true,
	log_messages: true,
	storage_method: LogStorageMethod::default(),
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum LoggerSettingsUpdate {
	SetLogAds(bool),
	SetLogMessages(bool),
}

/// How message logs should be stored by the client.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogStorageMethod {
	/// Stores logs via a custom binary format. Less error-prone than plain
	/// text, but is slow to use and impossible to read with other tools.
	Binary,

	/// Stores logs via a quick and robust embedded database engine. Provides
	/// fast read and write times, but requires special tools to read logs
	/// outside of Snowcat.
	#[default]
	Database,

	/// Stores logs via plain text files. Easy to read ouutside of Snowcat, but
	/// is the slowest and most error-prone storage option, and vulnerable to
	/// message fabrication. Not recommended.
	Text,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct NotificationSettings {
	pub word_list: Vec<String>,

	// tables must come last
	pub in_app_notifications: InAppNotificationSettings,
	pub native_notifications: NativeNotificationSettings,
	pub notification_types: NotificationTypes,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum NotificationSettingsUpdate {
	AddWordListEntry(String),
	AlterWordListEntry { old: String, new: String },
	RemoveWordListEntry(String),
	SetWordList(Vec<String>),

	SetInAppNotifications(bool),
	SetInAppNotificationAutoDismiss(bool),
	SetInAppNotificationDismissalTimer(f32),
	SetNativeNotifications(bool),

	SetNotifyAnnouncements(bool),
	SetNotifyMentions(bool),
	SetNotifyPrivateMessages(bool),
	SetNotifyWordList(bool),
}

/// Settings for in-app notifications, if enabled.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct InAppNotificationSettings {
	pub enabled: bool,
	pub auto_dismiss: bool,
	pub dismissal_timer: f32,
}

default!(InAppNotificationSettings => InAppNotificationSettings {
	enabled: true,
	auto_dismiss: true,
	dismissal_timer: 15.0,
});

/// Settings for native notifications, if enabled.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct NativeNotificationSettings {
	pub enabled: bool,
}

default!(NativeNotificationSettings => NativeNotificationSettings {
	enabled: true,
});

/// What notifications should be displayed to the user.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct NotificationTypes {
	pub announcements: bool,
	pub mentions: bool,
	pub private_messages: bool,
	pub word_list: bool,
}

default!(NotificationTypes => NotificationTypes {
	announcements: true,
	mentions: true,
	private_messages: true,
	word_list: true,
});

pub trait SettingsContainer {
	type Context: Debug;
	type Error: Debug + std::error::Error;

	/// Apply a change to a setting listed under the "Appearance" settings pane.
	fn apply_appearance_setting_update(&mut self, update: AppearanceSettingsUpdate, ctx: &mut Self::Context) -> Result<(), Self::Error>;

	/// Apply a change to a setting listed under the "Keyboard Shortcuts"
	/// settings pane.
	fn apply_keyboard_shortcut_update(&mut self, update: KeyboardShortcutUpdate, ctx: &mut Self::Context) -> Result<(), Self::Error>;

	/// Apply a change to a setting listed under the "General" settings pane.
	fn apply_client_setting_update(&mut self, update: ClientSettingsUpdate, ctx: &mut Self::Context) -> Result<(), Self::Error>;

	/// Apply a change to a setting listed under the "Logger" settings pane.
	fn apply_logger_setting_update(&mut self, update: LoggerSettingsUpdate, ctx: &mut Self::Context) -> Result<(), Self::Error>;

	/// Apply a change to a setting listed under the "Notifications" settings
	/// pane.
	fn apply_notification_setting_update(&mut self, update: NotificationSettingsUpdate, ctx: &mut Self::Context) -> Result<(), Self::Error>;

	/// Commit changes to permanent storage.
	fn commit_changes(&self, _ctx: &mut Self::Context) -> Result<(), Self::Error> {
		// noop
		Ok(())
	}

	/// Apply and commit a set of configuration changes.
	fn apply_changes(&mut self, updates: Vec<SettingsUpdate>, ctx: &mut Self::Context) -> Result<(), Self::Error> {
		for update in updates {
			let result = match update {
				SettingsUpdate::AppearanceSettingsUpdate(update) => {
					self.apply_appearance_setting_update(update, ctx)
				},

				SettingsUpdate::ClientSettingsUpdate(update) => {
					self.apply_client_setting_update(update, ctx)
				},

				SettingsUpdate::KeyboardShortcutUpdate(update) => {
					self.apply_keyboard_shortcut_update(update, ctx)
				},

				SettingsUpdate::LoggerSettingsUpdate(update) => {
					self.apply_logger_setting_update(update, ctx)
				},

				SettingsUpdate::NotificationSettingsUpdate(update) => {
					self.apply_notification_setting_update(update, ctx)
				},
			};

			match result {
				Ok(()) => {},
				Err(error) => return Err(error),
			}
		}

		self.commit_changes(ctx)
	}
}

pub mod defaults {
	use crate::state::settings::{MovementShortcut, TextFormatShortcut};
	use std::collections::BTreeMap;

	pub fn default_movement_keybind(shortcut: MovementShortcut) -> String {
		match shortcut {
			MovementShortcut::FocusTextBox => String::from("Tab"),
			MovementShortcut::Navigate => String::from("Tab"),
			MovementShortcut::NextItem => String::from("Alt+ArrowDown"),
			MovementShortcut::NextSection => String::from("Alt+ArrowRight"),
			MovementShortcut::OpenSearch => String::from("Ctrl+KeyT"),
			MovementShortcut::PreviousItem => String::from("Alt+ArrowUp"),
			MovementShortcut::PreviousSection => String::from("Alt+ArrowLeft"),
		}
	}
	
	pub fn default_movement_keybinds() -> BTreeMap<MovementShortcut, String> {
		BTreeMap::from([
			(MovementShortcut::FocusTextBox, default_movement_keybind(MovementShortcut::FocusTextBox)),
			(MovementShortcut::Navigate, default_movement_keybind(MovementShortcut::Navigate)),
			(MovementShortcut::NextItem, default_movement_keybind(MovementShortcut::NextItem)),
			(MovementShortcut::NextSection, default_movement_keybind(MovementShortcut::NextSection)),
			(MovementShortcut::OpenSearch, default_movement_keybind(MovementShortcut::OpenSearch)),
			(MovementShortcut::PreviousItem, default_movement_keybind(MovementShortcut::PreviousItem)),
			(MovementShortcut::PreviousSection, default_movement_keybind(MovementShortcut::PreviousSection)),
		])
	}
	
	pub fn default_text_format_keybind(shortcut: TextFormatShortcut) -> String {
		match shortcut {
			TextFormatShortcut::Bold => String::from("Ctrl+KeyB"),
			TextFormatShortcut::Color => String::from("Ctrl+KeyI"),
			TextFormatShortcut::InsertCharacterIcon => String::from("Ctrl+KeyU"),
			TextFormatShortcut::InsertEicon => String::from("Ctrl+KeyS"),
			TextFormatShortcut::Italic => String::from("Ctrl+KeyD"),
			TextFormatShortcut::LinkCharacter => String::from("Ctrl+ArrowUp"),
			TextFormatShortcut::LinkUrl => String::from("Ctrl+ArrowDown"),
			TextFormatShortcut::NoParse => String::from("Ctrl+KeyL"),
			TextFormatShortcut::Spoiler => String::from("Ctrl+KeyR"),
			TextFormatShortcut::Strikethrough => String::from("Ctrl+KeyO"),
			TextFormatShortcut::Subscript => String::from("Ctrl+KeyE"),
			TextFormatShortcut::Superscript => String::from("Ctrl+KeyK"),
			TextFormatShortcut::Underline => String::from("Ctrl+KeyN"),
		}
	}
	
	pub fn default_text_format_keybinds() -> BTreeMap<TextFormatShortcut, String> {
		BTreeMap::from([
			(TextFormatShortcut::Bold, default_text_format_keybind(TextFormatShortcut::Bold)),
			(TextFormatShortcut::Italic, default_text_format_keybind(TextFormatShortcut::Italic)),
			(TextFormatShortcut::Underline, default_text_format_keybind(TextFormatShortcut::Underline)),
			(TextFormatShortcut::Strikethrough, default_text_format_keybind(TextFormatShortcut::Strikethrough)),
			(TextFormatShortcut::Color, default_text_format_keybind(TextFormatShortcut::Color)),
			(TextFormatShortcut::Superscript, default_text_format_keybind(TextFormatShortcut::Superscript)),
			(TextFormatShortcut::Subscript, default_text_format_keybind(TextFormatShortcut::Subscript)),
			(TextFormatShortcut::LinkUrl, default_text_format_keybind(TextFormatShortcut::LinkUrl)),
			(TextFormatShortcut::LinkCharacter, default_text_format_keybind(TextFormatShortcut::LinkCharacter)),
			(TextFormatShortcut::InsertCharacterIcon, default_text_format_keybind(TextFormatShortcut::InsertCharacterIcon)),
			(TextFormatShortcut::InsertEicon, default_text_format_keybind(TextFormatShortcut::InsertEicon)),
			(TextFormatShortcut::Spoiler, default_text_format_keybind(TextFormatShortcut::Spoiler)),
			(TextFormatShortcut::NoParse, default_text_format_keybind(TextFormatShortcut::NoParse)),
		])
	}
}

mod serde_impl {
	pub mod color_scheme_to_string {
		use crate::state::settings::{BUILTIN_DARK, BUILTIN_FCHAT, BUILTIN_LIGHT, ColorScheme};
		use serde::{Serializer, Deserializer};

		pub fn deserialize<'de, D>(deserializer: D) -> Result<ColorScheme, D::Error>
		where D: Deserializer<'de> {
			use serde::de::{self, Visitor};
			use std::fmt;
	
			struct FieldVisitor;
			impl<'de> Visitor<'de> for FieldVisitor {
				type Value = ColorScheme;
	
				fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
					write!(formatter, "a theme name")
				}
	
				fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
				where E: de::Error, {
					let kind = match value {
						BUILTIN_DARK => ColorScheme::Dark,
						BUILTIN_FCHAT => ColorScheme::FChat,
						BUILTIN_LIGHT => ColorScheme::Light,
	
						name => ColorScheme::Custom(name.to_owned())
					};
	
					Ok(kind)
				}
			}
	
			deserializer.deserialize_str(FieldVisitor)
		}
		
		pub fn serialize<S>(value: &ColorScheme, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer {
			let theme = match &value {
				ColorScheme::Dark => self::BUILTIN_DARK,
				ColorScheme::FChat => self::BUILTIN_FCHAT,
				ColorScheme::Light => self::BUILTIN_LIGHT,
				ColorScheme::Custom(theme) => theme.as_str()
			};
	
			serializer.serialize_str(theme)
		}	
	}

	pub mod window_appearance_to_bool {
		use crate::state::settings::WindowAppearance;
		use serde::{Serializer, Deserializer};
	
		pub fn deserialize<'de, D>(deserializer: D) -> Result<WindowAppearance, D::Error>
		where D: Deserializer<'de> {
			use serde::de::{self, Visitor};
			use std::fmt;
	
			struct WindowAppearanceVisitor;
			impl<'de> Visitor<'de> for WindowAppearanceVisitor {
				type Value = WindowAppearance;
	
				fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
					formatter.write_str("a boolean value")
				}
	
				fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
				where E: de::Error, {
					Ok(match value {
						true => WindowAppearance::Native,
						false => WindowAppearance::Custom,
					})
				}
			}
	
			deserializer.deserialize_bool(WindowAppearanceVisitor)
		}
	
		pub fn serialize<S>(value: &WindowAppearance, serializer: S) -> Result<S::Ok, S::Error>
		where S: Serializer {
			serializer.serialize_bool(matches!(value, WindowAppearance::Native))
		}
	}
}
