use thiserror::Error;

use crate::api::ApiClient;
use crate::api::characters::{CharacterId, CharacterInfo};
use std::borrow::Borrow;
use std::collections::BTreeMap;
use std::ops::{Index, IndexMut};

pub type CharacterListResult<T> = Result<T, CharacterListError>;

pub struct Client {
	api: Option<ApiClient>,
	character_cache: CharacterList,
}

pub struct ChannelList(BTreeMap<String, /*ChannelInfo*/ ()>);

impl ChannelList {
	/// Create a new, empty channel cache with the global allocator.
	///
	/// Does not allocate anything on its own.
	pub fn new() -> Self {
		ChannelList(BTreeMap::new())
	}
}

impl Default for ChannelList {
	fn default() -> Self {
		ChannelList::new()
	}
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

	#[error("Cannot acquire a mutable reference to character {0:?} without violating memory safety rules")]
	CannotMutate(String),
}
