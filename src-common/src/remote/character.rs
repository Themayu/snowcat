use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct RemoteCharacter {
	pub badges: Vec<String>,
	pub character_list: Vec<RemoteCharacterDefinition>,
	pub created_at: i64,
	pub custom_kinks: HashMap<String, RemoteCustomKink>,
	pub custom_title: Option<String>,
	pub customs_first: bool,
	pub description: String,
	pub id: u64,
	pub images: Vec<RemoteImage>,
	pub infotags: HashMap<String, String>,
	pub inlines: Vec<RemoteInline>,
	pub is_self: bool,
	pub kinks: HashMap<String, String>,
	pub memo: RemoteMemo,
	pub name: String,
	pub settings: RemoteDisplaySettings,
	pub updated_at: i64,
	pub views: u64,
}

#[derive(Debug, Deserialize)]
pub struct RemoteCharacterDefinition {
	pub id: u64,
	pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoteCustomKink {
	pub children: Vec<u64>,
	pub choice: String,
	pub description: String,
	pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoteDisplaySettings {
	pub customs_first: bool,
	pub guestbook: bool,
	pub prevent_bookmarks: bool,
	pub public: bool,
	pub show_friends: bool,
}

#[derive(Debug, Deserialize)]
pub struct RemoteImage {
	pub description: String,
	pub extension: String,
	pub height: String,
	pub image_id: String,
	pub sort_order: String,
	pub width: String,
}

#[derive(Debug, Deserialize)]
pub struct RemoteInline {
	pub extension: String,
	pub hash: String,
	pub nsfw: bool,
}

#[derive(Debug, Deserialize)]
pub struct RemoteMemo {
	pub id: u64,
	pub memo: String,
}
