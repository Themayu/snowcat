use serde::{Deserialize, Serialize};
use serde_with::SerializeDisplay;
use std::fmt;

pub mod client;
pub mod server;

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ChannelId(String);

// #[derive(Debug, Deserialize, Serialize)]
// pub struct ChannelName(String);

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChannelMode {
	Chat,
	Ads,
	Both,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CharacterTypingStatus {
	#[default] Clear,
	Paused,
	Typing,
}

#[derive(Debug, SerializeDisplay)]
pub struct ClientVersion(u8, u8, u16);
impl fmt::Display for ClientVersion {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}.{}.{}", self.0, self.1, self.2)
	}
}
