use crate::api::Ticket;
use crate::api::error::{Result as ApiResult, DeserializeError};
use crate::api::remote::data::mock;
use reqwest::Client as HttpClient;
use serde::{Serialize, Deserialize};
use serde::ser::SerializeStruct;
use serde::de::DeserializeOwned;
use serde_with::{serde_as, DisplayFromStr};
use snowcat_common::characters::CharacterId;
use std::collections::HashMap;
use std::marker::PhantomData;

type OldCL = Vec<OldDC>;
type NewCL = HashMap<String, NewDC>;

type OldDC = String;
type NewDC = CharacterId;

//------------------------------------------------------------------------------
// REQUEST
//------------------------------------------------------------------------------

pub struct GetApiTicket<'src, CLType, DCType, const F: bool, const B: bool, const C: bool> {
	username: &'src str,
	password: &'src str,
	
	new_character_list: bool,
	include_bookmarks: bool,
	include_characters: bool,
	include_friends: bool,

	_character_list_type: PhantomData<CLType>,
	_character_type: PhantomData<DCType>,
}

impl<'src> GetApiTicket<'src, OldCL, OldDC, false, false, false> {
	pub fn new(username: &'src str, password: &'src str) -> GetApiTicket<'src, OldCL, OldDC, false, false, false> {
		GetApiTicket {
			username,
			password,

			new_character_list: false,
			include_bookmarks: false,
			include_characters: false,
			include_friends: false,

			_character_list_type: PhantomData,
			_character_type: PhantomData,
		}
	}
}

impl<'src, const F: bool, const B: bool, const C: bool> GetApiTicket<'src, OldCL, OldDC, F, B, C> {
	pub fn use_new_character_list(self) -> GetApiTicket<'src, NewCL, NewDC, F, B, C> {
		GetApiTicket {
			username: self.username,
			password: self.password,

			new_character_list: true,
			include_friends: self.include_friends,
			include_bookmarks: self.include_bookmarks,
			include_characters: self.include_characters,

			_character_list_type: PhantomData,
			_character_type: PhantomData,
		}
	}
}

impl<'src, CLType, DCType, const B: bool, const C: bool> GetApiTicket<'src, CLType, DCType, false, B, C>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	pub fn include_friends(self) -> GetApiTicket<'src, CLType, DCType, true, B, C> {
		GetApiTicket {
			username: self.username,
			password: self.password,

			new_character_list: self.new_character_list,
			include_bookmarks: self.include_bookmarks,
			include_characters: self.include_characters,
			include_friends: true,

			_character_list_type: PhantomData,
			_character_type: PhantomData,
		}
	}
}

impl<'src, CLType, DCType, const F: bool, const C: bool> GetApiTicket<'src, CLType, DCType, F, false, C>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	pub fn include_bookmarks(self) -> GetApiTicket<'src, CLType, DCType, F, true, C> {
		GetApiTicket {
			username: self.username,
			password: self.password,

			new_character_list: self.new_character_list,
			include_bookmarks: true,
			include_characters: self.include_characters,
			include_friends: self.include_friends,

			_character_list_type: PhantomData,
			_character_type: PhantomData,
		}
	}
}

impl<'src, CLType, DCType, const F: bool, const B: bool> GetApiTicket<'src, CLType, DCType, F, B, false>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	pub fn include_characters(self) -> GetApiTicket<'src, CLType, DCType, F, B, true> {
		GetApiTicket {
			username: self.username,
			password: self.password,

			new_character_list: self.new_character_list,
			include_bookmarks: self.include_bookmarks,
			include_characters: true,
			include_friends: self.include_friends,

			_character_list_type: PhantomData,
			_character_type: PhantomData,
		}
	}
}

impl<'src, CLType, DCType, const F: bool, const B: bool, const C: bool> GetApiTicket<'src, CLType, DCType, F, B, C>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
	GetApiTicketResponse<CLType, DCType, F, B, C>: Default,
{
	#[tracing::instrument(
		name = "Attempting F-List API authentication",
		level = "trace",
		skip(self),
	)]
	pub async fn execute(self, _http: HttpClient) -> ApiResult<GetApiTicketResponse<CLType, DCType, F, B, C>> {
		DeserializeError::from_value(GetApiTicketResponse::default()).into_result()
	}
}

impl<'src, CLType, DCType, const F: bool, const B: bool, const C: bool> Serialize
for GetApiTicket<'src, CLType, DCType, F, B, C>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let field_count =
			if self.include_characters && self.new_character_list {
				5
			} else {
				4
			};

		let mut serializer = serializer.serialize_struct(
			"Authenticate",
			field_count,
		)?;

		serializer.serialize_field("account", &self.username)?;
		serializer.serialize_field("password", &self.password)?;

		serializer.serialize_field("no_bookmarks", &(!self.include_bookmarks))?;
		serializer.serialize_field("no_characters", &(self.include_characters))?;
		serializer.serialize_field("no_friends", &(!self.include_friends))?;

		if self.include_characters && self.new_character_list {
			serializer.serialize_field("new_character_list", &true)?;
		}

		serializer.end()
	}
}

//------------------------------------------------------------------------------
// RESPONSE
//------------------------------------------------------------------------------

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GetApiTicketResponse<CLType, DCType, const F: bool, const B: bool, const C: bool> {
	#[serde(bound = "DCType: DeserializeOwned")]
	default_character: DCType,

	#[serde(default, bound = "CLType: DeserializeOwned")]
	characters: Option<CLType>,

	#[serde(default)]
	friends: Option<Vec<FriendBinding>>,

	#[serde(default)]
	bookmarks: Option<Vec<Bookmark>>,

	#[serde_as(as = "DisplayFromStr")]
	ticket: Ticket,
}

impl<CLType, DCType, const F: bool, const B: bool, const C: bool> GetApiTicketResponse<CLType, DCType, F, B, C>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	#[must_use]
	pub fn default_character(&self) -> &DCType {
		&self.default_character
	}

	#[must_use = "F-List API cannot be accessed without a ticket."]
	pub fn into_ticket(self) -> Ticket {
		self.ticket
	}
}

impl<CLType, DCType, const F: bool, const B: bool> GetApiTicketResponse<CLType, DCType, F, B, true>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	#[must_use]
	pub fn characters(&self) -> &CLType {
		self.characters.as_ref().unwrap()
	}
}

impl<CLType, DCType, const B: bool, const C: bool> GetApiTicketResponse<CLType, DCType, true, B, C>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	#[must_use]
	pub fn friends(&self) -> &[FriendBinding] {
		self.friends.as_deref().unwrap()
	}
}

impl<CLType, DCType, const F: bool, const C: bool> GetApiTicketResponse<CLType, DCType, F, true, C>
where
	CLType: DeserializeOwned,
	DCType: DeserializeOwned,
{
	#[must_use]
	pub fn bookmarks(&self) -> &[Bookmark] {
		self.bookmarks.as_deref().unwrap()
	}
}

impl<const F: bool, const B: bool, const C: bool> Default
for GetApiTicketResponse<Vec<String>, String, F, B, C> {
	fn default() -> Self {
		let characters = mock::character_names();
		let friends = mock::friends();

		Self {
			default_character: String::from(mock::default_character_name()),

			bookmarks: B.then(|| vec![]),
			characters: C.then(|| characters.map(String::from).to_vec()),
			friends: F.then(|| FriendBinding::from_list(&friends)),

			ticket: Ticket::default(),
		}
	}
}

impl<const F: bool, const B: bool, const C: bool> Default
for GetApiTicketResponse<HashMap<String, CharacterId>, CharacterId, F, B, C> {
	fn default() -> Self {
		let characters = mock::characters();
		let friends = mock::friends();

		Self {
			default_character: mock::default_character_id(),

			bookmarks: B.then(|| vec![]),
			characters: C.then(|| HashMap::from(characters.map(|(id, name)| (name.to_string(), id)))),
			friends: F.then(|| FriendBinding::from_list(&friends)),

			ticket: Ticket::default(),
		}
	}
}

//------------------------------------------------------------------------------
// DATA
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct Bookmark {
	#[serde(rename = "name")]
	pub character_name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FriendBinding {
	#[serde(rename = "dest_name")]
	pub friend_name: String,

	#[serde(rename = "source_name")]
	pub own_name: String,
}

impl FriendBinding {
	pub(super) fn from_list(list: &[(CharacterId, &'static [&'static str])]) -> Vec<FriendBinding> {
		let characters = mock::characters();

		list.into_iter()
			.flat_map(|(own_id, names)| names.into_iter().map(move |name| (*own_id, name)))
			.fold(vec![], |mut acc, (own_id, name)| {
				acc.push(FriendBinding {
					friend_name: name.to_string(),
					own_name: characters.iter()
						.find_map(|(mock_id, name)| (own_id == *mock_id).then_some(name.to_string()))
						.unwrap()
				});

				acc
			})
	}
}
