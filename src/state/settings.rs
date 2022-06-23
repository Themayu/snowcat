use futures_signals::signal::Mutable;
use futures_signals::signal_vec::MutableVec;
use snowcat_common::settings::{
	self as common_settings, ClickOpenTarget, ClockFormat, DisplaySize, LogStorageMethod, MovementShortcut,
	TextFormatShortcut, WindowAppearance, ColorScheme,
};
use std::collections::HashMap;

// Stores the application settings.
#[derive(Debug)]
pub struct AppSettings {
	pub client: ClientSettings,
	pub logger: LoggerSettings,
	pub notifications: NotificationSettings,
	pub shortcuts: KeyboardShortcuts,
}

impl From<common_settings::Settings> for AppSettings {
	fn from(settings: common_settings::Settings) -> Self {
		AppSettings {
			client: ClientSettings::from(settings.client),
			logger: LoggerSettings::from(settings.logger),
			notifications: NotificationSettings::from(settings.notifications),
			shortcuts: KeyboardShortcuts::from(settings.shortcuts),
		}
	}
}

#[derive(Debug)]
pub struct ClientSettings {
	pub animate_eicons: Mutable<bool>,
	pub click_open_target: Mutable<ClickOpenTarget>,
	pub clock_format: Mutable<ClockFormat>,
	pub display_size: Mutable<DisplaySize>,
	pub exclude_tags: MutableVec<String>,
	pub inactivity_timer: Mutable<Option<f32>>,
	pub show_avatars_in: ProfileAvatarLocations,
	pub theme: Mutable<ColorScheme>,
	pub window_appearance: Mutable<WindowAppearance>,
}

impl From<common_settings::ClientSettings> for ClientSettings {
	fn from(client: common_settings::ClientSettings) -> Self {
		ClientSettings {
			animate_eicons: Mutable::new(client.animate_eicons),
			click_open_target: Mutable::new(client.click_open_target),
			clock_format: Mutable::new(client.clock_format),
			display_size: Mutable::new(client.display_size),
			exclude_tags: MutableVec::new_with_values(client.exclude_tags),
			inactivity_timer: Mutable::new(client.inactivity_timer),
			show_avatars_in: ProfileAvatarLocations::from(client.show_avatars_in),
			theme: Mutable::new(client.theme),
			window_appearance: Mutable::new(client.window_appearance),
		}
	}
}

/// Where profile avatars should be displayed on the client
#[derive(Debug)]
pub struct ProfileAvatarLocations {
	pub channels: Mutable<bool>,
	pub character_lists: Mutable<bool>,
	pub console: Mutable<bool>,
	pub private_conversations: Mutable<bool>,
	pub profile_links: Mutable<bool>,
}

impl From<common_settings::ProfileAvatarLocations> for ProfileAvatarLocations {
	fn from(profile_avatar_locations: common_settings::ProfileAvatarLocations) -> Self {
		ProfileAvatarLocations {
			channels: Mutable::new(profile_avatar_locations.channels),
			character_lists: Mutable::new(profile_avatar_locations.character_lists),
			console: Mutable::new(profile_avatar_locations.console),
			private_conversations: Mutable::new(profile_avatar_locations.private_conversations),
			profile_links: Mutable::new(profile_avatar_locations.profile_links),
		}
	}
}

#[derive(Debug)]
pub struct LoggerSettings {
	pub log_ads: Mutable<bool>,
	pub log_messages: Mutable<bool>,
	pub storage_method: Mutable<LogStorageMethod>,
}

impl From<common_settings::LoggerSettings> for LoggerSettings {
	fn from(logger: common_settings::LoggerSettings) -> Self {
		LoggerSettings {
			log_ads: Mutable::new(logger.log_ads),
			log_messages: Mutable::new(logger.log_messages),
			storage_method: Mutable::new(logger.storage_method),
		}
	}
}

#[derive(Debug)]
pub struct NotificationSettings {
	pub in_app_notifications: Mutable<bool>,
	pub in_app_notification_timer: Mutable<Option<f32>>,
	pub native_notifications: Mutable<bool>,
	pub notify_for: NotificationSets,
	pub word_list: MutableVec<String>,
}

impl From<common_settings::NotificationSettings> for NotificationSettings {
	fn from(notifications: common_settings::NotificationSettings) -> Self {
		NotificationSettings {
			in_app_notifications: Mutable::new(notifications.in_app_notifications),
			in_app_notification_timer: Mutable::new(notifications.in_app_notification_timer),
			native_notifications: Mutable::new(notifications.native_notifications),
			notify_for: NotificationSets::from(notifications.notify_for),
			word_list: MutableVec::new_with_values(notifications.word_list),
		}
	}
}

/// What notifications should be displayed to the user
#[derive(Debug)]
pub struct NotificationSets {
	pub announcements: Mutable<bool>,
	pub mentions: Mutable<bool>,
	pub private_messages: Mutable<bool>,
	pub word_list_entries: Mutable<bool>,
}

impl From<common_settings::NotificationSets> for NotificationSets {
	fn from(notification_sets: common_settings::NotificationSets) -> Self {
		NotificationSets {
			announcements: Mutable::new(notification_sets.announcements),
			mentions: Mutable::new(notification_sets.mentions),
			private_messages: Mutable::new(notification_sets.private_messages),
			word_list_entries: Mutable::new(notification_sets.word_list_entries),
		}
	}
}

#[derive(Debug)]
pub struct KeyboardShortcuts {
	pub use_custom_bindings: Mutable<bool>,
	pub movement: Mutable<HashMap<MovementShortcut, String>>,
	pub text_format: Mutable<HashMap<TextFormatShortcut, String>>,
}

impl From<common_settings::KeyboardShortcuts> for KeyboardShortcuts {
	fn from(shortcuts: common_settings::KeyboardShortcuts) -> Self {
		let movement = shortcuts.movement.into_iter().map(|(key, value)| (key, value.to_owned())).collect();
		let text_format = shortcuts.text_format.into_iter().map(|(key, value)| (key, value.to_owned())).collect();

		KeyboardShortcuts {
			use_custom_bindings: Mutable::new(shortcuts.use_custom_bindings),
			movement: Mutable::new(movement),
			text_format: Mutable::new(text_format),
		}
	}
}
