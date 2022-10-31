use crate::api::{ApiClient, Account};
use crate::api::error::Result as ApiResult;
use crate::api::channels::{ChannelId, ChannelInfo};
use crate::api::characters::{CharacterId, CharacterInfo};
use crate::io::Connection;
use crate::state::prelude::*;
use reqwest::Client as HttpClient;
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::ops::{Index, IndexMut};
use thiserror::Error;

pub type ChannelListResult<T> = Result<T, ChannelListError>;
pub type CharacterListResult<T> = Result<T, CharacterListError>;

pub struct Client {
	api_client: RwLock<Option<ApiClient>>,
	connection: RwLock<Option<Connection>>,
	http: HttpClient,
}

impl Client {
	pub fn new(http: HttpClient) -> Self {
		Client {
			api_client: RwLock::new(None),
			connection: RwLock::new(None),
			http,
		}
	}

	pub async fn authenticate(&self, username: &str, password: &str) -> ApiResult<()> {
		let Client { http, .. } = self;

		*self.api_client_mut().await = Some(ApiClient::authenticate(http.clone(), username, password).await?);

		Ok(())
	}

	pub async fn deauthenticate(&self) {
		*self.api_client_mut().await = None;
	}

	pub async fn account_info(&self) -> RwLockReadGuard<'_, Account> {
		RwLockReadGuard::map(self.api_client().await, |opt| opt.as_ref().map(|api| api.account()).unwrap())
	}

	async fn api_client(&self) -> RwLockReadGuard<'_, Option<ApiClient>> {
		self.api_client.read().await
	}

	async fn api_client_mut(&self) -> RwLockWriteGuard<'_, Option<ApiClient>> {
		self.api_client.write().await
	}

	async fn socket_client(&self) -> RwLockReadGuard<'_, Option<Connection>> {
		self.connection.read().await
	}

	async fn socket_client_mut(&self) -> RwLockWriteGuard<'_, Option<Connection>> {
		self.connection.write().await
	}
}

pub struct ChannelList {
	id_map: BTreeMap<ChannelId, ChannelInfo>,
}

impl ChannelList {
	/// Create a new, empty channel cache with the global allocator.
	///
	/// Does not allocate anything on its own.
	pub fn new() -> Self {
		ChannelList {
			id_map: BTreeMap::new(),
		}
	}

	/// Insert a channel into the list, using its ID as the key.
	/// 
	/// Returns the channel ID to make future lookups easier.
	/// 
	/// # Example
	/// ```rust
	/// use snowcat::api::channels::{ChannelId, ChannelInfo};
	/// use snowcat::client::ChannelList;
	/// 
	/// let mut list = ChannelList::new();
	/// 
	/// let channel_id = "ADH-{example}";
	/// let channel_name = "Example Channel";
	/// 
	/// let channel = ChannelInfo {
	/// 	id: channel_id.into(),
	/// 	display_name: channel_name.to_owned(),
	/// 	character_count: 14,
	/// 
	/// 	is_official: false,
	/// };
	/// 
	/// let id = list.insert(channel.clone()).expect("channel should be inserted successfully");
	/// 
	/// assert_eq!(list.get(&id), Some(&channel));
	/// assert_eq!(list.get(&"nonexistent".into()), None);
	/// ```
	pub fn insert(&mut self, channel_info: ChannelInfo) -> ChannelListResult<ChannelId> {
		let channel_id = channel_info.id.clone();
		if self.id_map.contains_key(&channel_id) {
			return Err(ChannelListError::ChannelAlreadyPresent(channel_id));
		}

		self.id_map.insert(channel_id.clone(), channel_info);

		Ok(channel_id)
	}

	/// Acquire a reference to a channel in the list.
	/// 
	/// # Example
	/// ```rust
	/// use snowcat::api::channels::{ChannelId, ChannelInfo};
	/// use snowcat::client::ChannelList;
	/// 
	/// let mut list = ChannelList::new();
	/// 
	/// let channel_id = "ADH-{example}";
	/// let channel_name = "Example Channel";
	/// 
	/// let channel = ChannelInfo {
	/// 	id: channel_id.into(),
	/// 	display_name: channel_name.to_owned(),
	/// 	character_count: 14,
	/// 
	/// 	is_official: false,
	/// };
	/// 
	/// let id = list.insert(channel.clone()).expect("channel should be inserted successfully");
	/// 
	/// assert_eq!(list.get(&id), Some(&channel));
	/// assert_eq!(list.get(&"nonexistent".into()), None);
	/// ```
	pub fn get(&self, id: &ChannelId) -> Option<&ChannelInfo> {
		self.id_map.get(id)
	}

	/// Acquire a mutable reference to a channel in the list.
	/// 
	/// # Example
	/// ```rust
	/// use snowcat::api::channels::{ChannelId, ChannelInfo};
	/// use snowcat::client::ChannelList;
	/// 
	/// let mut list = ChannelList::new();
	/// 
	/// let channel_id = "ADH-{example}";
	/// let channel_name = "Example Channel";
	/// 
	/// let channel = ChannelInfo {
	/// 	id: channel_id.into(),
	/// 	display_name: channel_name.to_owned(),
	/// 	character_count: 14,
	/// 
	/// 	is_official: false,
	/// };
	/// 
	/// let id = list.insert(channel.clone()).expect("channel should be inserted successfully");
	/// 
	/// assert_eq!(list.get(&id), Some(&channel));
	/// assert_eq!(list.get(&id).map(|channel| &*channel.display_name), Some(channel_name));
	/// 
	/// let new_name = "Homepage";
	/// if let Some(channel) = list.get_mut(&id) {
	/// 	channel.display_name = new_name.to_owned();
	/// }
	/// 
	/// assert_eq!(list.get(&id).map(|channel| &*channel.display_name), Some(new_name));
	/// ```
	pub fn get_mut(&mut self, id: &ChannelId) -> Option<&mut ChannelInfo> {
		self.id_map.get_mut(id)
	}

	/// Remove a channel from the list, returning it to the caller.
	/// 
	/// # Example
	/// ```rust
	/// use snowcat::api::channels::{ChannelId, ChannelInfo};
	/// use snowcat::client::ChannelList;
	/// 
	/// let mut list = ChannelList::new();
	/// 
	/// let channel_id = "ADH-{example}";
	/// let channel_name = "Example Channel";
	/// 
	/// let channel = ChannelInfo {
	/// 	id: channel_id.into(),
	/// 	display_name: channel_name.to_owned(),
	/// 	character_count: 14,
	/// 
	/// 	is_official: false,
	/// };
	/// 
	/// let id = list.insert(channel.clone()).expect("channel should be inserted successfully");
	/// let existing_channel = list.remove(&id).expect("channel should exist under the correct ID");
	/// 
	/// assert_eq!(list.get(&id), None);
	/// assert_eq!(existing_channel, channel);
	/// ```
	pub fn remove(&mut self, id: &ChannelId) -> ChannelListResult<ChannelInfo> {
		match self.id_map.remove(id) {
			Some(channel_info) => Ok(channel_info),
			None => Err(ChannelListError::ChannelNotFound(id.clone())),
		}
	}

	/// Iterate over the list of channels.
	pub fn iter(&self) -> impl Iterator<Item = &ChannelInfo> {
		self.id_map.values()
	}

	/// Iterate over the list of channels, sorted by a key extraction function.
	/// 
	/// # Allocates
	/// This function uses an intermediary allocation to sort the iterator.
	pub fn iter_sorted<T, F>(&self, mut key_fn: F) -> impl Iterator<Item = &ChannelInfo>
	where
		T: Ord,
		F: FnMut(&ChannelInfo) -> &T,
	{
		let mut vector = self.id_map.values().collect::<Vec<_>>();
		vector.sort_by_key(|channel| key_fn(*channel));
		vector.into_iter()
	}
}

impl Default for ChannelList {
	fn default() -> Self {
		ChannelList::new()
	}
}

#[derive(Debug, Clone, Error)]
pub enum ChannelListError {
	#[error(r#"The channel "{0}" is already present in the list."#)]
	ChannelAlreadyPresent(ChannelId),

	#[error(r#"The channel "{0}" was not found in the list."#)]
	ChannelNotFound(ChannelId),
}

pub struct CharacterList {
	id_map: BTreeMap<CharacterId, String>,
	name_map: BTreeMap<String, CharacterInfo>,
}

impl CharacterList {
	/// Create a new, empty character cache with the global allocator.
	///
	/// Does not allocate anything on its own.
	pub fn new() -> Self {
		CharacterList {
			id_map: BTreeMap::new(),
			name_map: BTreeMap::new(),
		}
	}

	/// Attempt to associate a character with a given ID. Fails if either the
	/// character or the ID are already associated.
	///
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::{
	/// 	CharacterList,
	/// 	CharacterListError,
	/// };
	///
	/// let mut list = CharacterList::new();
	///
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	///
	/// let character_id = CharacterId(25565);
	///
	/// let other_character_name = "Markelio";
	/// let other_character = CharacterInfo {
	/// 	name: other_character_name.to_owned(),
	/// 	gender: CharacterGender::Male,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Looking),
	/// };
	///
	/// let other_character_id = CharacterId(14);
	///
	/// list.insert(character)
	/// 	.expect(&format!("character should be inserted successfully: {character_name}"));
	///
	/// list.insert(other_character)
	/// 	.expect(&format!("character should be inserted successfully: {other_character_id}"));
	///
	/// assert_eq!(list.associate_id(character_name, character_id), Ok(()));
	/// assert_eq!(
	/// 	list.associate_id(character_name, other_character_id),
	/// 	Err(CharacterListError::CharacterAlreadyAssociated(character_name.to_owned(), character_id)),
	/// );
	///
	/// assert_eq!(
	/// 	list.associate_id(other_character_name, character_id),
	/// 	Err(CharacterListError::IdAlreadyExists(character_id)),
	/// );
	/// ```
	pub fn associate_id(&mut self, name: &str, id: CharacterId) -> CharacterListResult<()> {
		if !self.id_map.contains_key(&id) {
			let character = self.name_map.get(name).ok_or_else(|| {
				CharacterListError::CharacterNotFound(name.to_owned())
			})?;

			let existing_id = self.id_map.iter().find_map(|(&id, entry)| (&**entry == name).then_some(id));

			if let Some(id) = existing_id {
				Err(CharacterListError::CharacterAlreadyAssociated(name.to_owned(), id))
			} else {
				self.id_map.insert(id, character.name.clone());
				Ok(())
			}
		} else {
			Err(CharacterListError::IdAlreadyExists(id))
		}
	}

	/// Returns `true` if the list contains a character with the given name.
	/// 
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	/// 
	/// let mut list = CharacterList::new();
	/// 
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	/// 
	/// let character_id = 25565;
	/// 
	/// let _name = list.insert_with_id(character.clone(), CharacterId(character_id))
	/// 	.expect("character should be inserted successfully");
	/// 
	/// assert_eq!(list.contains_id(&CharacterId(character_id)), true);
	/// ```
	pub fn contains_id(&self, id: &CharacterId) -> bool {
		self.id_map.contains_key(id)
	}

	/// Returns `true` if the list contains a character with the given name.
	/// 
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	/// 
	/// let mut list = CharacterList::new();
	/// 
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	/// 
	/// let name = list.insert(character.clone()).expect("character should be inserted successfully");
	/// 
	/// assert_eq!(list.contains_name(&name), true);
	/// ```
	pub fn contains_name(&self, name: &str) -> bool {
		self.name_map.contains_key(name)
	}

	/// Insert a character into the list, using its name as the key.
	///
	/// Returns the character name to make future lookups easier.
	/// 
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	/// 
	/// let mut list = CharacterList::new();
	/// 
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	/// 
	/// let name = list.insert(character.clone()).expect("character should be inserted successfully");
	/// 
	/// assert_eq!(&name, character_name);
	/// assert_eq!(list.get_by_name(&name), Some(&character));
	/// ```
	pub fn insert(&mut self, character_info: CharacterInfo) -> CharacterListResult<String> {
		if self.name_map.contains_key(&character_info.name) {
			return Err(CharacterListError::CharacterAlreadyPresent(character_info.name));
		}

		let name = character_info.name.clone();

		self.name_map.insert(name.clone(), character_info);

		Ok(name)
	}

	/// Insert an id-character pair into the list, using its name as the key.
	///
	/// Returns the character name to make future lookups easier.
	/// 
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	/// 
	/// let mut list = CharacterList::new();
	/// 
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	/// 
	/// let character_id = 25565;
	/// 
	/// let name = list.insert_with_id(character.clone(), CharacterId(character_id))
	/// 	.expect("character should be inserted successfully");
	/// 
	/// assert_eq!(&name, character_name);
	/// assert_eq!(list.get_by_id(&CharacterId(character_id)), Some(&character));
	/// assert_eq!(list.get_by_id(&CharacterId(character_id)), list.get_by_name(&name));
	/// ```
	pub fn insert_with_id(&mut self, character_info: CharacterInfo, id: CharacterId) -> CharacterListResult<String> {
		if self.id_map.contains_key(&id) {
			return Err(CharacterListError::IdAlreadyExists(id));
		}

		let name = self.insert(character_info)?;

		self.id_map.insert(id, name.clone());

		Ok(name)
	}

	/// Acquire a reference by name to a character in the list.
	///
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	///
	///	let mut list = CharacterList::new();
	///
	///	let character_name = "Sarah Blitz Garisson";
	///	let character = CharacterInfo {
	///		name: character_name.to_owned(),
	///		gender: CharacterGender::Hermaphrodite,
	///		status: CharacterStatus::new(CharacterStatusKind::Offline),
	///	};
	///
	///	let character_id = 12;
	///
	///	list.insert(character.clone()).expect("character should be inserted successfully");
	///	list.associate_id(character_name, CharacterId(character_id)).expect("id association should be successful");
	///
	///	assert_eq!(list.get_by_id(&CharacterId(character_id)), Some(&character));
	///	assert_eq!(list.get_by_id(&CharacterId(14)), None);
	/// ```
	pub fn get_by_id(&self, id: &CharacterId) -> Option<&CharacterInfo> {
		let name = self.id_map.get(&id)?;
		self.name_map.get(name)
	}

	/// Acquire a mutable reference by ID to a character in the list.
	///
	/// To preserve name-value relations, this API should never be used to
	/// alter a character's name. Use [`set_name`](CharacterList::set_name)
	/// instead.
	///
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	///
	/// let mut list = CharacterList::new();
	///
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	///
	/// let character_id = 12;
	///
	/// list.insert(character.clone()).expect("character should be inserted successfully");
	/// list.associate_id(character_name, CharacterId(character_id)).expect("id association should be successful");
	///
	/// assert_eq!(list.get_by_id(&CharacterId(character_id)), Some(&character));
	/// 
	/// if let Some(character) = list.get_mut_by_id(&CharacterId(character_id)) {
	/// 	character.status = CharacterStatus::new_with_message("just woke up...", CharacterStatusKind::DoNotDisturb);
	/// }
	///
	/// assert_eq!(
	/// 	list.get_by_id(&CharacterId(character_id)).map(|char| &char.status),
	/// 	Some(&CharacterStatus::new_with_message("just woke up...", CharacterStatusKind::DoNotDisturb)),
	/// );
	/// ```
	pub fn get_mut_by_id(&mut self, id: &CharacterId) -> Option<&mut CharacterInfo> {
		let name = self.id_map.get(&id)?;
		self.name_map.get_mut(name)
	}

	/// Acquire a reference by name to a character in the list.
	///
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	///
	/// let mut list = CharacterList::new();
	///
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	///
	/// list.insert(character.clone()).expect("character should be inserted successfully");
	///
	/// assert_eq!(list.get_by_name(character_name), Some(&character));
	/// assert_eq!(list.get_by_name("nonexistent"), None);
	/// ```
	pub fn get_by_name(&self, name: &str) -> Option<&CharacterInfo> {
		self.name_map.get(name)
	}

	/// Acquire a mutable reference by name to a character in the list.
	///
	/// To preserve name-value relations, this API should never be used to
	/// alter a character's name. Use [`set_name`](CharacterList::set_name)
	/// instead.
	///
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	///
	/// let mut list = CharacterList::new();
	///
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	///
	/// list.insert(character.clone()).expect("character should be inserted successfully");
	///
	/// assert_eq!(list.get_by_name(character_name), Some(&character));
	///
	/// if let Some(character) = list.get_mut_by_name(character_name) {
	/// 	character.status = CharacterStatus::new_with_message("just woke up...", CharacterStatusKind::DoNotDisturb);
	/// }
	///
	/// assert_eq!(
	/// 	list.get_by_name(character_name).map(|char| &char.status),
	/// 	Some(&CharacterStatus::new_with_message("just woke up...", CharacterStatusKind::DoNotDisturb)),
	/// );
	/// ```
	pub fn get_mut_by_name(&mut self, name: &str) -> Option<&mut CharacterInfo> {
		self.name_map.get_mut(name)
	}

	/// Update a character's name.
	///
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	///
	/// let mut list = CharacterList::new();
	///
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	///
	/// list.insert(character.clone()).expect("character should be inserted successfully");
	///
	/// let new_name = "Markelio";
	/// list.set_name(character_name, new_name).expect("character should exist under the correct name");
	///
	/// assert_eq!(list.get_by_name(new_name).map(|character| &*character.name), Some(new_name));
	/// assert_eq!(list.get_by_name(character_name), None);
	/// ```
	pub fn set_name(&mut self, character: &str, new_name: &str) -> CharacterListResult<()> {
		let (mut character, character_id) = self.remove_by_name(character)?;

		character.name = new_name.to_owned();

		self.name_map.insert(new_name.to_owned(), character);

		if let Some(character_id) = character_id {
			self.id_map.insert(character_id, new_name.to_owned());
		}

		Ok(())
	}

	/// Update a character's name using their ID.
	///
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	///
	/// let mut list = CharacterList::new();
	///
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	///
	/// let character_id = 25565;
	///
	/// list.insert(character.clone()).expect("character should be inserted successfully");
	/// list.associate_id(character_name, CharacterId(character_id)).expect("id association should be successful");
	///
	/// let new_name = "Markelio";
	/// list.set_name_by_id(CharacterId(character_id), new_name)
	/// 	.expect("character should exist under the correct ID");
	///
	/// assert_eq!(list.get_by_id(&CharacterId(character_id)).map(|character| &*character.name), Some(new_name));
	/// assert_eq!(list.get_by_name(character_name), None);
	/// ```
	pub fn set_name_by_id(&mut self, id: CharacterId, new_name: &str) -> CharacterListResult<()> {
		let mut character = self.remove_by_id(id)?;

		character.name = new_name.to_owned();

		self.name_map.insert(new_name.to_owned(), character);
		self.id_map.insert(id, new_name.to_owned());

		Ok(())
	}

	/// Remove a character from the map using their ID.
	/// 
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterId,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	/// 
	/// let mut list = CharacterList::new();
	/// 
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	/// 
	/// let character_id = 25565;
	/// 
	/// list.insert(character.clone()).expect("character should be inserted successfully");
	/// list.associate_id(character_name, CharacterId(character_id)).expect("id association should be successful");
	/// 
	/// assert_eq!(list.get_by_id(&CharacterId(character_id)), Some(&character));
	/// 
	/// let existing_character = list.remove_by_id(CharacterId(character_id))
	/// 	.expect("character should exist under the correct ID");
	/// 
	/// assert_eq!(list.get_by_id(&CharacterId(character_id)), None);
	/// assert_eq!(existing_character, character);
	/// ```
	pub fn remove_by_id(&mut self, id: CharacterId) -> CharacterListResult<CharacterInfo> {
		let character_name = self.id_map.remove(&id).ok_or_else(|| CharacterListError::IdNotFound(id))?;
		let character = self.name_map.remove(&character_name)
			.expect("character has a valid ID, and therefore should be present");

		Ok(character)
	}

	/// Remove a character from the map.
	/// 
	/// # Example
	/// ```
	/// use snowcat::api::characters::{
	/// 	CharacterGender,
	/// 	CharacterInfo,
	/// 	CharacterStatus,
	/// 	CharacterStatusKind,
	/// };
	/// use snowcat::client::CharacterList;
	/// 
	/// let mut list = CharacterList::new();
	/// 
	/// let character_name = "Sarah Blitz Garisson";
	/// let character = CharacterInfo {
	/// 	name: character_name.to_owned(),
	/// 	gender: CharacterGender::Hermaphrodite,
	/// 	status: CharacterStatus::new(CharacterStatusKind::Offline),
	/// };
	/// 
	/// list.insert(character.clone()).expect("character should be inserted successfully");
	/// 
	/// assert_eq!(list.get_by_name(character_name), Some(&character));
	/// 
	/// let (existing_character, _) = list.remove_by_name(character_name)
	/// 	.expect("character should exist under the correct ID");
	/// 
	/// assert_eq!(list.get_by_name(character_name), None);
	/// assert_eq!(existing_character, character);
	/// ```
	pub fn remove_by_name(&mut self, character: &str) -> CharacterListResult<(CharacterInfo, Option<CharacterId>)> {
		let character_id = self.id_map.iter().find_map(|(&id, name)| (&**name == character).then_some(id));

		let character = self.name_map.remove(character)
			.ok_or_else(|| CharacterListError::CharacterNotFound(character.to_owned()))?;
		
		if let Some(character_id) = character_id {
			self.id_map.remove(&character_id);
		}

		Ok((character, character_id))
	}
}

impl Default for CharacterList {
	fn default() -> Self {
		CharacterList::new()
	}
}

impl Index<&CharacterId> for CharacterList {
	type Output = CharacterInfo;

	fn index(&self, id: &CharacterId) -> &Self::Output {
		self.get_by_id(id).unwrap_or_else(|| panic!("character ID not found: {id}"))
	}
}

impl IndexMut<&CharacterId> for CharacterList {
	fn index_mut(&mut self, id: &CharacterId) -> &mut Self::Output {
		self.get_mut_by_id(id).unwrap_or_else(|| panic!("character ID not found: {id}"))
	}
}

impl<K> Index<&K> for CharacterList
where
	K: Borrow<str>,
{
	type Output = CharacterInfo;

	fn index(&self, name: &K) -> &Self::Output {
		let name = name.borrow();
		self.get_by_name(name).unwrap_or_else(|| panic!("character not found: {name}"))
	}
}

impl<K> IndexMut<&K> for CharacterList
where
	K: Borrow<str>,
{
	fn index_mut(&mut self, name: &K) -> &mut Self::Output {
		let name = name.borrow();
		self.get_mut_by_name(name).unwrap_or_else(|| panic!("character not found: {name}"))
	}
}

#[derive(Debug, Clone, Error, Eq, PartialEq)]
pub enum CharacterListError {
	#[error("The character {0:?} is already associated with ID {1}")]
	CharacterAlreadyAssociated(String, CharacterId),

	#[error("The character {0:?} is already in the list.")]
	CharacterAlreadyPresent(String),

	#[error("Did not find character {0:?} in the list.")]
	CharacterNotFound(String),

	#[error("The ID {0} is already associated with a value.")]
	IdAlreadyExists(CharacterId),

	#[error("Did not find character ID {0} in the list.")]
	IdNotFound(CharacterId),
}

fn _assert_client_traits()
where
	Client: Send + Sync + 'static,
{ }
