use crate::api::characters::{CharacterId, CharacterInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthenticateReturn(Vec<CharacterInfo>, CharacterId);
impl AuthenticateReturn {
	pub fn new(characters_list: Vec<CharacterInfo>, default_character: CharacterId) -> AuthenticateReturn {
		AuthenticateReturn(characters_list, default_character)
	}

	pub fn default_character(&self) -> CharacterId {
		self.1
	}

	pub fn characters_list(&self) -> &Vec<CharacterInfo> {
		&self.0
	}
}
