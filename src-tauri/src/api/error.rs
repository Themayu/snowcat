use std::convert::Infallible;
use std::str;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, NoneAsEmptyString};
use snowcat_macros::discriminate;
use thiserror::Error;

const INVALID_LOGIN_CREDENTIALS: &str = "Login Failed. If you have forgotten your username or password, please use the website to recover them.";
const INVALID_TICKET: &str = "Invalid ticket";

pub type Result<T> = std::result::Result<T, ApiError>;

#[discriminate]
#[derive(Debug, Error, Deserialize, Serialize)]
pub enum ApiError {
	#[error("err-invalid-login-credentials")]
	InvalidLoginCredentials,

	#[error("err-invalid-ticket")]
	InvalidTicket,

	#[error("err-other")]
	Other(String),
}

impl str::FromStr for ApiError {
	type Err = Infallible;
	
	fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
		match s {
			INVALID_LOGIN_CREDENTIALS => Ok(ApiError::InvalidLoginCredentials),
			INVALID_TICKET => Ok(ApiError::InvalidTicket),
			_ => Ok(ApiError::Other(String::from(s))),
		}
	}
}

#[serde_as]
#[derive(Deserialize)]
pub(super) struct DeserializeError<T> {
	#[serde(flatten)]
	data: Option<T>,

	#[serde_as(as = "NoneAsEmptyString")]
	error: Option<String>,
}

impl<T> DeserializeError<T> {
	pub fn from_value(data: T) -> Self {
		DeserializeError {
			data: Some(data),
			error: None,
		}
	}

	pub fn into_result(self) -> Result<T> {
		let DeserializeError { data, error } = self;

		match error {
			Some(err) => Err(err.parse().expect("conversion is infallible")),
			None => Ok(data.expect("data should exist"))
		}
	}
}
