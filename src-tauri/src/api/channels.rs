use crate::api::characters::CharacterInfo;
use serde::{Deserialize, Serialize};
use std::borrow::{Cow, Borrow};
use std::fmt;

#[derive(Debug, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct ChannelId(pub String);

impl fmt::Display for ChannelId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct ChannelInfo {
	pub id: ChannelId,
	pub display_name: String,
	pub character_count: usize,

	pub is_official: bool,
}

impl PartialEq for ChannelInfo {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl From<String> for ChannelId {
	fn from(id: String) -> Self {
		ChannelId(id)
	}
}

impl<'a> From<&'a str> for ChannelId {
	fn from(id: &'a str) -> Self {
		ChannelId(id.to_owned())
	}
}

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct ChannelData {
	pub id: ChannelId,
	pub description: String,
	pub characters: Vec<String>,
	pub mode: ChannelMode,
}

impl PartialEq for ChannelData {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChannelInfoChange<'channel, 'character> {
	#[serde(borrow)]
	channel_id: Cow<'channel, ChannelId>,
	
	#[serde(borrow)]
	pub data: ChannelInfoChangeData<'character>,
}

impl<'channel, 'character> ChannelInfoChange<'channel, 'character> {
	pub(crate) fn new(channel: &'channel ChannelInfo, data: ChannelInfoChangeData<'character>) -> Self {
		ChannelInfoChange {
			channel_id: Cow::Borrowed(&channel.id),
			data,
		}
	}

	pub fn channel_id(&self) -> &ChannelId {
		self.channel_id.borrow()
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ChannelInfoChangeData<'character> {
	ChannelDeleted,
	ChannelDescriptionChanged { new_description: String },
	ChannelModeChanged { new_mode: ChannelMode },

	CharacterCountChanged { new_count: usize },
	CharacterJoined { character: &'character str },
	CharacterLeft { character: &'character str },
}

impl<'character> ChannelInfoChangeData<'character> {
	pub(crate) fn channel_deleted() -> Self {
		ChannelInfoChangeData::ChannelDeleted
	}

	pub(crate) fn channel_description_changed(new_description: String) -> Self {
		ChannelInfoChangeData::ChannelDescriptionChanged { new_description }
	}

	pub(crate) fn channel_mode_changed(new_mode: ChannelMode) -> Self {
		ChannelInfoChangeData::ChannelModeChanged { new_mode }
	}

	pub(crate) fn character_count_changed(new_count: usize) -> Self {
		ChannelInfoChangeData::CharacterCountChanged { new_count }
	}

	pub(crate) fn character_joined(character: &'character CharacterInfo) -> Self {
		ChannelInfoChangeData::CharacterJoined { character: &character.name }
	}

	pub(crate) fn character_left(character: &'character CharacterInfo) -> Self {
		ChannelInfoChangeData::CharacterLeft { character: &character.name }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMode {
	Chat,
	Ads,
	Both,
}
