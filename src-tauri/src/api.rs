pub mod channels;
pub mod characters;
pub mod error;
pub mod url_helpers;

mod remote;

pub use crate::api::remote::constants::headers;

use crate::api::characters::CharacterId;
use crate::api::error::Result as ApiResult;
use crate::api::remote::data::ticket::GetApiTicket;
use crate::util::hex::{Hex, HexFromStrError};
use reqwest::Client as HttpClient;
use serde::Serialize;
use tauri::async_runtime::Mutex;
use tokio::sync::MutexGuard;
use std::collections::HashMap;
use std::{fmt, str};
use thiserror::Error;
use time::{Duration, OffsetDateTime};

type BookmarksList = Vec<String>;
type CharactersList = HashMap<CharacterId, String>;
type FriendsList = HashMap<CharacterId, Vec<String>>;

const TICKET_LIFETIME: Duration = Duration::minutes(5);

#[derive(Serialize)]
pub struct AccountCredentials {
	ticket: Ticket,

	#[serde(rename = "account")]
	username: String,

	#[serde(skip)]
	password: String,

	#[serde(skip)]
	expires_at: OffsetDateTime,
}

impl AccountCredentials {
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

impl fmt::Debug for AccountCredentials {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "<F-List API Account>")
	}
}

#[derive(Debug)]
pub struct Account {
	pub default_character: CharacterId,

	pub bookmarks_list: BookmarksList,
	pub characters_list: CharactersList,
	pub friends_list: FriendsList,

	credentials: Mutex<AccountCredentials>,
}

impl Account {
	async fn authenticate(
		http: HttpClient,
		username: &str,
		password: &str,
	) -> ApiResult<Self> {
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

		let bookmarks_list = bookmarks.collect();
		let characters_list = characters.collect();
		let friends_list = friends;

		let default_character = *account.default_character();

		let credentials = Mutex::new(AccountCredentials {
			username: String::from(username),
			password: String::from(password),
			ticket: account.into_ticket(),

			expires_at: OffsetDateTime::now_utc() + TICKET_LIFETIME,
		});

		let account = Account {
			credentials,
			default_character,
			bookmarks_list,
			characters_list,
			friends_list
		};

		Ok(account)
	}

	async fn credentials(&self) -> MutexGuard<'_, AccountCredentials> {
		self.credentials.lock().await
	}
}

#[derive(Debug)]
pub struct ApiClient {
	account: Account,
	http: HttpClient,
}

impl ApiClient {
	/// Attempt to authenticate with the F-List API.
	pub async fn authenticate(
		http: HttpClient,
		username: &str,
		password: &str,
	) -> ApiResult<ApiClient> {
		Ok(ApiClient {
			account: Account::authenticate(http.clone(), username, password).await?,
			http,
		})
	}

	/// Mark the current API ticket as invalid.
	///
	/// Useful for when the API returns the error "Invalid ticket." The API
	/// call should be attempted again after calling this function.
	pub fn invalidate_ticket(&mut self) {
		self.account.credentials.blocking_lock()
			.invalidate_ticket();
	}

	/// Retrieve the account information from this API client.
	pub fn account(&self) -> &Account {
		&self.account
	}

	/// Retrieve the HTTP client from this API client.
	pub(self) fn http(&self) -> HttpClient {
		self.http.clone()
	}
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
