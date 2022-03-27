// TODO: Consider moving this to the front end.

use crate::remote;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;
use std::time::{self, Duration};

#[derive(Debug, Clone, Eq, Deserialize, Serialize)]
pub struct Character {
	pub id: u64,
	pub name: String,
}

impl From<&remote::character::RemoteCharacter> for Character {
	fn from(remote: &remote::character::RemoteCharacter) -> Self {
		Character {
			id: remote.id,
			name: remote.name.to_owned(),
		}
	}
}

impl From<&remote::character::RemoteCharacterDefinition> for Character {
	fn from(remote: &remote::character::RemoteCharacterDefinition) -> Self {
		Character {
			id: remote.id,
			name: remote.name.to_owned(),
		}
	}
}

impl PartialEq<Self> for Character {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}

	fn ne(&self, other: &Self) -> bool {
		self.id != other.id
	}
}

#[derive(Debug, Clone)]
pub struct CharacterProfile {
	pub badges: Vec<Badge>,
	pub character_list: Vec<Character>,
	pub custom_kinks: HashMap<u64, CustomKink>,
	pub custom_title: Option<String>,
	pub description: String,
	pub images: Vec<Image>,
	pub is_self: bool,
	pub kinks: HashMap<u64, KinkCategory>,
	pub memo: Memo,
	pub settings: DisplaySettings,
	pub tags: Vec<Tag>,
	pub views: u64,

	pub created_at: DateTime<Local>,
	pub updated_at: DateTime<Local>,

	// Excluded in favour of Settings.show_customs_first, written for
	// completeness.
	// pub customs_first: bool,
}

impl From<&remote::character::RemoteCharacter> for CharacterProfile {
	fn from(remote: &remote::character::RemoteCharacter) -> Self {
		let created_at = time::UNIX_EPOCH + Duration::from_secs(remote.created_at.try_into().unwrap());
		let updated_at = time::UNIX_EPOCH + Duration::from_secs(remote.updated_at.try_into().unwrap());

		CharacterProfile {
			badges: remote.badges.iter().map(|badge| Badge::from_str(&badge).unwrap()).collect(),
			character_list: remote.character_list.iter().map(Character::from).collect(),
			custom_kinks: transform::collect_custom_kinks(&remote.custom_kinks),
			custom_title: remote.custom_title.clone(),
			description: remote.description.clone(),
			images: transform::collect_images(&remote.images),
			is_self: remote.is_self,
			kinks: transform::collect_kinks(&remote.kinks),
			memo: Memo::from(&remote.memo),
			settings: DisplaySettings::from(&remote.settings),
			tags: transform::collect_tags(&remote.infotags),
			views: remote.views,

			created_at: DateTime::from(created_at),
			updated_at: DateTime::from(updated_at),
		}
	}
}

pub enum CharacterStatus {
	Looking,
	Online,
	Away,
	Idle,
	Busy,
	DoNotDisturb,
	Offline,
}

/// A badge used to indicate special roles associated with a character.
#[derive(Debug, Clone, Copy)]
pub enum Badge {
	Administrator,
	Developer,
	Helpdesk,
	GlobalModerator,
	ChatOperator,
	ChannelOperator,
}

impl FromStr for Badge {
	type Err = ();

	fn from_str(string: &str) -> Result<Self, Self::Err> {
		match string {
			"admin" => Ok(Badge::Administrator),
			"developer" => Ok(Badge::Developer),
			"helpdesk" => Ok(Badge::Helpdesk),
			"global" => Ok(Badge::GlobalModerator),
			"chatop" => Ok(Badge::ChatOperator),
			"chanop" => Ok(Badge::ChannelOperator),
			_ => Err(())
		}
	}
}

impl fmt::Display for Badge {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Badge::Administrator => write!(f, "Administrator"),
			Badge::Developer => write!(f, "Developer"),
			Badge::Helpdesk => write!(f, "Helpdesk Operator"),
			Badge::GlobalModerator => write!(f, "Global Moderator"),
			Badge::ChatOperator => write!(f, "Chat Operator"),
			Badge::ChannelOperator => write!(f, "Channel Operator"),
		}
	}
}

/// A custom kink associated with the character.
#[derive(Debug, Clone)]
pub struct CustomKink {
	pub children: Vec<u64>,
	pub choice: KinkCategory,
	pub description: String,
	pub name: String,
}

impl From<&remote::character::RemoteCustomKink> for CustomKink {
	fn from(remote: &remote::character::RemoteCustomKink) -> Self {
		CustomKink {
			children: remote.children.clone(),
			choice: KinkCategory::from_str(&remote.choice).unwrap(),
			description: remote.description.clone(),
			name: remote.name.clone(),
		}
	}
}

/// Display settings to be used when building this character's profile.
#[derive(Debug, Clone)]
pub struct DisplaySettings {
	pub enable_bookmarks: bool,
	pub enable_guestbook: bool,
	pub is_public: bool,
	pub show_customs_first: bool,
	pub show_friends: bool,
}

impl From<&remote::character::RemoteDisplaySettings> for DisplaySettings {
	fn from(remote: &remote::character::RemoteDisplaySettings) -> Self {
		DisplaySettings {
			enable_bookmarks: !remote.prevent_bookmarks,
			enable_guestbook: remote.guestbook,
			is_public: remote.public,
			show_customs_first: remote.customs_first,
			show_friends: remote.show_friends,
		}
	}
}

/// An image from the character's image gallery.
#[derive(Debug, Clone)]
pub struct Image {
	pub description: String,
	pub extension: String,
	pub height: u64,
	pub image_id: u64,
	pub sort_order: u64,
	pub width: u64,
}

impl From<&remote::character::RemoteImage> for Image {
	fn from(remote: &remote::character::RemoteImage) -> Self {
		Image {
			description: remote.description.clone(),
			extension: remote.extension.clone(),
			height: remote.height.parse().unwrap(),
			image_id: remote.image_id.parse().unwrap(),
			sort_order: remote.sort_order.parse().unwrap(),
			width: remote.width.parse().unwrap(),
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub enum KinkCategory {
	Favourite,
	Yes,
	Maybe,
	No,
}

impl FromStr for KinkCategory {
	type Err = ();

	fn from_str(string: &str) -> Result<Self, Self::Err> {
		match string {
			"fave" => Ok(KinkCategory::Favourite),
			"yes" => Ok(KinkCategory::Yes),
			"maybe" => Ok(KinkCategory::Maybe),
			"no" => Ok(KinkCategory::No),
			_ => Err(())
		}
	}
}

// The "id" field is not required and is only written for completeness reasons.
/// A memo attached to the character by the current user.
#[derive(Debug, Clone)]
pub struct Memo {
	// id: u64,
	pub memo: String,
}

impl From<&remote::character::RemoteMemo> for Memo {
	fn from(remote: &remote::character::RemoteMemo) -> Self {
		Memo {
			memo: remote.memo.to_owned(),
		}
	}
}

/// Information associated with this character.
#[derive(Debug, Clone)]
pub struct Tag {
	data: String,
	key: String,
}

impl From<(&String, &String)> for Tag {
	fn from((key, data): (&String, &String)) -> Self {
		Tag {
			data: data.to_owned(),
			key: key.to_owned(),
		}
	}
}

mod transform {
	use crate::remote;
	use crate::state::character::{CustomKink, Image, KinkCategory, Tag};
	use std::collections::HashMap;
	use std::str::FromStr;

	pub fn collect_custom_kinks(
		customs: &HashMap<String, remote::character::RemoteCustomKink>
	) -> HashMap<u64, CustomKink> {
		customs.iter()
			.map(|(id, kink)| {
				(id.parse().unwrap(), kink.into())
			})
			.fold(HashMap::new(), |mut map, (id, kink)| {
				map.insert(id, kink);
				map
			})
	}

	pub fn collect_images(images: &Vec<remote::character::RemoteImage>) -> Vec<Image> {
		images.iter().map(|image| image.into()).collect()
	}

	pub fn collect_kinks(kinks: &HashMap<String, String>) -> HashMap<u64, KinkCategory> {
		kinks.iter()
			.map(|(id, category)| {
				(id.parse().unwrap(), KinkCategory::from_str(category).unwrap())
			})
			.fold(HashMap::new(), |mut map, (id, category)| {
				map.insert(id, category);
				map
			})
	}

	pub fn collect_tags(tags: &HashMap<String, String>) -> Vec<Tag> {
		tags.iter().map(|tag| tag.into()).collect()
	}
}

mod mock {
	use crate::deserialize;
	use crate::error::{ApiError, Error};
	use crate::remote;
	use super::CharacterProfile;

	const ANTINOV: &'static str = include_str!("../../mock/antinov.json");
	const DRAGON_WOLF: &'static str = include_str!("../../mock/dragon wolf.json");
	const MARGARET_ROSE: &'static str = include_str!("../../mock/margaret rose.json");
	const TIGERGRIFFIN: &'static str = include_str!("../../mock/tigergriffin.json");

	pub fn get_profile(id: u64) -> Result<CharacterProfile, Error> {
		match id {
			1069478 => deserialize!(ANTINOV => remote::character::RemoteCharacter => CharacterProfile),
			1926571 => deserialize!(DRAGON_WOLF => remote::character::RemoteCharacter => CharacterProfile),
			2866422 => deserialize!(TIGERGRIFFIN => remote::character::RemoteCharacter => CharacterProfile),
			3710010 => deserialize!(MARGARET_ROSE => remote::character::RemoteCharacter => CharacterProfile),
			_ => Err(ApiError::CharacterNotFound.into()),
		}
	}
}
