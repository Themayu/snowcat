use crate::api::remote::data::characters::GetChararacterResponse;
use crate::api::remote::data::lists::{
	GetInfoListResponse,
	GetKinkListResponse,
	GetMappingListResponse,
};
use snowcat_common::characters::CharacterId;

macro_rules! include_mock {
	($file_name: literal) => {
		include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/mock_data/", $file_name, ".json"))
	}
}

const BOOKMARKS: [&'static str; 8] = [
	"Dimentio",
	"Element Four",
	"Feedback",
	"Korban",
	"Scinner",
	"Shweetz",
	"Vandel",
	"Vile",
];

/// A pre-existing set of owned characters for testing purposes.
const CHARACTERS: [(CharacterId, &'static str); 5] = [
	(CharacterId(2543), "Marabel Thorne"),
	(CharacterId(191498), "Markelio"),
	(CharacterId(273593), "Phoney Baloney"),
	(CharacterId(327067), "Sarah Blitz Garrison"),
	(CharacterId(68851), "Yanozo Serna"),
];

/// A pre-existing set of friend bindings for testing purposes.
const FRIENDS: [(CharacterId, &'static [&'static str]); 5] = [
	(CharacterId(2543), &["Andrew Kane", "Anthony", "Corny Corn", "Lilia Norse", "Parrot Clara"]), // Marabel Thorne: 5
	(CharacterId(191498), &["Parrot Clara"]), // Markelio: 1
	(CharacterId(273593), &["Andrew Kane", "Corny Corn", "Lilia Norse", "Parrot Clara"]), // Phoney Baloney: 4
	(CharacterId(327067), &["Andrew Kane", "Anthony", "Corny Corn", "Parrot Clara"]), // Sara Blitz Garrison: 4
	(CharacterId(68851), &["Andrew Kane", "Corny Corn", "Lilia Norse"]), // Yanozo Serna: 3
];

/// The default character to use when testing (0-indexed).
const DEFAULT_CHARACTER: usize = 3;

const INFO_LIST: &str = include_mock!("info_list");
const KINK_LIST: &str = include_mock!("kink_list");
const MAPPING_LIST: &str = include_mock!("mapping_list");

const CHARACTER_DATA: &str = include_mock!("character_data");

/// The default character to use when testing.
pub fn default_character_name() -> &'static str {
	CHARACTERS[DEFAULT_CHARACTER].1
}

pub fn default_character_id() -> CharacterId {
	CHARACTERS[DEFAULT_CHARACTER].0
}

pub fn bookmarks() -> [&'static str; 8] {
	BOOKMARKS
}

pub fn characters() -> [(CharacterId, &'static str); 5] {
	CHARACTERS
}

pub fn character_names() -> [&'static str; 5] {
	CHARACTERS.map(|(_, name)| name)
}

pub fn friends() -> [(CharacterId, &'static [&'static str]); 5] {
	FRIENDS
}

pub fn info_list() -> GetInfoListResponse {
	serde_json::from_str(INFO_LIST).unwrap()
}

pub fn kink_list() -> GetKinkListResponse {
	serde_json::from_str(KINK_LIST).unwrap()
}

pub fn mapping_list() -> GetMappingListResponse {
	serde_json::from_str(MAPPING_LIST).unwrap()
}

pub fn character_data() -> GetChararacterResponse {
	serde_json::from_str(CHARACTER_DATA).unwrap()
}
