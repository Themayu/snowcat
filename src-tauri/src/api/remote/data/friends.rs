use crate::api::{Account, ApiClient};
use crate::api::error::{DeserializeError, Result as ApiResult};
use crate::api::remote::data::mock;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use std::fmt;
use time::{Duration, OffsetDateTime};

//------------------------------------------------------------------------------
// API CLIENT IMPLEMENTATION
//------------------------------------------------------------------------------

impl ApiClient {
	pub async fn list_friends(&self) -> ApiResult<Vec<FriendBinding>> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		ListFriends::new()
			.use_account(account)
			.execute(self.http()).await
			.map(|response| response.into_list())
	}

	pub async fn remove_friend(&self, friend: &str) -> ApiResult<()> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		RemoveFriend::new(friend)
			.use_account(account)
			.execute(self.http()).await
	}

	pub async fn accept_friend_request(&self, request_id: u64) -> ApiResult<()> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		AcceptFriendRequest::new(request_id)
			.use_account(account)
			.execute(self.http()).await
	}

	pub async fn cancel_friend_request(&self, request_id: u64) -> ApiResult<()> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		CancelFriendRequest::new(request_id)
			.use_account(account)
			.execute(self.http()).await
	}

	pub async fn deny_friend_request(&self, request_id: u64) -> ApiResult<()> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		DenyFriendRequest::new(request_id)
			.use_account(account)
			.execute(self.http()).await
	}

	pub async fn list_friend_requests(&self, list: ListType) -> ApiResult<Vec<FriendRequest>> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		match list {
			ListType::Incoming => {
				ListIncomingFriendRequests::new()
					.use_account(account)
					.execute(self.http()).await
					.map(|response| response.into_list())
			},

			ListType::Outgoing => {
				ListOutgoingFriendRequests::new()
					.use_account(account)
					.execute(self.http()).await
					.map(|response| response.into_list())
			},
		}
	}

	pub async fn send_friend_request<'command>(&self, source: &'command str, target: &'command str) -> ApiResult<()> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		SendFriendRequest::new()
			.with_source(source)
			.with_target(target)
			.use_account(account)
			.execute(self.http()).await
	}
}

//------------------------------------------------------------------------------
// REQUEST
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ListFriends<'client, const A: bool> {
	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl ListFriends<'_, false> {
	pub fn new() -> Self {
		ListFriends {
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> ListFriends<'client, true> {
		ListFriends {
			account: Some(account),
		}
	}
}

impl ListFriends<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<ListFriendsResponse> {
		DeserializeError::from_value(ListFriendsResponse::default()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveFriend<'client, 'command, const A: bool> {
	friend: &'command str,
	
	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl<'client, 'command> RemoveFriend<'client, 'command, false> {
	pub fn new(friend: &'command str) -> RemoveFriend<'_, 'command, false> {
		RemoveFriend {
			friend,
			account: None,
		}
	}

	pub fn use_account(self, account: &'client Account) -> RemoveFriend<'client, 'command, true> {
		RemoveFriend {
			friend: self.friend,
			account: Some(account),
		}
	}
}

impl RemoveFriend<'_, '_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<()> {
		DeserializeError::from_value(()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct AcceptFriendRequest<'client, const A: bool> {
	request_id: u64,

	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl AcceptFriendRequest<'_, false> {
	pub fn new(request_id: u64) -> Self {
		AcceptFriendRequest {
			request_id,
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> AcceptFriendRequest<'client, true> {
		AcceptFriendRequest {
			request_id: self.request_id,
			account: Some(account),
		}
	}
}

impl AcceptFriendRequest<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<()> {
		DeserializeError::from_value(()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct CancelFriendRequest<'client, const A: bool> {
	request_id: u64,

	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl CancelFriendRequest<'_, false> {
	pub fn new(request_id: u64) -> Self {
		CancelFriendRequest {
			request_id,
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> CancelFriendRequest<'client, true> {
		CancelFriendRequest {
			request_id: self.request_id,
			account: Some(account),
		}
	}
}

impl CancelFriendRequest<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<()> {
		DeserializeError::from_value(()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct DenyFriendRequest<'client, const A: bool> {
	request_id: u64,

	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl DenyFriendRequest<'_, false> {
	pub fn new(request_id: u64) -> Self {
		DenyFriendRequest {
			request_id,
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> DenyFriendRequest<'client, true> {
		DenyFriendRequest {
			request_id: self.request_id,
			account: Some(account),
		}
	}
}

impl DenyFriendRequest<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<()> {
		DeserializeError::from_value(()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct ListIncomingFriendRequests<'client, const A: bool> {
	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl ListIncomingFriendRequests<'_, false> {
	pub fn new() -> Self {
		ListIncomingFriendRequests {
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> ListIncomingFriendRequests<'client, true> {
		ListIncomingFriendRequests { 
			account: Some(account),
		}
	}
}

impl ListIncomingFriendRequests<'_, true> {
	pub async fn execute(&self, _http: HttpClient) -> ApiResult<ListIncomingFriendRequestsResponse> {
		DeserializeError::from_value(ListIncomingFriendRequestsResponse::default()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct ListOutgoingFriendRequests<'client, const A: bool> {
	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl ListOutgoingFriendRequests<'_, false> {
	pub fn new() -> Self {
		ListOutgoingFriendRequests {
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> ListOutgoingFriendRequests<'client, true> {
		ListOutgoingFriendRequests { 
			account: Some(account),
		}
	}
}

impl ListOutgoingFriendRequests<'_, true> {
	pub async fn execute(&self, _http: HttpClient) -> ApiResult<ListOutgoingFriendRequestsResponse> {
		DeserializeError::from_value(ListOutgoingFriendRequestsResponse::default()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct SendFriendRequest<'client, 'command, const S: bool, const T: bool, const A: bool> {
	#[serde(rename = "source_name")]
	source: Option<&'command str>,
	
	#[serde(rename = "dest_name")]
	target: Option<&'command str>,
	
	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl SendFriendRequest<'_, '_, false, false, false> {
	pub fn new() -> Self {
		SendFriendRequest {
			source: None,
			target: None,
			account: None,
		}
	}
}

impl<'client, 'command, const T: bool, const A: bool> SendFriendRequest<'client, 'command, false, T, A> {
	pub fn with_source(self, source: &'command str) -> SendFriendRequest<'client, 'command, true, T, A> {
		SendFriendRequest {
			source: Some(source),
			target: self.target,
			account: self.account,
		}
	}
}

impl<'client, 'command, const S: bool, const A: bool> SendFriendRequest<'client, 'command, S, false, A> {
	pub fn with_target(self, target: &'command str) -> SendFriendRequest<'client, 'command, S, true, A> {
		SendFriendRequest {
			source: self.source,
			target: Some(target),
			account: self.account,
		}
	}
}

impl<'command, const S: bool, const T: bool> SendFriendRequest<'_, 'command, S, T, false> {
	pub fn use_account<'client>(self, account: &'client Account) -> SendFriendRequest<'client, 'command, S, T, true> {
		SendFriendRequest {
			source: self.source,
			target: self.target,
			account: Some(account),
		}
	}
}

impl SendFriendRequest<'_, '_, true, true, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<()> {
		DeserializeError::from_value(()).into_result()
	}
}

//------------------------------------------------------------------------------
// PARAMETERS
//------------------------------------------------------------------------------

pub enum ListType {
	Incoming,
	Outgoing,
}

//------------------------------------------------------------------------------
// RESPONSE
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ListFriendsResponse {
	friends: Vec<FriendBinding>,
}

impl ListFriendsResponse {
	pub fn into_list(self) -> Vec<FriendBinding> {
		self.friends
	}
}

impl Default for ListFriendsResponse {
	fn default() -> Self {
		ListFriendsResponse {
			friends: mock::friends()
				.iter()
				.flat_map({
					let characters = mock::characters();

					move |&(own_id, friends)| {
						let mut characters = characters.iter();
						let &(_, own_name) = characters.find(|&&(id, _)| id == own_id).unwrap();

						friends.into_iter().map({
							let own_name = own_name.to_string();

							move |friend_name| {
								(own_name.clone(), friend_name.to_string())
							}
						})
					}
				})
				.map(|(character_name, friend_name)| {
					FriendBinding {
						character_name,
						friend_name,

						last_online: OffsetDateTime::UNIX_EPOCH,
					}
				})
				.collect()
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListIncomingFriendRequestsResponse {
	requests: Vec<FriendRequest>,
}

impl ListIncomingFriendRequestsResponse {
	pub fn into_list(self) -> Vec<FriendRequest> {
		self.requests
	}
}

impl Default for ListIncomingFriendRequestsResponse {
	fn default() -> Self {
		ListIncomingFriendRequestsResponse {
			requests: vec![
				FriendRequest {
					id: 0,
					from: String::from("Dimentio"),
					to: String::from("Sarah Blitz Garrison"),
				},

				FriendRequest {
					id: 1,
					from: String::from("Feedback"),
					to: String::from("Sarah Blitz Garrison"),
				},

				FriendRequest {
					id: 2,
					from: String::from("Shweetz"),
					to: String::from("Sarah Blitz Garrison"),
				},

				FriendRequest {
					id: 3,
					from: String::from("Feedback"),
					to: String::from("Yanozo Serna"),
				},
			],
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListOutgoingFriendRequestsResponse {
	requests: Vec<FriendRequest>,
}

impl ListOutgoingFriendRequestsResponse {
	pub fn into_list(self) -> Vec<FriendRequest> {
		self.requests
	}
}

impl Default for ListOutgoingFriendRequestsResponse {
	fn default() -> Self {
		ListOutgoingFriendRequestsResponse {
			requests: vec![
				FriendRequest {
					id: 4,
					from: String::from("Sarah Blitz Garrison"),
					to: String::from("Element Four"),
				},

				FriendRequest {
					id: 5,
					from: String::from("Markelio"),
					to: String::from("Feedback"),
				},

				FriendRequest {
					id: 6,
					from: String::from("Markelio"),
					to: String::from("Scinner"),
				},

				FriendRequest {
					id: 7,
					from: String::from("Yanozo Serna"),
					to: String::from("Korban"),
				},

				FriendRequest {
					id: 8,
					from: String::from("Phoney Baloney"),
					to: String::from("Vandel"),
				},

				FriendRequest {
					id: 9,
					from: String::from("Phoney Baloney"),
					to: String::from("Korban"),
				},

				FriendRequest {
					id: 10,
					from: String::from("Marabel Thorne"),
					to: String::from("Vile"),
				},

				FriendRequest {
					id: 11,
					from: String::from("Marabel Thorne"),
					to: String::from("Scinner"),
				},
			],
		}
	}
}

//------------------------------------------------------------------------------
// DATA
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct FriendBinding {
	#[serde(rename = "source")]
	pub character_name: String,

	#[serde(rename = "dest")]
	pub friend_name: String,

	#[serde(deserialize_with = "integer_past_duration_to_datetime")]
	pub last_online: OffsetDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FriendRequest {
	pub id: u64,

	#[serde(rename = "source")]
	pub from: String,

	#[serde(rename = "dest")]
	pub to: String,
}

//------------------------------------------------------------------------------
// HELPER
//------------------------------------------------------------------------------

/// Deserialize an integer representing a duration in the past as a
/// OffsetDateTime
pub fn integer_past_duration_to_datetime<'de, D>(deserializer: D) -> Result<OffsetDateTime, D::Error>
where
	D: serde::Deserializer<'de>,
{
	use serde::de::{Error, Visitor};

	struct DurationVisitor;
	impl<'de> Visitor<'de> for DurationVisitor {
		type Value = Duration;
		
		fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
			write!(f, "an integer duration in seconds")
		}

		fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
		where
			E: Error,
		{
			Ok(Duration::new(v, 0))
		}
	}

	let duration = deserializer.deserialize_i64(DurationVisitor)?;
	OffsetDateTime::now_utc().checked_sub(duration)
		.ok_or_else(|| Error::custom("underflow when subtracting duration from now_utc"))
}
