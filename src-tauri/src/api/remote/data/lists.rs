use super::serialize_account;
use crate::api::{AccountCredentials, ApiClient};
use crate::api::error::{DeserializeError, Result as ApiResult};
use crate::api::remote::data::mock;
use reqwest::Client as HttpClient;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::collections::BTreeMap;

//------------------------------------------------------------------------------
// API CLIENT IMPLEMENTATION
//------------------------------------------------------------------------------

impl ApiClient {
	pub async fn get_global_field_list(http: HttpClient) -> ApiResult<BTreeMap<u64, InfoListGroup>> {
		GetInfoList.execute(http).await.map(|response| response.into_groups())
	}

	pub async fn get_global_kink_list(http: HttpClient) -> ApiResult<BTreeMap<u64, KinkGroup>> {
		GetKinkList.execute(http).await.map(|response| response.into_groups())
	}

	// TODO: reformat? decide on better representation
	pub async fn get_global_mapping_list(http: HttpClient) -> ApiResult<GetMappingListResponse> {
		GetMappingList.execute(http).await
	}

	pub async fn get_group_list(&self) -> ApiResult<Vec<String>> {
		let mut account = self.account.credentials().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		GetGroupList::new()
			.use_account(account)
			.execute(self.http()).await
			.map(|response| response.into_list())
	}

	pub async fn get_ignore_list(&self) -> ApiResult<Vec<String>> {
		let mut account = self.account.credentials().await;
		let account = &mut *account;

		account.refresh_if_needed(self.http()).await?;

		GetIgnoreList::new()
			.use_account(account)
			.execute(self.http()).await
			.map(|response| response.into_list())
	}
}

//------------------------------------------------------------------------------
// REQUEST
//------------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GetGroupList<'client, const A: bool> {
	account: Option<&'client AccountCredentials>,
}

impl GetGroupList<'_, false> {
	pub fn new() -> Self {
		GetGroupList {
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client AccountCredentials) -> GetGroupList<'client, true> {
		GetGroupList {
			account: Some(account),
		}
	}
}

impl GetGroupList<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<GetGroupListResponse> {
		DeserializeError::from_value(GetGroupListResponse::default()).into_result()
	}
}

impl Serialize for GetGroupList<'_, true> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let account = self.account.unwrap();
		let serializer = serializer.serialize_struct("GetIgnoreList", 2)?;

		serialize_account(serializer, &account)
	}
}

#[derive(Debug, Clone)]
pub struct GetIgnoreList<'client, const A: bool> {
	account: Option<&'client AccountCredentials>,
}

impl GetIgnoreList<'_, false> {
	pub fn new() -> Self {
		GetIgnoreList {
			account: None,
		}
	}

	pub fn use_account<'client>(self, account: &'client AccountCredentials) -> GetIgnoreList<'client, true> {
		GetIgnoreList {
			account: Some(account),
		}
	}
}

impl GetIgnoreList<'_, true> {
	pub async fn execute(self, _http: HttpClient) -> ApiResult<GetIgnoreListResponse> {
		DeserializeError::from_value(GetIgnoreListResponse::default()).into_result()
	}
}

impl Serialize for GetIgnoreList<'_, true> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		let account = self.account.unwrap();
		let serializer = serializer.serialize_struct("GetIgnoreList", 2)?;

		serialize_account(serializer, &account)
	}
}

#[derive(Debug, Clone, Copy)]
pub struct GetInfoList;
impl GetInfoList {
	#[tracing::instrument(
		name = "Retrieving profile field mapping from F-List"
		level = "trace",
		skip(self),
	)]
	pub async fn execute(self, _http: HttpClient) -> ApiResult<GetInfoListResponse> {
		DeserializeError::from_value(GetInfoListResponse::default()).into_result()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct GetKinkList;
impl GetKinkList {
	#[tracing::instrument(
		name = "Retrieving kink mapping from F-List",
		level = "trace",
		skip(self),
	)]
	pub async fn execute(self, _http: HttpClient) -> ApiResult<GetKinkListResponse> {
		DeserializeError::from_value(GetKinkListResponse::default()).into_result()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct GetMappingList;
impl GetMappingList {
	#[tracing::instrument(
		name = "Retrieving global mapping information from F-List",
		level = "trace",
		skip(self),
	)]
	pub async fn execute(self, _http: HttpClient) -> ApiResult<GetMappingListResponse> {
		DeserializeError::from_value(GetMappingListResponse::default()).into_result()
	}
}

//------------------------------------------------------------------------------
// RESPONSE
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct GetGroupListResponse {
	groups: Vec<String>,
}

impl GetGroupListResponse {
	pub fn into_list(self) -> Vec<String> {
		self.groups
	}
}

impl Default for GetGroupListResponse {
	fn default() -> Self {
		GetGroupListResponse {
			groups: vec![],
		}
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetIgnoreListResponse {
	#[serde(rename = "ignores")]
	characters: Vec<String>,
}

impl GetIgnoreListResponse {
	pub fn into_list(self) -> Vec<String> {
		self.characters
	}
}

impl Default for GetIgnoreListResponse {
	fn default() -> Self {
		GetIgnoreListResponse {
			characters: vec![],
		}
	}
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GetInfoListResponse {
	#[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
	info: BTreeMap<u64, InfoListGroup>,
}

impl GetInfoListResponse {
	pub fn into_groups(self) -> BTreeMap<u64, InfoListGroup> {
		self.info
	}
}

impl Default for GetInfoListResponse {
	fn default() -> Self {
		mock::info_list()
	}
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct GetKinkListResponse {
	#[serde_as(as = "BTreeMap<DisplayFromStr, _>")]
	kinks: BTreeMap<u64, KinkGroup>,
}

impl GetKinkListResponse {
	pub fn into_groups(self) -> BTreeMap<u64, KinkGroup> {
		self.kinks
	}
}

impl Default for GetKinkListResponse {
	fn default() -> Self {
		mock::kink_list()
	}
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetMappingListResponse {
	kinks: Vec<MapKink>,
	kink_groups: Vec<MapGroup>,

	infotags: Vec<MapInfoTag>,
	infotag_groups: Vec<MapGroup>,

	listitems: Vec<MapListItem>,
}

impl GetMappingListResponse {
	pub fn kinks(&self) -> &[MapKink] {
		&self.kinks
	}

    pub fn kink_groups(&self) -> &[MapGroup] {
        &self.kink_groups
    }

    pub fn infotags(&self) -> &[MapInfoTag] {
        &self.infotags
    }

    pub fn infotag_groups(&self) -> &[MapGroup] {
        &self.infotag_groups
    }

    pub fn listitems(&self) -> &[MapListItem] {
        &self.listitems
    }
}

impl Default for GetMappingListResponse {
	fn default() -> Self {
		mock::mapping_list()
	}
}

//------------------------------------------------------------------------------
// DATA
//------------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct InfoListGroup {
	#[serde(rename = "group")]
	pub group_name: String,

	pub items: Vec<InfoListItem>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct InfoListItem {
	#[serde_as(as = "DisplayFromStr")]
	pub id: u64,

	pub name: String,
	
	#[serde(flatten)]
	pub kind: ItemKind,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ItemKind {
	List {
		list: Vec<String>,
	},

	Text,
}

impl ItemKind {
	pub fn list_items(&self) -> Option<&[String]> {
		match self {
			Self::List { list } => Some(list),
			Self::Text => None,
		}
	}
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct Kink {
	#[serde_as(as = "DisplayFromStr")]
	#[serde(rename = "kink_id")]
	pub id: u64,

	pub name: String,
	pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KinkGroup {
	#[serde(rename = "group")]
	pub group_name: String,

	pub items: Vec<Kink>,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct MapGroup {
	#[serde_as(as = "DisplayFromStr")]
	pub id: u64,

	pub name: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct MapInfoTag {
	#[serde_as(as = "DisplayFromStr")]
	pub id: u64,

	pub name: String,
	
	#[serde(flatten)]
	pub kind: ItemKind,

	#[serde_as(as = "DisplayFromStr")]
	pub group_id: u64,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct MapListItem {
	#[serde_as(as = "DisplayFromStr")]
	pub id: u64,

	pub name: String,
	pub value: String,
}

#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct MapKink {
	#[serde_as(as = "DisplayFromStr")]
	pub id: u64,

	pub name: String,
	pub description: String,

	#[serde_as(as = "DisplayFromStr")]
	pub group_id: u64,
}
