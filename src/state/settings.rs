use futures_signals::signal::Mutable;
use futures_signals::signal_map::MutableBTreeMap;
use futures_signals::signal_vec::MutableVec;
use snowcat_common::state::settings as common_settings;
use levenshtein_diff::{Edit, distance, generate_edits};

#[derive(Debug)]
pub struct Settings {
	pub appearance: AppearanceSettings,
	pub client: ClientSettings,
	pub keyboard_shortcuts: KeyboardShortcuts,
	pub logs: LoggerSettings,
	pub notifications: NotificationSettings,
}

impl Settings {
	/// Construct a list of differences between this configuration fragment and
	/// another one.
	pub fn diff(&self, other: &Self) -> Vec<common_settings::SettingsUpdate> {
		use common_settings::SettingsUpdate;

		let mut diff = Vec::new();

		diff.extend(self.appearance.diff(&other.appearance).into_iter().map(|value| {
			SettingsUpdate::AppearanceSettingsUpdate(value)
		}));

		diff.extend(self.client.diff(&other.client).into_iter().map(|value| {
			SettingsUpdate::ClientSettingsUpdate(value)
		}));

		diff.extend(self.keyboard_shortcuts.diff(&other.keyboard_shortcuts).into_iter().map(|value| {
			SettingsUpdate::KeyboardShortcutUpdate(value)
		}));

		diff.extend(self.logs.diff(&other.logs).into_iter().map(|value| {
			SettingsUpdate::LoggerSettingsUpdate(value)
		}));

		diff.extend(self.notifications.diff(&other.notifications).into_iter().map(|value| {
			SettingsUpdate::NotificationSettingsUpdate(value)
		}));

		diff
	}

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
	/// Construct a list of differences between this configuration fragment and
	/// another one.
	pub fn diff(&self, other: &Self) -> Vec<common_settings::AppearanceSettingsUpdate> {
		use common_settings::AppearanceSettingsUpdate;

		let mut diff = Vec::new();

		let clock_format = other.clock_format.get_cloned();
		if self.clock_format.get_cloned() != clock_format {
			diff.push(AppearanceSettingsUpdate::SetClockFormat(clock_format));
		}

		let display_size = other.display_size.get_cloned();
		if self.display_size.get_cloned() != display_size {
			diff.push(AppearanceSettingsUpdate::SetDisplaySize(display_size));
		}

		let show_avatars_in_channels = other.show_avatars_in.channels.get();
		if self.show_avatars_in.channels.get() != show_avatars_in_channels {
			diff.push(AppearanceSettingsUpdate::SetAvatarsInChannels(show_avatars_in_channels));
		}

		let show_avatars_in_console = other.show_avatars_in.console.get();
		if self.show_avatars_in.console.get() != show_avatars_in_console {
			diff.push(AppearanceSettingsUpdate::SetAvatarsInConsole(show_avatars_in_console));
		}

		let show_avatars_in_private_messages = other.show_avatars_in.private_messages.get();
		if self.show_avatars_in.private_messages.get() != show_avatars_in_private_messages {
			diff.push(AppearanceSettingsUpdate::SetAvatarsInPrivateMessages(show_avatars_in_private_messages));
		}

		let show_avatars_in_profile_links = other.show_avatars_in.profile_links.get();
		if self.show_avatars_in.profile_links.get() != show_avatars_in_profile_links {
			diff.push(AppearanceSettingsUpdate::SetAvatarsInProfileLinks(show_avatars_in_profile_links));
		}

		let show_avatars_in_system_messages = other.show_avatars_in.system_messages.get();
		if self.show_avatars_in.system_messages.get() != show_avatars_in_system_messages {
			diff.push(AppearanceSettingsUpdate::SetAvatarsInSystemMessages(show_avatars_in_system_messages));
		}

		let theme = other.theme.get_cloned();
		if self.theme.get_cloned() != theme {
			diff.push(AppearanceSettingsUpdate::SetColorScheme(theme));
		}

		let window_appearance = other.window_appearance.get_cloned();
		if self.window_appearance.get_cloned() != window_appearance {
			diff.push(AppearanceSettingsUpdate::SetWindowAppearance(window_appearance));
		}

		diff
	}

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
	/// Construct a list of differences between this configuration fragment and
	/// another one.
	pub fn diff(&self, other: &Self) -> Vec<common_settings::ClientSettingsUpdate> {
		use common_settings::ClientSettingsUpdate;

		let mut diff = Vec::new();

		let animate_eicons = other.animate_eicons.get();
		if self.animate_eicons.get() != animate_eicons {
			diff.push(ClientSettingsUpdate::SetAnimateEIcons(animate_eicons));
		}

		let click_open_target = other.click_open_target.get_cloned();
		if self.click_open_target.get_cloned() != click_open_target {
			diff.push(ClientSettingsUpdate::SetClickOpenTarget(click_open_target));
		}

		{
			let own_excluded_tags = self.exclude_tags.lock_ref();
			let other_excluded_tags = other.exclude_tags.lock_ref();

			let (distance, matrix) = distance(&own_excluded_tags, &other_excluded_tags);
			log::debug!("word_list collections have levenshtein distance of {distance}");

			let edits = generate_edits(&own_excluded_tags, &other_excluded_tags, &matrix).unwrap_or_else(|_| {
				unreachable!("we just generated these distances")
			});

			diff.extend(edits.into_iter().map(|edit| match edit {
				Edit::Delete(position) => ClientSettingsUpdate::RemoveExcludedTagsEntry(own_excluded_tags[position].to_owned()),
				Edit::Insert(_, value) => ClientSettingsUpdate::AddExcludedTagsEntry(value),
				Edit::Substitute(position, new) => ClientSettingsUpdate::AlterExcludedTagsEntry {
					old: own_excluded_tags[position].to_owned(), new
				},
			}));
		}

		let inactivity_timer_enabled = other.inactivity_timer.enabled.get();
		if self.inactivity_timer.enabled.get() != inactivity_timer_enabled {
			diff.push(ClientSettingsUpdate::SetInactivityTimerEnabled(inactivity_timer_enabled));
		}

		let inactivity_timer = other.inactivity_timer.timer.get();
		if self.inactivity_timer.timer.get() != inactivity_timer {
			diff.push(ClientSettingsUpdate::SetInactivityTimer(inactivity_timer));
		}

		let system_messages_enabled = other.system_messages.enabled.get();
		if self.system_messages.enabled.get() != system_messages_enabled {
			diff.push(ClientSettingsUpdate::SetSystemMessagesEnabled(system_messages_enabled));
		}

		let sm_display_in_chats = other.system_messages.display_in_chats.get();
		if self.system_messages.display_in_chats.get() != sm_display_in_chats {
			diff.push(ClientSettingsUpdate::SetDisplaySystemMessagesInActiveChannel(sm_display_in_chats));
		}

		let sm_display_in_console = other.system_messages.display_in_console.get();
		if self.system_messages.display_in_console.get() != sm_display_in_console {
			diff.push(ClientSettingsUpdate::SetDisplaySystemMessagesInConsole(sm_display_in_console));
		}

		let sm_notify = other.system_messages.notify.get();
		if self.system_messages.notify.get() != sm_notify {
			diff.push(ClientSettingsUpdate::SetSystemMessageNotifications(sm_notify));
		}

		diff
	}

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
	/// Construct a list of differences between this configuration fragment and
	/// another one.
	pub fn diff(&self, other: &Self) -> Vec<common_settings::KeyboardShortcutUpdate> {
		use common_settings::KeyboardShortcutUpdate;

		let mut diff = Vec::new();

		{
			let own_movement = self.movement.lock_ref();
			let other_movement = other.movement.lock_ref();

			own_movement.keys()
				.map(|key| (
					key.clone(),
					own_movement[key].to_owned(),
					other_movement[key].to_owned(),
				))
				.for_each(|item| if item.1 != item.2 {
					diff.push(KeyboardShortcutUpdate::SetMovementShortcut(item.0, item.2))
				});
		}

		{
			let own_text_format = self.text_format.lock_ref();
			let other_text_format = other.text_format.lock_ref();

			own_text_format.keys()
				.map(|key| (
					key.clone(),
					own_text_format[key].to_owned(),
					other_text_format[key].to_owned(),
				))
				.for_each(|item| if item.1 != item.2 {
					diff.push(KeyboardShortcutUpdate::SetTextFormatShortcut(item.0, item.2))
				});
		}

		let use_custom_keybinds = other.use_custom_bindings.get();
		if self.use_custom_bindings.get() != use_custom_keybinds {
			diff.push(KeyboardShortcutUpdate::SetCustomBindings(use_custom_keybinds));
		}

		diff
	}
	
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
	/// Construct a list of differences between this configuration fragment and
	/// another one.
	pub fn diff(&self, other: &Self) -> Vec<common_settings::LoggerSettingsUpdate> {
		use common_settings::LoggerSettingsUpdate;

		let mut diff = Vec::new();

		let log_ads = other.log_ads.get();
		if self.log_ads.get() != log_ads {
			diff.push(LoggerSettingsUpdate::SetLogAds(log_ads));
		}

		let log_messages = other.log_messages.get();
		if self.log_messages.get() != log_messages {
			diff.push(LoggerSettingsUpdate::SetLogMessages(log_messages));
		}

		diff
	}

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
	/// Construct a list of differences between this configuration fragment and
	/// another one.
	pub fn diff(&self, other: &Self) -> Vec<common_settings::NotificationSettingsUpdate> {
		use common_settings::NotificationSettingsUpdate;

		let mut diff = Vec::new();

		let ian_auto_dismiss = other.in_app_notifications.auto_dismiss.get();
		if self.in_app_notifications.auto_dismiss.get() != ian_auto_dismiss {
			diff.push(NotificationSettingsUpdate::SetInAppNotificationAutoDismiss(ian_auto_dismiss));
		}

		let ian_dismissal_timer = other.in_app_notifications.dismissal_timer.get();
		if self.in_app_notifications.dismissal_timer.get() != ian_dismissal_timer {
			diff.push(NotificationSettingsUpdate::SetInAppNotificationDismissalTimer(ian_dismissal_timer));
		}

		let ian_enabled = other.in_app_notifications.enabled.get();
		if self.in_app_notifications.enabled.get() != ian_enabled {
			diff.push(NotificationSettingsUpdate::SetInAppNotifications(ian_enabled));
		}

		let native_enabled = other.native_notifications.enabled.get();
		if self.native_notifications.enabled.get() != native_enabled {
			diff.push(NotificationSettingsUpdate::SetNativeNotifications(native_enabled));
		}

		let notify_announcements = other.notification_types.announcements.get();
		if self.notification_types.announcements.get() != notify_announcements {
			diff.push(NotificationSettingsUpdate::SetNotifyAnnouncements(notify_announcements));
		}

		let notify_mentions = other.notification_types.mentions.get();
		if self.notification_types.mentions.get() != notify_mentions {
			diff.push(NotificationSettingsUpdate::SetNotifyMentions(notify_mentions));
		}

		let notify_private_messages = other.notification_types.private_messages.get();
		if self.notification_types.private_messages.get() != notify_private_messages {
			diff.push(NotificationSettingsUpdate::SetNotifyPrivateMessages(notify_private_messages));
		}

		let notify_word_list = other.notification_types.word_list.get();
		if self.notification_types.word_list.get() != notify_word_list {
			diff.push(NotificationSettingsUpdate::SetNotifyWordList(notify_word_list));
		}

		{
			let own_word_list = self.word_list.lock_ref();
			let other_word_list = other.word_list.lock_ref();

			let (distance, matrix) = distance(&own_word_list, &other_word_list);
			log::debug!("word_list collections have levenshtein distance of {distance}");

			let edits = generate_edits(&own_word_list, &other_word_list, &matrix).unwrap_or_else(|_| {
				unreachable!("we just generated these distances")
			});

			diff.extend(edits.into_iter().map(|edit| match edit {
				Edit::Delete(position) => NotificationSettingsUpdate::RemoveWordListEntry(own_word_list[position].to_owned()),
				Edit::Insert(_, value) => NotificationSettingsUpdate::AddWordListEntry(value),
				Edit::Substitute(position, new) => NotificationSettingsUpdate::AlterWordListEntry {
					old: own_word_list[position].to_owned(), new
				},
			}));
		}

		diff
	}

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
