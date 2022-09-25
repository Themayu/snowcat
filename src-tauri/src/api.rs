pub mod error;

mod remote;

use crate::api::error::Result as ApiResult;
use crate::api::remote::data::ticket::GetApiTicket;
use crate::util::hex::{Hex, HexFromStrError};
use reqwest::Client as HttpClient;
use serde::Serialize;
use snowcat_common::characters::CharacterId;
use tauri::async_runtime::Mutex;
use std::collections::HashMap;
use std::fmt;
use std::str;
use thiserror::Error;
use time::{Duration, OffsetDateTime};

type BookmarksList = Vec<String>;
type CharactersList = HashMap<CharacterId, String>;
type FriendsList = HashMap<CharacterId, Vec<String>>;

const TICKET_LIFETIME: Duration = Duration::minutes(5);

#[derive(Serialize)]
pub struct Account {
	ticket: Ticket,

	#[serde(rename = "account")]
	username: String,

	#[serde(skip)]
	password: String,

	#[serde(skip)]
	expires_at: OffsetDateTime,
}

impl Account {
	async fn authenticate(
		http: HttpClient,
		bookmarks_list: &mut Option<BookmarksList>,
		characters_list: &mut Option<CharactersList>,
		friends_list: &mut Option<FriendsList>,
		default_character: &mut Option<CharacterId>,
		username: &str,
		password: &str,
	) -> ApiResult<Account> {
		let account = GetApiTicket::new(username, password)
			.include_bookmarks()
			.include_characters()
			.include_friends()
			.use_new_character_list()
			.execute(http).await?;

		let bookmarks = account.bookmarks().iter().map(|bookmark| bookmark.character_name.clone());
		let characters = account.characters().iter().map(|(name, &id)| (id, name.to_string()));

		let friends = account.friends().iter().map({
			let characters = account.characters();
			move |binding| {
				let mut characters = characters.iter();

				let friend_name = &binding.friend_name;
				let own_name = &binding.own_name;

				let (_, &own_id) = characters.find(|(name, _)| *name == own_name).unwrap();

				(own_id, friend_name)
			}
		}).fold(HashMap::new(), |mut map, (own_id, friend_name)| {
			let friend_name = friend_name.clone();

			map.entry(own_id)
				.or_insert_with(Vec::new)
				.push(friend_name);

			map
		});

		bookmarks_list.replace(bookmarks.collect());
		characters_list.replace(characters.collect());
		friends_list.replace(friends);

		default_character.replace(*account.default_character());

		Ok(Account {
			username: String::from(username),
			password: String::from(password),
			ticket: account.into_ticket(),

			expires_at: OffsetDateTime::now_utc() + TICKET_LIFETIME,
		})
	}

	async fn refresh_ticket(&mut self, http: HttpClient) -> ApiResult<()> {
		self.ticket = GetApiTicket::new(&self.username, &self.password)
			.execute(http).await?
			.into_ticket();

		self.expires_at = OffsetDateTime::now_utc() + TICKET_LIFETIME;

		Ok(())
	}

	async fn refresh_if_needed(&mut self, http: HttpClient) -> ApiResult<()> {
		if self.expires_at <= OffsetDateTime::now_utc() {
			self.refresh_ticket(http).await?;
		};

		Ok(())
	}

	fn invalidate_ticket(&mut self) {
		self.expires_at = OffsetDateTime::now_utc();
	}
}

impl fmt::Debug for Account {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "<F-List API Account>")
	}
}

#[derive(Debug)]
pub struct ApiClient {
	account: Mutex<Account>,
	http: HttpClient,
}

impl ApiClient {
	/// Attempt to obtain an API token from the F-List API.
	pub async fn authenticate(
		http: HttpClient,
		username: &str,
		password: &str,
	) -> ApiResult<(ApiClient, CharactersInfo)> {
		let mut bookmarks_list = None;
		let mut characters_list = None;
		let mut friends_list = None;
		let mut default_character = None;

		let account = Mutex::new(Account::authenticate(
			http.clone(),
			&mut bookmarks_list,
			&mut characters_list,
			&mut friends_list,
			&mut default_character,
			username,
			password,
		).await?);

		// Either we successfully assigned to the references above, or we
		// crashed when trying to retrieve the values. Either way, if we get
		// here, these options contain values.
		let bookmarks_list = bookmarks_list.unwrap();
		let characters_list = characters_list.unwrap();
		let friends_list = friends_list.unwrap();
		let default_character = default_character.unwrap();

		let client = ApiClient {
			account,
			http,
		};

		let character_info = CharactersInfo {
			default_character,
			bookmarks_list,
			characters_list,
			friends_list
		};

		Ok((client, character_info))
	}

	/// Mark the current API ticket as invalid.
	///
	/// Useful for when the API returns the error "Invalid ticket." The API
	/// call should be attempted again after calling this function.
	pub fn invalidate_ticket(&mut self) {
		self.account.blocking_lock()
			.invalidate_ticket();
	}

	/// Retrieve the HTTP client from this API client.
	pub(self) fn http(&self) -> HttpClient {
		self.http.clone()
	}
}

pub struct CharactersInfo {
	pub default_character: CharacterId,

	pub bookmarks_list: BookmarksList,
	pub characters_list: CharactersList,
	pub friends_list: FriendsList,
}

#[derive(Clone, Copy)]
pub struct Ticket(Hex<32>);

impl Ticket {
	fn value(&self) -> String {
		format!("fct_{hex}", hex = self.0)
	}
}

impl Default for Ticket {
	fn default() -> Self {
		Ticket(Hex::default())
	}
}

impl fmt::Debug for Ticket {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "<F-List API Ticket>")
	}
}

impl str::FromStr for Ticket {
	type Err = TicketParseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if !s.starts_with("fct_") {
			return Err(TicketParseError::MissingPrefix);
		}

		let hex = Hex::from_str(&s[4..])
			.map_err(|err| TicketParseError::HexFromStrError(err))?;

		Ok(Ticket(hex))
	}
}

impl Serialize for Ticket {
	fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		s.serialize_str(&self.value())
	}
}

#[derive(Debug, Clone, Copy, Error)]
pub enum TicketParseError {
	#[error("{0}")]
	HexFromStrError(HexFromStrError),

	#[error("Missing ticket prefix.")]
	MissingPrefix,
}
