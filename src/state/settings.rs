use futures_signals::signal::Mutable;
use futures_signals::signal_map::MutableBTreeMap;
use futures_signals::signal_vec::MutableVec;
use snowcat_common::state::settings as common_settings;

#[derive(Debug)]
pub struct Settings {
	pub appearance: AppearanceSettings,
	pub client: ClientSettings,
	pub keyboard_shortcuts: KeyboardShortcuts,
	pub logs: LoggerSettings,
	pub notifications: NotificationSettings,
}

impl Settings {
	/// Merge a configuration fragment into this one, overwriting values if
	/// required.
	pub fn merge(&self, other: common_settings::Settings) {
		self.appearance.merge(other.appearance);
		self.client.merge(other.client);
		self.keyboard_shortcuts.merge(other.keyboard_shortcuts);
		self.logs.merge(other.logs);
		self.notifications.merge(other.notifications);
	}
}

impl From<common_settings::Settings> for Settings {
	fn from(remote: common_settings::Settings) -> Self {
		Self {
			appearance: remote.appearance.into(),
			client: remote.client.into(),
			keyboard_shortcuts: remote.keyboard_shortcuts.into(),
			logs: remote.logs.into(),
			notifications: remote.notifications.into(),
		}
	}
}

#[derive(Debug)]
pub struct AppearanceSettings {
	pub clock_format: Mutable<common_settings::ClockFormat>,
	pub display_size: Mutable<common_settings::DisplaySize>,
	pub show_avatars_in: ProfileAvatarLocations,
	pub theme: Mutable<common_settings::ColorScheme>,
	pub window_appearance: Mutable<common_settings::WindowAppearance>,
}

impl AppearanceSettings {
	/// Merge a configuration fragment into this one, overwriting values if
	/// required.
	pub fn merge(&self, other: common_settings::AppearanceSettings) {
		self.clock_format.set_neq(other.clock_format);
		self.display_size.set_neq(other.display_size);
		self.show_avatars_in.merge(other.show_avatars_in);
		self.theme.set_neq(other.theme);
		self.window_appearance.set_neq(other.window_appearance);
	}
}

impl From<common_settings::AppearanceSettings> for AppearanceSettings {
	fn from(remote: common_settings::AppearanceSettings) -> Self {
		Self {
			clock_format: Mutable::new(remote.clock_format),
			display_size: Mutable::new(remote.display_size),
			show_avatars_in: remote.show_avatars_in.into(),
			theme: Mutable::new(remote.theme),
			window_appearance: Mutable::new(remote.window_appearance),
		}
	}
}

/// Where profile avatars should be displayed on the client
#[derive(Debug)]
pub struct ProfileAvatarLocations {
	pub channels: Mutable<bool>,
	pub console: Mutable<bool>,
	pub private_messages: Mutable<bool>,
	pub profile_links: Mutable<bool>,
	pub system_messages: Mutable<bool>,
}

impl ProfileAvatarLocations {
	fn merge(&self, other: common_settings::ProfileAvatarLocations) {
		self.channels.set_neq(other.channels);
		self.console.set_neq(other.console);
		self.private_messages.set_neq(other.private_messages);
		self.profile_links.set_neq(other.profile_links);
		self.system_messages.set_neq(other.system_messages);
	}
}

impl From<common_settings::ProfileAvatarLocations> for ProfileAvatarLocations {
	fn from(remote: common_settings::ProfileAvatarLocations) -> Self {
		Self {
			channels: Mutable::new(remote.channels),
			console: Mutable::new(remote.console),
			private_messages: Mutable::new(remote.private_messages),
			profile_links: Mutable::new(remote.profile_links),
			system_messages: Mutable::new(remote.system_messages),
		}
	}
}

#[derive(Debug)]
pub struct ClientSettings {
	pub animate_eicons: Mutable<bool>,
	pub click_open_target: Mutable<common_settings::ClickOpenTarget>,
	pub exclude_tags: MutableVec<String>,
	pub inactivity_timer: InactivityTimer,
	pub system_messages: SystemMessages,
}

impl ClientSettings {
	/// Merge a configuration fragment into this one, overwriting values if
	/// required.
	pub fn merge(&self, other: common_settings::ClientSettings) {
		self.animate_eicons.set_neq(other.animate_eicons);
		self.click_open_target.set_neq(other.click_open_target);
		self.exclude_tags.lock_mut().replace_cloned(other.exclude_tags);
		self.inactivity_timer.merge(other.inactivity_timer);
		self.system_messages.merge(other.system_messages);
	}
}

impl From<common_settings::ClientSettings> for ClientSettings {
	fn from(remote: common_settings::ClientSettings) -> Self {
		Self {
			animate_eicons: Mutable::new(remote.animate_eicons),
			click_open_target: Mutable::new(remote.click_open_target),
			exclude_tags: MutableVec::new_with_values(remote.exclude_tags),
			inactivity_timer: remote.inactivity_timer.into(),
			system_messages: remote.system_messages.into(),
		}
	}
}

/// How long to wait before setting the user's status to idle.
#[derive(Debug)]
pub struct InactivityTimer {
	pub enabled: Mutable<bool>,
	pub timer: Mutable<f32>,
}

impl InactivityTimer {
	fn merge(&self, other: common_settings::InactivityTimer) {
		self.enabled.set_neq(other.enabled);
		self.timer.set_neq(other.timer);
	}
}

impl From<common_settings::InactivityTimer> for InactivityTimer {
	fn from(remote: common_settings::InactivityTimer) -> Self {
		Self {
			enabled: Mutable::new(remote.enabled),
			timer: Mutable::new(remote.timer),
		}
	}
}

/// Where to show messages from the F-Chat server.
#[derive(Debug)]
pub struct SystemMessages {
	pub enabled: Mutable<bool>,
	pub display_in_chats: Mutable<bool>,
	pub display_in_console: Mutable<bool>,
	pub notify: Mutable<bool>,
}

impl SystemMessages {
	fn merge(&self, other: common_settings::SystemMessages) {
		self.enabled.set_neq(other.enabled);
		self.display_in_chats.set_neq(other.display_in_chats);
		self.display_in_console.set_neq(other.display_in_console);
		self.notify.set_neq(other.notify);
	}
}

impl From<common_settings::SystemMessages> for SystemMessages {
	fn from(remote: common_settings::SystemMessages) -> Self {
		Self {
			enabled: Mutable::new(remote.enabled),
			display_in_chats: Mutable::new(remote.display_in_chats),
			display_in_console: Mutable::new(remote.display_in_console),
			notify: Mutable::new(remote.notify),
		}
	}
}

#[derive(Debug)]
pub struct KeyboardShortcuts {
	pub movement: MutableBTreeMap<common_settings::MovementShortcut, String>,
	pub text_format: MutableBTreeMap<common_settings::TextFormatShortcut, String>,
	pub use_custom_bindings: Mutable<bool>,
}

impl KeyboardShortcuts {
	/// Merge a configuration fragment into this one, overwriting values if
	/// required.
	pub fn merge(&self, other: common_settings::KeyboardShortcuts) {
		self.movement.lock_mut().replace_cloned(other.movement);
		self.text_format.lock_mut().replace_cloned(other.text_format);
		self.use_custom_bindings.set_neq(other.use_custom_bindings);
	}
}

impl From<common_settings::KeyboardShortcuts> for KeyboardShortcuts {
	fn from(remote: common_settings::KeyboardShortcuts) -> Self {
		Self {
			movement: MutableBTreeMap::with_values(remote.movement),
			text_format: MutableBTreeMap::with_values(remote.text_format),
			use_custom_bindings: Mutable::new(remote.use_custom_bindings),
		}
	}
}

#[derive(Debug)]
pub struct LoggerSettings {
	pub log_ads: Mutable<bool>,
	pub log_messages: Mutable<bool>,
	pub storage_method: Mutable<common_settings::LogStorageMethod>,
}

impl LoggerSettings {
	/// Merge a configuration fragment into this one, overwriting values if
	/// required.
	pub fn merge(&self, other: common_settings::LoggerSettings) {
		self.log_ads.set_neq(other.log_ads);
		self.log_messages.set_neq(other.log_messages);
		self.storage_method.set_neq(other.storage_method);
	}
}

impl From<common_settings::LoggerSettings> for LoggerSettings {
	fn from(remote: common_settings::LoggerSettings) -> Self {
		Self {
			log_ads: Mutable::new(remote.log_ads),
			log_messages: Mutable::new(remote.log_messages),
			storage_method: Mutable::new(remote.storage_method),
		}
	}
}

#[derive(Debug)]
pub struct NotificationSettings {
	pub in_app_notifications: InAppNotificationSettings,
	pub native_notifications: NativeNotificationSettings,
	pub notification_types: NotificationTypes,
	pub word_list: MutableVec<String>,
}

impl NotificationSettings {
	/// Merge a configuration fragment into this one, overwriting values if
	/// required.
	pub fn merge(&self, other: common_settings::NotificationSettings) {
		self.in_app_notifications.merge(other.in_app_notifications);
		self.native_notifications.merge(other.native_notifications);
		self.notification_types.merge(other.notification_types);
		self.word_list.lock_mut().replace_cloned(other.word_list);
	}
}

impl From<common_settings::NotificationSettings> for NotificationSettings {
	fn from(remote: common_settings::NotificationSettings) -> Self {
		Self {
			in_app_notifications: remote.in_app_notifications.into(),
			native_notifications: remote.native_notifications.into(),
			notification_types: remote.notification_types.into(),
			word_list: MutableVec::new_with_values(remote.word_list),
		}
	}
}

/// Settings for in-app notifications.
#[derive(Debug)]
pub struct InAppNotificationSettings {
	pub auto_dismiss: Mutable<bool>,
	pub dismissal_timer: Mutable<f32>,
	pub enabled: Mutable<bool>,
}

impl InAppNotificationSettings {
	fn merge(&self, other: common_settings::InAppNotificationSettings) {
		self.auto_dismiss.set_neq(other.auto_dismiss);
		self.dismissal_timer.set_neq(other.dismissal_timer);
		self.enabled.set_neq(other.enabled);
	}
}

impl From<common_settings::InAppNotificationSettings> for InAppNotificationSettings {
	fn from(remote: common_settings::InAppNotificationSettings) -> Self {
		Self {
			auto_dismiss: Mutable::new(remote.auto_dismiss),
			dismissal_timer: Mutable::new(remote.dismissal_timer),
			enabled: Mutable::new(remote.enabled),
		}
	}
}

/// Settings for native notifications.
#[derive(Debug)]
pub struct NativeNotificationSettings {
	pub enabled: Mutable<bool>,
}

impl NativeNotificationSettings {
	fn merge(&self, other: common_settings::NativeNotificationSettings) {
		self.enabled.set_neq(other.enabled);
	}
}

impl From<common_settings::NativeNotificationSettings> for NativeNotificationSettings {
	fn from(remote: common_settings::NativeNotificationSettings) -> Self {
		Self {
			enabled: Mutable::new(remote.enabled),
		}
	}
}

/// What notifications should be displayed to the user.
#[derive(Debug)]
pub struct NotificationTypes {
	pub announcements: Mutable<bool>,
	pub mentions: Mutable<bool>,
	pub private_messages: Mutable<bool>,
	pub word_list: Mutable<bool>,
}

impl NotificationTypes {
	fn merge(&self, other: common_settings::NotificationTypes) {
		self.announcements.set_neq(other.announcements);
		self.mentions.set_neq(other.mentions);
		self.private_messages.set_neq(other.private_messages);
		self.word_list.set_neq(other.word_list);
	}
}

impl From<common_settings::NotificationTypes> for NotificationTypes {
	fn from(remote: common_settings::NotificationTypes) -> Self {
		Self {
			announcements: Mutable::new(remote.announcements),
			mentions: Mutable::new(remote.mentions),
			private_messages: Mutable::new(remote.private_messages),
			word_list: Mutable::new(remote.word_list),
		}
	}
}
