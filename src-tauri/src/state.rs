mod mock;

use snowcat_common::settings::Settings;
use snowcat_common::state::character::Character;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/// A wrapper type to store a map of characters to character IDs.
pub struct CharacterMap(pub HashMap<u64, Character>);
impl CharacterMap {
	/// Insert a character into the character map, returning the old value if
	/// one was previously present.
	pub fn insert(&mut self, id: u64, name: &str) -> Option<Character> {
		self.0.insert(id, Character { id, name: name.to_owned() })
	}

	/// Remove a character from the character map, returning the removed value
	/// if it was previously present.
	pub fn remove(&mut self, id: u64) -> Option<Character> {
		self.0.remove(&id)
	}

	/// Remove a character from the character map, returning a tuple of the ID
	/// and removed value if it was previously present.
	pub fn remove_entry(&mut self, id: u64) -> Option<(u64, Character)> {
		self.0.remove_entry(&id)
	}
}

impl Default for CharacterMap {
	fn default() -> Self {
		let mut map = CharacterMap(HashMap::default());

		mock::CHARACTERS.iter()
			.for_each(|(id, name)| { map.insert(*id, *name); });
		
		map
	}
}

impl Deref for CharacterMap {
	type Target = HashMap<u64, Character>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for CharacterMap {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

pub struct User<'data> {
	pub my_characters: Vec<&'data Character>,
}

pub async fn read_settings_file(_name: &str) -> Settings {
	Settings::default()
}

pub async fn write_settings_file(_settings: &Settings) {
	todo!("write settings to file")
}
