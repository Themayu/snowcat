use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
	pub characters: HashMap<u64, String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Credentials {
	pub account: String,
	pub password: String,
	pub server_url: String,

	pub save_credentials: bool,
}
