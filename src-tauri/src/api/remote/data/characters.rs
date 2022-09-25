use crate::api::{Account, ApiClient};
use crate::api::error::{DeserializeError, Result as ApiResult};
use crate::api::remote::data::mock;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DefaultOnNull, DisplayFromStr};
use snowcat_common::characters::CharacterId;
use std::collections::BTreeMap;
use std::fmt;
use time::{OffsetDateTime, UtcOffset};

//------------------------------------------------------------------------------
// API CLIENT IMPLEMENTATION
//------------------------------------------------------------------------------

impl ApiClient {
	pub async fn get_character(&self, character: &str) -> ApiResult<GetChararacterResponse> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		GetChararacter::new(character)
			.use_account(account)
			.execute(self.http()).await
	}

	pub async fn list_characters(&self) -> ApiResult<Vec<String>> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		ListCharacters::new()
			.use_account(account)
			.execute(self.http()).await
			.map(|response| response.into_list())
	}
}

//------------------------------------------------------------------------------
// REQUEST
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct GetChararacter<'client, 'command, const A: bool> {
	name: &'command str,

	#[serde(flatten)]
	account: Option<&'client Account>
}

impl<'client, 'command> GetChararacter<'client, 'command, false> {
	pub fn new(character: &'command str) -> GetChararacter<'_, 'command, false> {
		GetChararacter {
			name: character,
			account: None,
		}
	}

	pub fn use_account(self, account: &'client Account) -> GetChararacter<'client, 'command, true> {
		GetChararacter {
			name: self.name,
			account: Some(account),
		}
	}
}

impl GetChararacter<'_, '_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<GetChararacterResponse> {
		DeserializeError::from_value(GetChararacterResponse::default()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct ListCharacters<'client, const A: bool> {
	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl ListCharacters<'_, false> {
	pub fn new() -> Self {
		ListCharacters {
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> ListCharacters<'client, true> {
		ListCharacters {
			account: Some(account),
		}
	}
}

impl ListCharacters<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<ListCharactersResponse> {
		DeserializeError::from_value(ListCharactersResponse::default()).into_result()
	}
}

//------------------------------------------------------------------------------
// RESPONSE
//------------------------------------------------------------------------------

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GetChararacterResponse {
	#[serde(deserialize_with = "string_to_character_id")]
	pub id: CharacterId,
	
	pub name: String,
	pub description: String,
	pub custom_title: String,
	pub is_self: bool,
	pub views: u64,
	pub badges: Vec<String>, // TODO: change to Vec<CharacterBadge>,
	pub images: Vec<GalleryImage>,

	// ignored
	// customs_first: bool,

	#[serde(rename = "timezone", deserialize_with = "hours_to_offset")]
	pub offset: UtcOffset,

	#[serde(rename = "character_list")]
	pub linked_characters: Vec<CharacterLink>,

	#[serde(rename = "settings")]
	pub page_settings: PageSettings,

	#[serde(rename = "current_user")]
	pub view_settings: ViewSettings,

	#[serde(deserialize_with = "time::serde::timestamp::deserialize")]
	pub created_at: OffsetDateTime,

	#[serde(deserialize_with = "time::serde::timestamp::deserialize")]
	pub updated_at: OffsetDateTime,

	#[serde_as(as = "Vec<DisplayFromStr>")]
	pub kinks: Vec<u64>,

	#[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
	pub inlines: BTreeMap<u64, InlineImage>,
}

impl Default for GetChararacterResponse {
	fn default() -> Self {
		mock::character_data()
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListCharactersResponse {
	characters: Vec<String>,
}

impl ListCharactersResponse {
	pub fn into_list(self) -> Vec<String> {
		self.characters
	}
}

impl Default for ListCharactersResponse {
	fn default() -> Self {
		ListCharactersResponse {
			characters: mock::character_names().map(ToString::to_string).to_vec(),
		}
	}
}

//------------------------------------------------------------------------------
// DATA
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterLink {
	pub id: u64,
	pub name: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct CustomKink {
	pub name: String,
	pub description: String,

	#[serde(rename = "choice")]
	pub column: KinkColumn,

	#[serde_as(as = "Vec<DisplayFromStr>")]
	#[serde(rename = "children")]
	pub subkinks: Vec<u8>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GalleryImage {
	#[serde_as(as = "DisplayFromStr")]
	#[serde(rename = "image_id")]
	pub id: u64,

	#[serde_as(as = "DefaultOnNull")]
	pub sort_order: Option<u64>,

	#[serde_as(as = "DisplayFromStr")]
	pub width: u32,

	#[serde_as(as = "DisplayFromStr")]
	pub height: u32,

	pub extension: String,
	pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InlineImage {
	pub hash: String,
	pub extension: String,
	pub nsfw: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KinkColumn {
	Fave,
	Maybe,
	No,
	Yes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PageSettings {
	pub customs_first: bool,
	pub prevent_bookmarks: bool,
	pub show_friends: bool,

	#[serde(rename = "public")]
	pub is_public: bool,
	
	#[serde(rename = "guestbook")]
	pub show_guestbook: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ViewSettings {
	pub animated_icons: bool,
	pub inline_mode: u64,
}

//------------------------------------------------------------------------------
// HELPER
//------------------------------------------------------------------------------

/// Deserialize an amount of hours as a UtcOffset
fn hours_to_offset<'de, D>(deserializer: D) -> Result<UtcOffset, D::Error>
where
	D: serde::Deserializer<'de>,
{
	use serde::de::{Error, Visitor};

	struct OffsetVisitor;
	impl<'de> Visitor<'de> for OffsetVisitor {
		type Value = UtcOffset;

		fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(f, "a signed integer -24 < i < 24")
		}

		fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E>
		where
			E: Error,
		{
			UtcOffset::from_hms(v, 0, 0).map_err(|err| Error::custom(err))
		}
	}

	deserializer.deserialize_i8(OffsetVisitor)
}

/// Deserialize a numeric string as a CharacterId
fn string_to_character_id<'de, D>(deserializer: D) -> Result<CharacterId, D::Error>
where
	D: serde::Deserializer<'de>,
{
	use serde::de::{Error, Visitor};

	struct IdVisitor;
	impl<'de> Visitor<'de> for IdVisitor {
		type Value = u64;

		fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(f, "a string representing an integer")
		}

		fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
		where
			E: Error,
		{
			v.parse().map_err(|_| {
				Error::invalid_type(serde::de::Unexpected::Other("non-integer string"), &"integer string")
			})
		}
	}

	deserializer.deserialize_str(IdVisitor).map(CharacterId::from)
}
