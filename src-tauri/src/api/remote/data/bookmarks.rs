use crate::api::{Account, ApiClient};
use crate::api::error::{DeserializeError, Result as ApiResult};
use crate::api::remote::data::mock;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};

//------------------------------------------------------------------------------
// API CLIENT IMPLEMENTATION
//------------------------------------------------------------------------------

impl ApiClient {
	pub async fn add_bookmark(&self, character: &str) -> ApiResult<()> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		AddBookmark::new(character)
			.use_account(account)
			.execute(self.http()).await
	}

	pub async fn list_bookmarks(&self) -> ApiResult<Vec<String>> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		ListBookmarks::new()
			.use_account(account)
			.execute(self.http()).await
			.map(|response| response.into_list())
	}

	pub async fn remove_bookmark(&self, character: &str) -> ApiResult<()> {
		let mut account = self.account.lock().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		RemoveBookmark::new(character)
			.use_account(account)
			.execute(self.http()).await
	}
}

//------------------------------------------------------------------------------
// REQUEST
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct AddBookmark<'client, 'command, const A: bool> {
	name: &'command str,

	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl<'client, 'command> AddBookmark<'client, 'command, false> {
	pub fn new(character: &'command str) -> AddBookmark<'_, 'command, false> {
		AddBookmark {
			name: character,
			account: None,
		}
	}

	pub fn use_account(self, account: &'client Account) -> AddBookmark<'client, 'command, true> {
		AddBookmark {
			name: self.name,
			account: Some(account),
		}
	}
}

impl AddBookmark<'_, '_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<()> {
		DeserializeError::from_value(()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct ListBookmarks<'client, const A: bool> {
	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl ListBookmarks<'_, false> {
	pub fn new() -> Self {
		ListBookmarks {
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client Account) -> ListBookmarks<'client, true> {
		ListBookmarks {
			account: Some(account),
		}
	}
}

impl ListBookmarks<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<ListBookmarksResponse> {
		DeserializeError::from_value(ListBookmarksResponse::default()).into_result()
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoveBookmark<'client, 'command, const A: bool> {
	name: &'command str,

	#[serde(flatten)]
	account: Option<&'client Account>,
}

impl<'client, 'command> RemoveBookmark<'client, 'command, false> {
	pub fn new(character: &'command str) -> RemoveBookmark<'_, 'command, false> {
		RemoveBookmark {
			name: character,
			account: None,
		}
	}

	pub fn use_account(self, account: &'client Account) -> RemoveBookmark<'client, 'command, true> {
		RemoveBookmark {
			name: self.name,
			account: Some(account),
		}
	}
}

impl RemoveBookmark<'_, '_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<()> {
		DeserializeError::from_value(()).into_result()
	}
}

//------------------------------------------------------------------------------
// RESPONSE
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct ListBookmarksResponse {
	#[serde(rename = "characters")]
	bookmarks: Vec<String>,
}

impl ListBookmarksResponse {
	pub fn into_list(self) -> Vec<String> {
		self.bookmarks
	}
}

impl Default for ListBookmarksResponse {
	fn default() -> Self {
		ListBookmarksResponse {
			bookmarks: mock::bookmarks().iter().map(ToString::to_string).collect()
		}
	}
}

//------------------------------------------------------------------------------
// DATA
//------------------------------------------------------------------------------
