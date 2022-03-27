use serde::{Deserialize, Serialize};
use thiserror::Error;

/// An error occurred during execution. 
#[derive(Debug, Clone, Deserialize, Error, Serialize)]
pub enum Error {
	#[error(transparent)]
	ApiError(#[from] ApiError),

	#[error(transparent)]
	CommandError(#[from] CommandError),
	
	#[error(transparent)]
	ConnectionError(#[from] ConnectionError),

	#[error(transparent)]
	RequestError(#[from] RequestError),
	
	#[error(transparent)]
	ResponseError(#[from] ResponseError),

	#[error("Expected a {} object, which was not returned from the backend.", .0.to_lowercase())]
	MissingValueError(String),
}

/// The F-List API gave an error message in response to a request.
/// 
/// Instances of this enum are created by matching against well-known error
/// messages. Any message that is not recognised will be wrapped in a `Generic`
/// value and made available to the caller.
#[derive(Debug, Clone, Deserialize, Error, Serialize)]
pub enum ApiError {
	/// The error message was not recognised and is made available to the
	/// caller for handling.
	#[error("An unexpected error was received from the API: {0}")]
	Generic(String),

	/// Could not access a character's data.
	/// 
	/// This usually means that the character in question was banned, or the
	/// account it belongs to was banned.
	#[error("Could not access this character's information.")]
	CharacterAccessDenied,

	/// This character does not have a guestbook.
	#[error("This character does not have a guestbook.")]
	CharacterGuestbookDisabled,

	/// This character does not publish their friends list.
	#[error("This character does not publish their friends list.")]
	CharacterFriendsAccessDenied,

	/// No character exists under this name.
	/// 
	/// If a character previously existed with this name, this usually means
	/// the character was renamed.
	#[error("This character could not be found.")]
	CharacterNotFound,

	/// An incorrect username or password was provided.
	#[error("Login Failed. If you have forgotten your username or password, please use the website to recover them.")]
	InvalidCredentials,
}

/// F-Chat gave an error message in response to a command. Instances of this
/// enum are created by matching against well-known error messages. Any message
/// that is not recognised will be wrapped in a `Generic` value and made
/// available to the caller.
#[derive(Debug, Clone, Deserialize, Error, Serialize)]
pub enum CommandError {
	#[error(".")]
	IdentificationFailed,	

	/// The error message was not recognised and is made available to the
	/// caller for handling.
	#[error("Received unknown error code {code}: {message}")]
	Generic {
		code: i16,
		message: String
	},
}

/// An error occurred during a network connection.
#[derive(Debug, Clone, Deserialize, Error, Serialize)]
pub enum ConnectionError {
	/// No connection could be established between the client and the server.
	#[error("Failed to connect to the server at {0}")]
	ConnectionFailed(String),

	/// No error message was found for this error.
	#[error("The connection ran into an error: {0}")]
	Generic(String),

	/// The connection closed while in the middle of sending a message from
	/// one side to the other.
	#[error("The connection closed while a message was being sent.")]
	IncompleteMessage,

	/// The connection could not be established because the server was not
	/// found.
	#[error("No server could be found at {0}")]
	ServerNotFound(String),
}

/// An error occurred while making a HTTP request.
#[derive(Debug, Clone, Deserialize, Error, Serialize)]
pub enum RequestError {
	/// The request was cancelled.
	#[error("The request was cancelled while being sent.")]
	Cancelled,

	/// The request body could not be serialized.
	#[error("Could not serialize the request body.")]
	SerializationFailed,
}

/// An error occurred while interacting with a HTTP response message.
#[derive(Debug, Clone, Deserialize, Error, Serialize)]
pub enum ResponseError {
	/// The response body could not be deserialized.
	/// 
	/// This usually means that the Rust object definition did not match the 
	/// shape of the data coming from the remote service.
	#[error("The remote service response could not be deserialized.")]
	DeserializationFailed,

	/// The response body could not be read into memory to be deserialized.
	#[error("Could not read the remote service response into memory.")]
	ReadFailed,
}
