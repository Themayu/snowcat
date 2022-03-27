use crate::user::Credentials;
use serde::{Deserialize, Serialize};

/// Make an attempt at logging in. Transfers a [`user::Credentials`] object,
/// and returns [`Result<(), error::Error>`].
pub const LOGIN: &'static str = "account_login";

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginArgs {
	credentials: Credentials,
}

impl LoginArgs {
	pub fn new(credentials: Credentials) -> Self {
		LoginArgs {
			credentials,
		}
	}
}
