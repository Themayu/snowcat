use crate::api::error::Result as ApiResult;
use crate::api::remote::constants;
use crate::state::Cache;
use serde::{Deserialize, Serialize};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;
use tracing::{instrument, warn};
use std::{cmp, str, fmt};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;

type ProfileFuture = Pin<Box<dyn Future<Output = ApiResult<CharacterProfile>>>>;
pub type ProfileCache = Cache<CharacterProfile, Box<dyn FnMut() -> ProfileFuture>>;

#[derive(Debug, Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd, Deserialize, Serialize)]
pub struct CharacterId(pub u64);

impl CharacterId {
	pub const UNKNOWN: Self = CharacterId(0);
}

impl fmt::Display for CharacterId {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl From<u64> for CharacterId {
	fn from(number: u64) -> Self {
		CharacterId(number)
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CharacterInfo {
	pub name: String,
	pub gender: CharacterGender,
	pub status: CharacterStatus,
}

impl CharacterInfo {
	pub fn default_for(name: &str) -> Self {
		CharacterInfo {
			name: name.to_owned(),
			gender: CharacterGender::Unknown,
			status: CharacterStatus::new(CharacterStatusKind::Unknown),
		}
	}
}

impl PartialEq for CharacterInfo {
	fn eq(&self, other: &Self) -> bool {
		self.name == other.name
	}
}

impl Eq for CharacterInfo {}

impl PartialOrd for CharacterInfo {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.name.partial_cmp(&other.name)
	}
}

impl Ord for CharacterInfo {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct CharacterInfoChange {
	pub character: String,
	pub id: Option<CharacterId>,
	pub data: CharacterInfoChangeData,
}

impl CharacterInfoChange {
	pub(crate) fn new(character: String, data: CharacterInfoChangeData) -> Self {
		CharacterInfoChange {
			character,
			data,

			id: None,
		}
	}

	pub(crate) fn with_id(self, id: CharacterId) -> Self {
		CharacterInfoChange { id: Some(id), ..self }
	}

	pub fn character(&self) -> CharacterIdentifier<'_> {
		let CharacterInfoChange { character, id, .. } = self;

		match id {
			Some(id) => CharacterIdentifier::Id(*id),
			None => CharacterIdentifier::Name(&character),
		}
	}

	pub fn data(&self) -> &CharacterInfoChangeData {
		&self.data
	}
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum CharacterInfoChangeData {
	NameChanged { new_name: String },
	GenderChanged { new_gender: CharacterGender },
	StatusChanged { new_status: CharacterStatus },
}

impl CharacterInfoChangeData {
	pub(crate) fn name_changed(new_name: String) -> Self {
		CharacterInfoChangeData::NameChanged { new_name }
	}

	pub(crate) fn gender_changed(new_gender: CharacterGender) -> Self {
		CharacterInfoChangeData::GenderChanged { new_gender }
	}

	pub(crate) fn status_changed(new_status: CharacterStatus) -> Self {
		CharacterInfoChangeData::StatusChanged { new_status }
	}
}

pub enum CharacterIdentifier<'info> {
	Id(CharacterId),
	Name(&'info str)
}

#[derive(Debug, Clone)]
pub struct CharacterProfile {

}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, DeserializeFromStr, SerializeDisplay)]
pub enum CharacterGender {
	// Binary genders
	Female,
	Male,

	// Intersex genders
	CuntBoy,
	Hermaphrodite,
	MaleHerm,
	Shemale,

	// Other genders
	Transgender,
	#[default] None,

	// Unknown gender
	Unknown
}

impl fmt::Display for CharacterGender {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use constants::character_info::gender::*;

		write!(f, "{}", match self {
			// Binary genders
			Self::Male => GENDER_MALE,
			Self::Female => GENDER_FEMALE,

			// Intersex genders
			Self::CuntBoy => GENDER_CUNT_BOY,
			Self::Hermaphrodite => GENDER_HERM,
			Self::MaleHerm => GENDER_MALE_HERM,
			Self::Shemale => GENDER_SHEMALE,

			// Other genders
			Self::Transgender => GENDER_TRANS,
			Self::None => GENDER_NONE,

			// Unknown gender
			Self::Unknown => GENDER_NONE,
		})
	}
}

impl str::FromStr for CharacterGender {
	type Err = CharacterGenderUnknownValue;

	#[instrument(name = "Parsing a character gender")]
	fn from_str(gender: &str) -> Result<Self, Self::Err> {
		use constants::character_info::gender::*;

		let variant = match gender {
			// Binary genders
			GENDER_MALE => Self::Male,
			GENDER_FEMALE => Self::Female,

			// Intersex genders
			GENDER_CUNT_BOY => Self::CuntBoy,
			GENDER_HERM => Self::Hermaphrodite,
			GENDER_MALE_HERM => Self::MaleHerm,
			GENDER_SHEMALE => Self::Shemale,

			// Other genders
			GENDER_TRANS => Self::Transgender,
			GENDER_NONE => Self::None,

			// Unknown item
			value => {
				warn!("received unknown gender value {value:?} from server");
				return Err(CharacterGenderUnknownValue(String::from(value)));
			},
		};

		Ok(variant)
	}
}

#[derive(Debug, Clone, Error)]
#[error("Unknown character gender value: {0:?}")]
pub struct CharacterGenderUnknownValue(String);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, DeserializeFromStr, SerializeDisplay)]
pub enum CharacterLanguagePreference {
	Arabic,
	Chinese,
	Dutch,
	English,
	French,
	German,
	Italian,
	Japanese,
	Korean,
	Portuguese,
	Russian,
	Spanish,
	Swedish,
	#[default] Other,
}

impl fmt::Display for CharacterLanguagePreference {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use constants::character_info::language::*;

		write!(f, "{}", match self {
			Self::Arabic => LANGUAGE_ARABIC,
			Self::Chinese => LANGUAGE_CHINESE,
			Self::Dutch => LANGUAGE_DUTCH,
			Self::English => LANGUAGE_ENGLISH,
			Self::French => LANGUAGE_FRENCH,
			Self::German => LANGUAGE_GERMAN,
			Self::Italian => LANGUAGE_ITALIAN,
			Self::Japanese => LANGUAGE_JAPANESE,
			Self::Korean => LANGUAGE_KOREAN,
			Self::Portuguese => LANGUAGE_PORTUGUESE,
			Self::Russian => LANGUAGE_RUSSIAN,
			Self::Spanish => LANGUAGE_SPANISH,
			Self::Swedish => LANGUAGE_SWEDISH,
			Self::Other => LANGUAGE_OTHER,
		})
	}
}

impl str::FromStr for CharacterLanguagePreference {
	type Err = Infallible;

	#[instrument(name = "Parsing a character language preference")]
	fn from_str(preference: &str) -> Result<Self, Self::Err> {
		use constants::character_info::language::*;

		let variant = match preference {
			LANGUAGE_ARABIC => Self::Arabic,
			LANGUAGE_CHINESE => Self::Chinese,
			LANGUAGE_DUTCH => Self::Dutch,
			LANGUAGE_ENGLISH => Self::English,
			LANGUAGE_FRENCH => Self::French,
			LANGUAGE_GERMAN => Self::German,
			LANGUAGE_ITALIAN => Self::Italian,
			LANGUAGE_JAPANESE => Self::Japanese,
			LANGUAGE_KOREAN => Self::Korean,
			LANGUAGE_PORTUGUESE => Self::Portuguese,
			LANGUAGE_RUSSIAN => Self::Russian,
			LANGUAGE_SPANISH => Self::Spanish,
			LANGUAGE_SWEDISH => Self::Swedish,
			LANGUAGE_OTHER => Self::Other,

			value => {
				warn!("received unknown language preference value {value:?} from server");
				return Ok(Self::Other);
			}
		};

		Ok(variant)
	}
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, DeserializeFromStr, SerializeDisplay)]
pub enum CharacterOrientation {
	Asexual,
	BiCurious,
	BiFemalePreference,
	BiMalePreference,
	Bisexual,
	Gay,
	Pansexual,
	Straight,
	#[default] Unsure,
}

impl fmt::Display for CharacterOrientation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use constants::character_info::orientation::*;

		write!(f, "{}", match self {
			Self::Asexual => ORIENTATION_ASEXUAL,
			Self::BiCurious => ORIENTATION_BI_CURIOUS,
			Self::BiFemalePreference => ORIENTATION_BI_FEMALE_PREFERENCE,
			Self::BiMalePreference => ORIENTATION_BI_MALE_PREFERENCE,
			Self::Bisexual => ORIENTATION_BISEXUAL,
			Self::Gay => ORIENTATION_GAY,
			Self::Pansexual => ORIENTATION_PANSEXUAL,
			Self::Straight => ORIENTATION_STRAIGHT,
			Self::Unsure => ORIENTATION_UNSURE,
		})
    }
}

impl str::FromStr for CharacterOrientation {
	type Err = CharacterOrientationUnknownValue;

	#[instrument(name = "Parsing a character orientation")]
	fn from_str(orientation: &str) -> Result<Self, Self::Err> {
        use constants::character_info::orientation::*;

		let variant = match orientation {
			ORIENTATION_ASEXUAL => Self::Asexual,
			ORIENTATION_BI_CURIOUS => Self::BiCurious,
			ORIENTATION_BI_FEMALE_PREFERENCE => Self::BiFemalePreference,
			ORIENTATION_BI_MALE_PREFERENCE => Self::BiMalePreference,
			ORIENTATION_BISEXUAL => Self::Bisexual,
			ORIENTATION_GAY => Self::Gay,
			ORIENTATION_PANSEXUAL => Self::Pansexual,
			ORIENTATION_STRAIGHT => Self::Straight,
			ORIENTATION_UNSURE => Self::Unsure,

			value => {
				warn!("received unknown orientation value {value:?} from server");
				return Err(CharacterOrientationUnknownValue(String::from(value)));
			}
		};

		Ok(variant)
	}
}

#[derive(Debug, Clone, Error)]
#[error("Unknown character orientation value: {0:?}")]
pub struct CharacterOrientationUnknownValue(String);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, DeserializeFromStr, SerializeDisplay)]
pub enum CharacterPreference {
	HumansOnly,
	HumansPreferred,
	#[default] NoPreference,
	FurriesPreferred,
	FurriesOnly,
}

impl fmt::Display for CharacterPreference {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use constants::character_info::furrypref::*;

		write!(f, "{}", match self {
			Self::HumansOnly => FURRYPREF_HUMANS_ONLY,
			Self::HumansPreferred => FURRYPREF_HUMANS_PREFERRED,
			Self::NoPreference => FURRYPREF_NONE,
			Self::FurriesPreferred => FURRYPREF_FURRIES_PREFERRED,
			Self::FurriesOnly => FURRYPREF_FURRIES_ONLY,
		})
	}
}

impl str::FromStr for CharacterPreference {
	type Err = CharacterPreferenceUnknownValue;

	#[instrument(name = "Parsing a character preference")]
	fn from_str(preference: &str) -> Result<Self, Self::Err> {
		use constants::character_info::furrypref::*;

		let variant = match preference {
			FURRYPREF_HUMANS_ONLY => Self::HumansOnly,
			FURRYPREF_HUMANS_PREFERRED => Self::HumansPreferred,
			FURRYPREF_NONE => Self::NoPreference,
			FURRYPREF_FURRIES_PREFERRED => Self::FurriesPreferred,
			FURRYPREF_FURRIES_ONLY => Self::FurriesOnly,

			value => {
				warn!("received unknown preference value {value:?} from server");
				return Err(CharacterPreferenceUnknownValue(String::from(value)));
			}
		};

		Ok(variant)
	}
}

#[derive(Debug, Clone, Error)]
#[error("Unknown character preference value: {0:?}")]
pub struct CharacterPreferenceUnknownValue(String);

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, DeserializeFromStr, SerializeDisplay)]
pub enum CharacterRole {
	DomOnly,
	DomPreference,
	#[default] Switch,
	SubPreference,
	SubOnly,
}

impl fmt::Display for CharacterRole {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		use constants::character_info::role::*;

		write!(f, "{}", match self {
			Self::DomOnly => ROLE_DOM_ONLY,
			Self::DomPreference => ROLE_DOM_PREFERENCE,
			Self::Switch => ROLE_SWITCH,
			Self::SubPreference => ROLE_SUB_PREFERENCE,
			Self::SubOnly => ROLE_SUB_ONLY,
		})
	}
}

impl str::FromStr for CharacterRole {
	type Err = CharacterRoleUnknownValue;

	#[instrument(name = "Parsing a character role")]
	fn from_str(role: &str) -> Result<Self, Self::Err> {
		use constants::character_info::role::*;

		let variant = match role {
			ROLE_DOM_ONLY => Self::DomOnly,
			ROLE_DOM_PREFERENCE => Self::DomPreference,
			ROLE_SWITCH => Self::Switch,
			ROLE_SUB_PREFERENCE => Self::SubPreference,
			ROLE_SUB_ONLY => Self::SubOnly,

			value => {
				warn!("received unknown role value {value:?} from server");
				return Err(CharacterRoleUnknownValue(String::from(value)));
			}
		};

		Ok(variant)
	}
}

#[derive(Debug, Clone, Error)]
#[error("Unknown character role value: {0:?}")]
pub struct CharacterRoleUnknownValue(String);

#[derive(Debug, Clone, Default, Eq, PartialEq, Deserialize, Serialize)]
pub struct CharacterStatus {
	pub message: Option<String>,
	pub kind: CharacterStatusKind,
}

impl CharacterStatus {
	pub fn new(status: CharacterStatusKind) -> Self {
		CharacterStatus {
			message: None,
			kind: status,
		}
	}

	pub fn new_with_message(message: &str, status: CharacterStatusKind) -> Self {
		CharacterStatus {
			message: Some(message.to_owned()),
			kind: status,
		}
	}
}

impl str::FromStr for CharacterStatus {
	type Err = Infallible;

	#[instrument(name = "Parsing a character status")]
	fn from_str(status: &str) -> Result<Self, Self::Err> {
		use constants::character_info::status::*;

		let kind = match status {
			STATUS_CROWN => CharacterStatusKind::Crown,
			STATUS_ONLINE => CharacterStatusKind::Online,
			STATUS_LOOKING => CharacterStatusKind::Looking,
			STATUS_IDLE => CharacterStatusKind::Idle,
			STATUS_AWAY => CharacterStatusKind::Away,
			STATUS_BUSY => CharacterStatusKind::Busy,
			STATUS_DO_NOT_DISTURB => CharacterStatusKind::DoNotDisturb,

			value => {
				let default = CharacterStatusKind::default();
				warn!(?default, "received unknown status value {value:?} from server, returning default");

				default
			}
		};

		Ok(CharacterStatus {
			message: None,
			kind,
		})
	}
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CharacterStatusKind {
	#[default] Online,
	Crown,
	Looking,
	Idle,
	Away,
	Busy,

	#[serde(rename = "dnd")]
	DoNotDisturb,

	Unknown,
	Offline,
}
